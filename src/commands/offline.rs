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

/// OFFLINE — take the selected disk or volume offline.
///
/// DiskPart syntax:
///   offline disk   [noerr]
///   offline volume [noerr]
///
/// On Linux:
///   offline disk   — unmounts all volumes on the disk, then spins it down via hdparm -Y.
///   offline volume — unmounts the selected partition (same as `remove`).
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  offline disk   [noerr]");
        println!("  offline volume [noerr]");
        return;
    }

    let noerr = args.iter().any(|a| a.eq_ignore_ascii_case("noerr"));

    match args[0].to_lowercase().as_str() {
        "disk"   => offline_disk(ctx, noerr),
        "volume" => offline_volume(ctx, noerr),
        _ => {
            println!("Unknown offline target: '{}'. Use 'disk' or 'volume'.", args[0]);
        }
    }
}

fn offline_disk(ctx: &Context, noerr: bool) {
    let disk = match &ctx.selected_disk {
        Some(d) => d.clone(),
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    println!("Taking disk {} offline...", disk.path);

    // Unmount all partitions on this disk first
    let _ = Command::new("sudo")
        .args(["umount", "--all-targets", &disk.path])
        .output();

    // Spin down via hdparm
    let status = Command::new("sudo")
        .args(["hdparm", "-Y", &disk.path])
        .status()
        .expect("Failed to execute hdparm");

    if status.success() {
        println!("Disk {} has been taken offline (spun down).", disk.path);
    } else {
        // hdparm may not support -Y on all disks (e.g. NVMe); that's acceptable
        println!("Disk {} unmounted. Spin-down may not be supported on this device.", disk.path);
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}

fn offline_volume(ctx: &Context, noerr: bool) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    println!("Taking volume {} offline (unmounting)...", partition.path);

    let status = Command::new("sudo")
        .args(["umount", &partition.path])
        .status()
        .expect("Failed to execute umount");

    if status.success() {
        println!("Volume {} is now offline.", partition.path);
    } else {
        println!("Failed to unmount {}. It may not be mounted.", partition.path);
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}
