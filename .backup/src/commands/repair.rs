/*
 * DiskParted - A Disk Management Tool
 * Copyright (C) 2026 DiskParted Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::process::Command;
use crate::context::Context;

/// Run the REPAIR command.
///
/// DiskPart syntax (RAID-5 on dynamic disks — Windows only):
///   repair disk=<n> [align=<n>] [noerr]
///
/// On Linux, dynamic disks / RAID-5 volumes are not supported, so this command
/// instead detects what volume manager (if any) is managing the selected partition
/// and delegates to the appropriate repair tool:
///
///   - md (Linux software RAID)  →  mdadm --manage --re-add / --add
///   - ZFS                       →  zpool scrub / zpool replace
///   - LVM                       →  pvck / vgck / lvconvert --repair
///
/// Usage:
///   repair disk=<n> [align=<n>] [noerr]
///
///   disk=<n>   — the replacement disk index (from `list disk`) to add into the array.
///                Pass disk=0 if you just want to scrub/check without replacing.
///   align=<n>  — alignment in KB (passed through to mdadm where applicable; ignored elsewhere).
///   noerr      — on error, continue rather than stopping (mirrors DiskPart scripting behaviour).
pub fn run(args: &[&str], ctx: &mut Context) {
    // Parse flags
    let mut disk_n: Option<u32>  = None;
    let mut align_kb: Option<u32> = None;
    let mut noerr = false;

    for arg in args {
        if let Some(val) = arg.strip_prefix("disk=") {
            match val.parse::<u32>() {
                Ok(n) => disk_n = Some(n),
                Err(_) => { println!("Invalid value for disk=: '{}'.", val); return; }
            }
        } else if let Some(val) = arg.strip_prefix("align=") {
            match val.parse::<u32>() {
                Ok(n) => align_kb = Some(n),
                Err(_) => { println!("Invalid value for align=: '{}'.", val); return; }
            }
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    // A volume must be selected
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    // Resolve replacement disk path (if disk= was supplied and != 0)
    let replacement_disk: Option<String> = match disk_n {
        Some(0) | None => None, // scrub / check only
        Some(n) => {
            use crate::commands::list::get_disks;
            match get_disks().into_iter().find(|d| d.index == n) {
                Some(d) => Some(d.path),
                None => {
                    println!("Disk {} not found. Run 'list disk' to see available disks.", n);
                    if !noerr { return; }
                    None
                }
            }
        }
    };

    // Detect which volume manager owns this partition and dispatch
    let vol_type = detect_volume_type(&partition.path);

    match vol_type.as_deref() {
        Some("md")  => repair_md(&partition.path, replacement_disk.as_deref(), align_kb, noerr),
        Some("zfs") => repair_zfs(&partition.path, replacement_disk.as_deref(), noerr),
        Some("lvm") => repair_lvm(&partition.path, replacement_disk.as_deref(), noerr),
        _ => {
            println!("The selected volume is not part of a software RAID, ZFS pool, or LVM group.");
            println!("DiskPart's 'repair' command targets RAID-5 volumes (Windows dynamic disks).");
            println!();
            println!("For plain partitions, use 'format' to reinitialise the filesystem, or use");
            println!("fsck directly:  sudo fsck -n {}", partition.path);
        }
    }
}

// ---------------------------------------------------------------------------
// Volume type detection
// ---------------------------------------------------------------------------

/// Detect whether a partition is part of an md array, a ZFS pool, or LVM.
/// Returns Some("md"), Some("zfs"), Some("lvm"), or None.
fn detect_volume_type(part_path: &str) -> Option<String> {
    // ---- md ----------------------------------------------------------------
    // /proc/mdstat lists all active md arrays and their member devices
    if let Ok(mdstat) = std::fs::read_to_string("/proc/mdstat") {
        // Strip /dev/ prefix for matching
        let dev_name = part_path.trim_start_matches("/dev/");
        if mdstat.contains(dev_name) {
            return Some("md".to_string());
        }
    }

    // ---- ZFS ---------------------------------------------------------------
    // `zpool status` lists all pools and their member vdevs
    if which::which("zpool").is_ok() {
        let out = Command::new("sudo")
            .args(["zpool", "status"])
            .output();
        if let Ok(o) = out {
            let text = String::from_utf8_lossy(&o.stdout);
            let dev_name = part_path.trim_start_matches("/dev/");
            if text.contains(dev_name) {
                return Some("zfs".to_string());
            }
        }
    }

    // ---- LVM ---------------------------------------------------------------
    // `pvs` lists physical volumes; if our partition appears it's LVM
    if which::which("pvs").is_ok() {
        let out = Command::new("sudo")
            .args(["pvs", "--noheadings", "-o", "pv_name"])
            .output();
        if let Ok(o) = out {
            let text = String::from_utf8_lossy(&o.stdout);
            if text.lines().any(|l| l.trim() == part_path) {
                return Some("lvm".to_string());
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// md (Linux software RAID via mdadm)
// ---------------------------------------------------------------------------

fn repair_md(part_path: &str, replacement: Option<&str>, _align_kb: Option<u32>, noerr: bool) {
    // Find the md device that owns this partition
    let md_dev = match find_md_device(part_path) {
        Some(d) => d,
        None => {
            println!("Could not determine which md array contains {}.", part_path);
            if !noerr { return; }
            return;
        }
    };

    println!("Detected Linux software RAID array: {}", md_dev);
    println!();

    if let Some(repl) = replacement {
        // --- Replace: mark failed then add new disk ---
        println!("Step 1/2 — Marking {} as failed in {}...", part_path, md_dev);
        let fail_ok = Command::new("sudo")
            .args(["mdadm", "--manage", &md_dev, "--fail", part_path])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !fail_ok {
            println!("Warning: could not mark device as failed (it may already be failed).");
        }

        println!("Step 2/2 — Adding replacement disk {} to {}...", repl, md_dev);
        let add_status = Command::new("sudo")
            .args(["mdadm", "--manage", &md_dev, "--add", repl])
            .status()
            .expect("Failed to execute mdadm");

        if add_status.success() {
            println!("Replacement disk added. Array rebuild has started.");
            println!("Monitor rebuild progress with:  cat /proc/mdstat");
        } else {
            println!("Failed to add replacement disk to array.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    } else {
        // --- No replacement: attempt re-add of existing (possibly failed) member ---
        println!("No replacement disk specified (disk=0 or omitted).");
        println!("Attempting to re-add {} to {} (use if device was temporarily missing)...", part_path, md_dev);

        let status = Command::new("sudo")
            .args(["mdadm", "--manage", &md_dev, "--re-add", part_path])
            .status()
            .expect("Failed to execute mdadm");

        if status.success() {
            println!("Device re-added. Resync started if needed.");
            println!("Monitor progress with:  cat /proc/mdstat");
        } else {
            println!("Re-add failed. The device may be too damaged to re-add.");
            println!("Specify a replacement with:  repair disk=<n>");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    }
}

/// Find the /dev/mdX device that contains the given partition by parsing /proc/mdstat.
fn find_md_device(part_path: &str) -> Option<String> {
    let mdstat = std::fs::read_to_string("/proc/mdstat").ok()?;
    let dev_name = part_path.trim_start_matches("/dev/");

    let mut current_md: Option<String> = None;
    for line in mdstat.lines() {
        // Lines like "md1 : active raid5 sdb1[1] sdc1[2] sdd1[0]"
        if line.starts_with("md") && line.contains(':') {
            current_md = line.split_whitespace().next().map(|s| format!("/dev/{}", s));
        } else if let Some(ref md) = current_md {
            if line.contains(dev_name) {
                return Some(md.clone());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// ZFS (via zpool)
// ---------------------------------------------------------------------------

fn repair_zfs(part_path: &str, replacement: Option<&str>, noerr: bool) {
    // Find the pool name that contains this partition
    let pool = match find_zfs_pool(part_path) {
        Some(p) => p,
        None => {
            println!("Could not determine which ZFS pool contains {}.", part_path);
            if !noerr { return; }
            return;
        }
    };

    println!("Detected ZFS pool: {}", pool);
    println!();

    if let Some(repl) = replacement {
        // zpool replace <pool> <failed-vdev> <new-disk>
        println!("Replacing {} with {} in ZFS pool '{}'...", part_path, repl, pool);

        let status = Command::new("sudo")
            .args(["zpool", "replace", &pool, part_path, repl])
            .status()
            .expect("Failed to execute zpool");

        if status.success() {
            println!("ZFS resilver (rebuild) started on pool '{}'.", pool);
            println!("Monitor progress with:  sudo zpool status {}", pool);
        } else {
            println!("zpool replace failed.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    } else {
        // No replacement — run a scrub to detect and repair any correctable errors
        println!("No replacement disk specified. Running scrub on ZFS pool '{}'...", pool);
        println!("(A scrub reads all data and corrects any errors that can be recovered from redundancy.)");

        let status = Command::new("sudo")
            .args(["zpool", "scrub", &pool])
            .status()
            .expect("Failed to execute zpool");

        if status.success() {
            println!("Scrub started on pool '{}'.", pool);
            println!("Monitor progress with:  sudo zpool status {}", pool);
        } else {
            println!("zpool scrub failed.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    }
}

/// Find the ZFS pool name that contains the given partition.
fn find_zfs_pool(part_path: &str) -> Option<String> {
    let out = Command::new("sudo")
        .args(["zpool", "status"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&out.stdout);
    let dev_name = part_path.trim_start_matches("/dev/");

    let mut current_pool: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        // Lines like "pool: tank"
        if let Some(name) = trimmed.strip_prefix("pool:") {
            current_pool = Some(name.trim().to_string());
        }
        // Vdev lines contain the device name
        if trimmed.contains(dev_name) {
            if let Some(pool) = current_pool {
                return Some(pool);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// LVM (via pvck / vgck / lvconvert)
// ---------------------------------------------------------------------------

fn repair_lvm(part_path: &str, replacement: Option<&str>, noerr: bool) {
    // Find which volume group owns this PV
    let vg = find_lvm_vg(part_path);

    println!("Detected LVM physical volume: {}", part_path);
    if let Some(ref v) = vg {
        println!("Volume group: {}", v);
    }
    println!();

    if let Some(repl) = replacement {
        // Move all extents off the failing PV onto the replacement, then remove it
        println!("Step 1/3 — Initialising {} as a new LVM physical volume...", repl);
        let pvcreate_ok = Command::new("sudo")
            .args(["pvcreate", repl])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !pvcreate_ok {
            println!("pvcreate failed on {}.", repl);
            if !noerr { return; }
        }

        if let Some(ref vg_name) = vg {
            println!("Step 2/3 — Extending volume group '{}' with {}...", vg_name, repl);
            let vgextend_ok = Command::new("sudo")
                .args(["vgextend", vg_name, repl])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if !vgextend_ok {
                println!("vgextend failed.");
                if !noerr { return; }
            }

            println!("Step 3/3 — Moving extents from {} to {} (this may take a while)...", part_path, repl);
            let pvmove_status = Command::new("sudo")
                .args(["pvmove", part_path, repl])
                .status()
                .expect("Failed to execute pvmove");

            if pvmove_status.success() {
                println!("Extents moved successfully.");
                println!("You can now remove the old PV with:  sudo vgreduce {} {}", vg_name, part_path);
                println!("And then:                            sudo pvremove {}", part_path);
            } else {
                println!("pvmove failed. The volume group may be in a degraded state.");
                println!("Check with:  sudo vgck {}", vg_name);
                if noerr { println!("(noerr: continuing despite error)"); }
            }
        } else {
            println!("Could not determine volume group for {}. Cannot proceed with pvmove.", part_path);
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    } else {
        // No replacement — run pvck and vgck to check/repair metadata
        println!("No replacement disk specified (disk=0 or omitted).");
        println!("Running LVM metadata check on {}...", part_path);

        let pvck = Command::new("sudo")
            .args(["pvck", "--verbose", part_path])
            .status()
            .expect("Failed to execute pvck");

        if pvck.success() {
            println!("pvck: physical volume metadata is consistent.");
        } else {
            println!("pvck: metadata errors detected on {}.", part_path);
        }

        if let Some(ref vg_name) = vg {
            println!("Running volume group check on '{}'...", vg_name);
            let vgck = Command::new("sudo")
                .args(["vgck", "--verbose", vg_name])
                .status()
                .expect("Failed to execute vgck");

            if vgck.success() {
                println!("vgck: volume group '{}' is consistent.", vg_name);
            } else {
                println!("vgck: errors found in volume group '{}'.", vg_name);
                println!("Attempting to repair with vgck --updatemetadata...");

                let repair = Command::new("sudo")
                    .args(["vgck", "--updatemetadata", vg_name])
                    .status()
                    .expect("Failed to execute vgck");

                if repair.success() {
                    println!("Volume group metadata repaired.");
                } else {
                    println!("Automatic repair failed. Manual intervention may be required.");
                    if noerr { println!("(noerr: continuing despite error)"); }
                }
            }
        }
    }
}

/// Find the LVM volume group that contains the given physical volume.
fn find_lvm_vg(part_path: &str) -> Option<String> {
    let out = Command::new("sudo")
        .args(["pvs", "--noheadings", "-o", "pv_name,vg_name"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let mut cols = line.split_whitespace();
        let pv = cols.next().unwrap_or("");
        let vg = cols.next().unwrap_or("");
        if pv == part_path && !vg.is_empty() {
            return Some(vg.to_string());
        }
    }
    None
}
