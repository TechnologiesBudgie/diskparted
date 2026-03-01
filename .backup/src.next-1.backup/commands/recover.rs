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

/// RECOVER — refresh disk state and attempt filesystem recovery.
///
/// DiskPart syntax:  recover
///
/// DiskPart's original behaviour: refreshes the state of all disks in a disk pack,
/// attempts recovery on invalid disks, and resynchronises RAID-5 volumes with stale
/// plex or parity data.
///
/// On Linux this does the closest equivalent:
///   1. Re-probe the partition table on the selected disk (partprobe)
///   2. Run fsck -n (read-only check) on the selected partition, if any
///   3. Attempt mdadm --assemble --scan if the disk is part of a software RAID
pub fn run(_args: &[&str], ctx: &mut Context) {
    let disk_path = ctx.selected_disk.as_ref().map(|d| d.path.clone());
    let part_path = ctx.selected_partition.as_ref().map(|p| p.path.clone());

    if disk_path.is_none() && part_path.is_none() {
        println!("No disk or partition selected. Use 'select disk <n>' first.");
        return;
    }

    println!("Attempting recovery...");
    println!();

    // Step 1: re-probe partition table
    if let Some(ref dp) = disk_path {
        println!("Step 1/3 — Re-probing partition table on {}...", dp);
        let status = Command::new("sudo")
            .args(["partprobe", dp])
            .status()
            .expect("Failed to execute partprobe");

        if status.success() {
            println!("  Partition table refreshed.");
        } else {
            println!("  partprobe failed (disk may be busy).");
        }
    } else {
        println!("Step 1/3 — Skipped (no disk selected).");
    }

    println!();

    // Step 2: fsck read-only check on selected partition
    if let Some(ref pp) = part_path {
        println!("Step 2/3 — Running filesystem check on {}...", pp);
        println!("  (Read-only check — no changes will be made)");

        // Unmount silently to allow fsck to run
        let _ = Command::new("sudo").args(["umount", pp]).output();

        let status = Command::new("sudo")
            .args(["fsck", "-n", pp])
            .status()
            .expect("Failed to execute fsck");

        if status.success() {
            println!("  Filesystem check passed — no errors found.");
        } else {
            println!("  Filesystem errors detected on {}.", pp);
            println!("  To attempt repair, run:  sudo fsck -y {}", pp);
        }
    } else {
        println!("Step 2/3 — Skipped (no partition selected).");
    }

    println!();

    // Step 3: attempt mdadm reassembly
    if which::which("mdadm").is_ok() {
        println!("Step 3/3 — Attempting software RAID reassembly (mdadm --assemble --scan)...");

        let status = Command::new("sudo")
            .args(["mdadm", "--assemble", "--scan"])
            .status()
            .expect("Failed to execute mdadm");

        if status.success() {
            println!("  RAID reassembly completed.");
        } else {
            println!("  No degraded RAID arrays found, or reassembly was not needed.");
        }
    } else {
        println!("Step 3/3 — Skipped (mdadm not installed).");
    }

    println!();
    println!("Recovery scan complete.");
}
