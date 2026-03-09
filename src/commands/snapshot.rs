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

//! SNAPSHOT  [!]
//!
//! Create, list, delete, or restore snapshots of the selected volume.
//! DiskParted Linux extension — no equivalent in Windows DiskPart.
//!
//! Syntax
//! ------
//!   snapshot create  [name=<label>]
//!   snapshot list
//!   snapshot delete  <name>
//!   snapshot restore <name>
//!
//! Subcommands
//! -----------
//!   create [name=<x>]   Create a snapshot. If name is omitted a timestamped
//!                       name is generated automatically.
//!
//!   list                List all snapshots for the selected volume.
//!
//!   delete <n>          Delete a named snapshot.
//!
//!   restore <n>         Restore the volume to a snapshot state.
//!                       LVM : schedules a merge on next reboot.
//!                       Btrfs: guides the user through a subvolume swap.
//!
//! Backend detection
//! -----------------
//!   LVM   — detected when the selected partition is an LVM logical volume
//!            (`lvdisplay` succeeds).
//!   Btrfs — detected from the partition's FSTYPE via lsblk.
//!
//! Requires
//! --------
//!   LVM   — lvm2          (pacman -S lvm2)
//!   Btrfs — btrfs-progs   (pacman -S btrfs-progs)

use std::process::Command;
use crate::context::Context;
use crate::utils::confirm;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        print_usage();
        return;
    }

    let part_dev = match ctx.selected_partition.as_ref() {
        Some(p) => p.clone(),
        None => {
            eprintln!("There is no partition selected.");
            eprintln!("Use 'select partition <n>' first.");
            return;
        }
    };

    let backend = detect_backend(&*part_dev);

    // Parse optional name= from trailing args.
    let name_arg: Option<String> = args
        .iter()
        .skip(1)
        .find(|a| a.to_lowercase().starts_with("name="))
        .map(|a| a[5..].to_string());

    match args[0].to_lowercase().as_str() {
        "create" => snap_create(&*part_dev, &backend, name_arg.as_deref()),
        "list" => snap_list(&*part_dev, &backend),
        "delete" => {
            let name = match args.get(1) {
                Some(n) => *n,
                None => {
                    eprintln!("Syntax: snapshot delete <name>");
                    return;
                }
            };
            snap_delete(&*part_dev, &backend, name);
        }
        "restore" => {
            let name = match args.get(1) {
                Some(n) => *n,
                None => {
                    eprintln!("Syntax: snapshot restore <name>");
                    return;
                }
            };
            snap_restore(&*part_dev, &backend, name);
        }
        other => {
            eprintln!(
                "Unknown subcommand: '{}'. Valid: create, list, delete, restore.",
                other
            );
            print_usage();
        }
    }
}

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

enum Backend {
    Lvm,
    Btrfs,
    Unsupported(String),
}

fn detect_backend(dev: &str) -> Backend {
    // Try lvdisplay first — succeeds only for LVM logical volumes.
    if Command::new("lvdisplay")
        .arg(dev)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Backend::Lvm;
    }

    let fstype = Command::new("lsblk")
        .args(["-no", "FSTYPE", dev])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase())
        .unwrap_or_default();

    match fstype.as_str() {
        "btrfs" => Backend::Btrfs,
        other => Backend::Unsupported(other.to_string()),
    }
}

fn unsupported_message(fstype: &str) {
    eprintln!(
        "Snapshots are not supported for '{}' volumes.",
        if fstype.is_empty() { "unformatted" } else { fstype }
    );
    eprintln!("Supported backends: LVM logical volumes, Btrfs subvolumes.");
}

// ---------------------------------------------------------------------------
// LVM helpers
// ---------------------------------------------------------------------------

fn lvm_vg_lv(dev: &str) -> Option<(String, String)> {
    let out = Command::new("lvdisplay")
        .args(["--columns", "--noheadings", "-o", "vg_name,lv_name", dev])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.split_whitespace();
    let vg = parts.next()?.to_string();
    let lv = parts.next()?.to_string();
    Some((vg, lv))
}

fn timestamp_name() -> String {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("snap_{}", epoch)
}

fn lvm_create(dev: &str, name: Option<&str>) {
    let snap_name = name.map(str::to_string).unwrap_or_else(timestamp_name);
    let (vg, lv) = match lvm_vg_lv(dev) {
        Some(v) => v,
        None => {
            eprintln!("Could not determine VG/LV for {}.", dev);
            return;
        }
    };

    println!("Creating LVM snapshot '{}' of {}/{}...", snap_name, vg, lv);

    match Command::new("lvcreate")
        .args([
            "--snapshot",
            "--name", &snap_name,
            "--size", "1G",
            &format!("{}/{}", vg, lv),
        ])
        .status()
    {
        Ok(s) if s.success() => {
            println!("Snapshot created: /dev/{}/{}", vg, snap_name);
        }
        Ok(_) => eprintln!(
            "lvcreate failed. \
             Ensure there is enough free space in volume group '{}'.",
            vg
        ),
        Err(e) => eprintln!("Failed to run lvcreate: {}", e),
    }
}

fn lvm_list(dev: &str) {
    let (vg, lv) = match lvm_vg_lv(dev) {
        Some(v) => v,
        None => {
            eprintln!("Could not determine VG/LV for {}.", dev);
            return;
        }
    };

    println!("LVM snapshots of {}/{}:", vg, lv);
    let _ = Command::new("lvdisplay")
        .args([
            "--columns",
            "-o", "lv_name,lv_size,origin",
            &format!("/dev/{}", vg),
        ])
        .status();
}

fn lvm_delete(dev: &str, name: &str) {
    let (vg, _) = match lvm_vg_lv(dev) {
        Some(v) => v,
        None => {
            eprintln!("Could not determine VG for {}.", dev);
            return;
        }
    };

    if !confirm(&format!("Delete LVM snapshot {}/{}?", vg, name)) {
        println!("No change was made.");
        return;
    }

    match Command::new("lvremove")
        .args(["--force", &format!("{}/{}", vg, name)])
        .status()
    {
        Ok(s) if s.success() => println!("Snapshot {}/{} deleted.", vg, name),
        Ok(_) => eprintln!("lvremove failed."),
        Err(e) => eprintln!("Failed to run lvremove: {}", e),
    }
}

fn lvm_restore(dev: &str, name: &str) {
    let (vg, lv) = match lvm_vg_lv(dev) {
        Some(v) => v,
        None => {
            eprintln!("Could not determine VG/LV for {}.", dev);
            return;
        }
    };

    println!(
        "Scheduling merge of snapshot '{}/{}' into '{}/{}'.",
        vg, name, vg, lv
    );
    println!("NOTE: The merge will complete on the next reboot.");
    println!();

    if !confirm("Proceed with scheduling the snapshot merge?") {
        println!("No change was made.");
        return;
    }

    match Command::new("lvconvert")
        .args(["--mergesnapshot", &format!("{}/{}", vg, name)])
        .status()
    {
        Ok(s) if s.success() => {
            println!("Merge scheduled. Reboot to apply the snapshot restore.");
        }
        Ok(_) => eprintln!("lvconvert merge failed."),
        Err(e) => eprintln!("Failed to run lvconvert: {}", e),
    }
}

// ---------------------------------------------------------------------------
// Btrfs helpers
// ---------------------------------------------------------------------------

fn btrfs_mp(dev: &str) -> Option<String> {
    let out = Command::new("lsblk")
        .args(["-no", "MOUNTPOINT", dev])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn btrfs_create(dev: &str, name: Option<&str>) {
    let mp = match btrfs_mp(dev) {
        Some(m) => m,
        None => {
            eprintln!("The Btrfs volume is not mounted. Mount it first.");
            return;
        }
    };

    let snap_name = name.map(str::to_string).unwrap_or_else(timestamp_name);
    let snap_dir = format!("{}/.snapshots", mp);
    let snap_path = format!("{}/{}", snap_dir, snap_name);

    if let Err(e) = std::fs::create_dir_all(&snap_dir) {
        eprintln!("Could not create snapshots directory: {}", e);
        return;
    }

    println!("Creating Btrfs snapshot at {}...", snap_path);

    match Command::new("btrfs")
        .args(["subvolume", "snapshot", &mp, &snap_path])
        .status()
    {
        Ok(s) if s.success() => println!("Snapshot created: {}", snap_path),
        Ok(_) => eprintln!("btrfs subvolume snapshot failed."),
        Err(e) => eprintln!("Failed to run btrfs: {}", e),
    }
}

fn btrfs_list(dev: &str) {
    let mp = match btrfs_mp(dev) {
        Some(m) => m,
        None => {
            eprintln!("The Btrfs volume is not mounted.");
            return;
        }
    };

    println!("Btrfs snapshots for {} (mounted at {}):", dev, mp);
    let _ = Command::new("btrfs")
        .args(["subvolume", "list", "-s", &mp])
        .status();
}

fn btrfs_delete(dev: &str, name: &str) {
    let mp = match btrfs_mp(dev) {
        Some(m) => m,
        None => {
            eprintln!("The Btrfs volume is not mounted.");
            return;
        }
    };

    let snap_path = format!("{}/.snapshots/{}", mp, name);

    if !confirm(&format!("Delete Btrfs snapshot {}?", snap_path)) {
        println!("No change was made.");
        return;
    }

    match Command::new("btrfs")
        .args(["subvolume", "delete", &snap_path])
        .status()
    {
        Ok(s) if s.success() => println!("Snapshot {} deleted.", snap_path),
        Ok(_) => eprintln!("btrfs subvolume delete failed."),
        Err(e) => eprintln!("Failed to run btrfs: {}", e),
    }
}

fn btrfs_restore(dev: &str, name: &str) {
    let mp = match btrfs_mp(dev) {
        Some(m) => m,
        None => {
            eprintln!("The Btrfs volume is not mounted.");
            return;
        }
    };

    let snap_path = format!("{}/.snapshots/{}", mp, name);

    println!("Restore from Btrfs snapshot: {}", snap_path);
    println!();
    println!(
        "  Btrfs restore requires setting the default subvolume and \
         remounting. Steps:"
    );
    println!("  1. Identify the snapshot's subvolume ID:");
    println!("       btrfs subvolume list {}", mp);
    println!("  2. Set it as default:");
    println!("       btrfs subvolume set-default <ID> {}", mp);
    println!("  3. Reboot or remount the volume.");
    println!();
    println!("  The snapshot is located at: {}", snap_path);
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

fn snap_create(dev: &str, backend: &Backend, name: Option<&str>) {
    match backend {
        Backend::Lvm => lvm_create(dev, name),
        Backend::Btrfs => btrfs_create(dev, name),
        Backend::Unsupported(f) => unsupported_message(f),
    }
}

fn snap_list(dev: &str, backend: &Backend) {
    match backend {
        Backend::Lvm => lvm_list(dev),
        Backend::Btrfs => btrfs_list(dev),
        Backend::Unsupported(f) => unsupported_message(f),
    }
}

fn snap_delete(dev: &str, backend: &Backend, name: &str) {
    match backend {
        Backend::Lvm => lvm_delete(dev, name),
        Backend::Btrfs => btrfs_delete(dev, name),
        Backend::Unsupported(f) => unsupported_message(f),
    }
}

fn snap_restore(dev: &str, backend: &Backend, name: &str) {
    match backend {
        Backend::Lvm => lvm_restore(dev, name),
        Backend::Btrfs => btrfs_restore(dev, name),
        Backend::Unsupported(f) => unsupported_message(f),
    }
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

fn print_usage() {
    println!("Syntax:  snapshot create  [name=<label>]");
    println!("         snapshot list");
    println!("         snapshot delete  <name>");
    println!("         snapshot restore <name>");
    println!();
    println!("Supported backends: LVM logical volumes, Btrfs subvolumes.");
}
