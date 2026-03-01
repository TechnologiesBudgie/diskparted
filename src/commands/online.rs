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

/// ONLINE — bring the selected disk or volume to the online state.
///
/// DiskPart syntax:
///   online disk   [noerr]
///   online volume [noerr]
///
/// On Linux:
///   online disk   — wakes a spun-down disk with hdparm -S 0 (cancel standby)
///                   and re-probes partitions with partprobe.
///   online volume — mounts the selected partition (same as `assign`).
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  online disk   [noerr]");
        println!("  online volume [noerr]");
        return;
    }

    let noerr = args.iter().any(|a| a.eq_ignore_ascii_case("noerr"));

    match args[0].to_lowercase().as_str() {
        "disk"   => online_disk(ctx, noerr),
        "volume" => online_volume(ctx, noerr),
        _ => {
            println!("Unknown online target: '{}'. Use 'disk' or 'volume'.", args[0]);
        }
    }
}

fn online_disk(ctx: &Context, noerr: bool) {
    let disk = match &ctx.selected_disk {
        Some(d) => d.clone(),
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    println!("Bringing disk {} online...", disk.path);

    // Cancel standby / spin up
    let _ = Command::new("sudo")
        .args(["hdparm", "-S", "0", &disk.path])
        .output();

    // Re-probe partition table so kernel picks up any changes
    let status = Command::new("sudo")
        .args(["partprobe", &disk.path])
        .status()
        .expect("Failed to execute partprobe");

    if status.success() {
        println!("Disk {} is now online.", disk.path);
    } else {
        println!("Failed to bring disk {} online.", disk.path);
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}

fn online_volume(ctx: &Context, noerr: bool) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    // Check if already mounted
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        if mounts.lines().any(|l| l.split_whitespace().next() == Some(partition.path.as_str())) {
            println!("Volume {} is already mounted (online).", partition.path);
            return;
        }
    }

    let mount_point = format!("/mnt/{}", partition.name);
    println!("Bringing volume {} online (mounting at {})...", partition.path, mount_point);

    let _ = Command::new("sudo")
        .args(["mkdir", "-p", &mount_point])
        .status();

    let status = Command::new("sudo")
        .args(["mount", &partition.path, &mount_point])
        .status()
        .expect("Failed to execute mount");

    if status.success() {
        println!("Volume {} is now online at {}.", partition.path, mount_point);
    } else {
        println!("Failed to mount {}.", partition.path);
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}
