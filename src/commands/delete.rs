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
use std::path::Path;

use crate::utils;
use crate::context::Context;

/// Run the DELETE command.
///
/// Syntax (mirrors DiskPart):
///   delete partition [override] [noerr]
///   delete volume    [noerr]
///   delete disk      [noerr] [override]   (stub — dynamic disks not supported on Linux)
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        print_usage();
        return;
    }

    let flags = &args[1..];
    let override_flag = flags.iter().any(|a| a.eq_ignore_ascii_case("override"));
    let noerr_flag    = flags.iter().any(|a| a.eq_ignore_ascii_case("noerr"));

    // Warn about unrecognised flags
    for flag in flags {
        if !flag.eq_ignore_ascii_case("override") && !flag.eq_ignore_ascii_case("noerr") {
            println!("Unknown parameter: '{}'. Ignoring.", flag);
        }
    }

    match args[0].to_lowercase().as_str() {
        "partition" => delete_partition(ctx, override_flag, noerr_flag),
        "volume"    => delete_volume(ctx, noerr_flag),
        "disk"      => delete_disk(),
        _ => {
            println!("Unknown delete target: '{}'.", args[0]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  delete partition [override] [noerr]");
    println!("  delete volume    [noerr]");
    println!("  delete disk      [noerr] [override]");
}

// ---------------------------------------------------------------------------
// DELETE PARTITION
// ---------------------------------------------------------------------------

/// Detect whether a partition looks like a protected system/EFI/MSR/recovery
/// partition that DiskPart would refuse to delete without `override`.
/// Uses `parted -s <disk> print` and checks the partition flags column.
fn is_protected_partition(disk_path: &str, part_index: u32) -> bool {
    let output = match Command::new("sudo")
        .args(["parted", "-s", disk_path, "print"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };

    let text = String::from_utf8_lossy(&output.stdout);

    // Each data line in `parted print` looks like:
    //   1      1049kB  538MB   537MB   fat32        EFI System Partition  boot, esp
    // The partition number is the first token; flags are at the end.
    for line in text.lines() {
        let trimmed = line.trim();
        let mut cols = trimmed.split_whitespace();
        if let Some(num_str) = cols.next() {
            if num_str.parse::<u32>().ok() != Some(part_index) {
                continue;
            }
            let flags_text = line.to_lowercase();
            if flags_text.contains("esp")
                || flags_text.contains("boot")
                || flags_text.contains("msftres")
                || flags_text.contains("hidden")
                || flags_text.contains("diag")
            {
                return true;
            }
        }
    }
    false
}

fn delete_partition(ctx: &mut Context, override_flag: bool, noerr: bool) {
    let partition = match ctx.selected_partition.clone() {
        Some(p) => p,
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    let disk_path = match ctx.selected_disk.as_ref().map(|d| d.path.clone()) {
        Some(p) => p,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    if !Path::new(&partition.path).exists() {
        println!("Partition {} no longer exists.", partition.path);
        return;
    }

    // Guard: refuse to delete protected partitions without `override`
    if !override_flag && is_protected_partition(&disk_path, partition.index) {
        println!(
            "Cannot delete a protected partition without the override parameter.\n\
             To delete a system, EFI, or recovery partition use:  delete partition override"
        );
        return;
    }

    println!(
        "WARNING: You are about to delete partition {} on {}!",
        partition.path, disk_path
    );
    println!("This operation is irreversible.");

    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // Attempt to unmount silently before deleting
    let _ = Command::new("sudo").args(["umount", &partition.path]).output();

    let status = Command::new("sudo")
        .args(["parted", &disk_path, "--script", "rm", &partition.index.to_string()])
        .status()
        .expect("Failed to execute parted");

    if status.success() {
        println!("Partition {} deleted successfully.", partition.path);
        ctx.selected_partition = None;
    } else {
        println!("Failed to delete partition {}.", partition.path);
        if noerr {
            println!("(noerr: continuing despite error)");
        }
    }
}

// ---------------------------------------------------------------------------
// DELETE VOLUME
// ---------------------------------------------------------------------------

/// Deletes the currently selected volume (represented as the selected partition).
///
/// `noerr`  — on failure, report the error but keep going (mirrors DiskPart scripting mode).
/// No `override` flag — this matches DiskPart's spec for `delete volume`.
fn delete_volume(ctx: &mut Context, noerr: bool) {
    let partition = match ctx.selected_partition.clone() {
        Some(p) => p,
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    if !Path::new(&partition.path).exists() {
        println!("Volume {} no longer exists.", partition.path);
        return;
    }

    // DiskPart refuses to delete the system/boot volume or active paging file
    if is_active_system_volume(&partition.path) {
        println!(
            "Cannot delete the system volume, boot volume, or a volume containing \
             an active paging file or crash dump."
        );
        return;
    }

    let disk_path = match ctx.selected_disk.as_ref().map(|d| d.path.clone()) {
        Some(p) => p,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    println!(
        "WARNING: You are about to delete volume {} on {}!",
        partition.path, disk_path
    );
    println!("This operation is irreversible.");

    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // Unmount the volume first, then wipe its filesystem signature.
    // This leaves the partition entry on disk intact but unformatted —
    // matching DiskPart's behaviour where `delete volume` destroys the
    // filesystem data rather than removing the partition table entry.
    let _ = Command::new("sudo").args(["umount", &partition.path]).output();

    let wipe = Command::new("sudo")
        .args(["wipefs", "-a", &partition.path])
        .status()
        .expect("Failed to execute wipefs");

    if wipe.success() {
        println!("Volume {} deleted successfully.", partition.path);
        ctx.selected_partition = None;
    } else {
        println!("Failed to delete volume {}.", partition.path);
        if noerr {
            println!("(noerr: continuing despite error)");
        }
    }
}

/// Returns true if the partition is actively in use as a system mountpoint or swap.
fn is_active_system_volume(part_path: &str) -> bool {
    // Check /proc/mounts for system-critical mountpoints
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let mut cols = line.split_whitespace();
            let dev = cols.next().unwrap_or("");
            let mnt = cols.next().unwrap_or("");
            if dev == part_path && matches!(mnt, "/" | "/boot" | "/boot/efi") {
                return true;
            }
        }
    }

    // Check /proc/swaps for active swap partitions
    if let Ok(swaps) = std::fs::read_to_string("/proc/swaps") {
        for line in swaps.lines().skip(1) { // first line is the header
            if line.split_whitespace().next() == Some(part_path) {
                return true;
            }
        }
    }

    false
}

// ---------------------------------------------------------------------------
// DELETE DISK
// ---------------------------------------------------------------------------

/// DiskPart's `delete disk` removes a *missing* dynamic disk from the disk list.
/// Dynamic disks are a Windows-only concept and are not supported on Linux.
/// Direct users to `clean` for the equivalent operation.
fn delete_disk() {
    println!("The 'delete disk' command removes a missing dynamic disk from the disk list.");
    println!("Dynamic disks are not supported on Linux.");
    println!("To wipe all partition data from the selected disk, use the 'clean' command instead.");
}
