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

/// DETAIL — display detailed properties of the selected disk, partition, or volume.
///
/// DiskPart syntax:
///   detail disk
///   detail partition
///   detail volume
pub fn run(args: &[&str], ctx: &Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  detail disk");
        println!("  detail partition");
        println!("  detail volume");
        return;
    }

    match args[0].to_lowercase().as_str() {
        "disk"      => detail_disk(ctx),
        "partition" => detail_partition(ctx),
        "volume"    => detail_volume(ctx),
        _ => println!("Unknown detail target: '{}'. Use disk, partition, or volume.", args[0]),
    }
}

fn detail_disk(ctx: &Context) {
    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => { println!("No disk selected. Use 'select disk <n>' first."); return; }
    };

    println!("Disk {}  ({})", disk.index, disk.path);
    println!();

    // Use parted for geometry and partition table type
    let parted_out = Command::new("sudo")
        .args(["parted", "-s", &disk.path, "print"])
        .output()
        .expect("Failed to execute parted");
    println!("{}", String::from_utf8_lossy(&parted_out.stdout).trim());

    println!();
    println!("Volumes on this disk:");

    // Use lsblk for volumes
    let lsblk_out = Command::new("lsblk")
        .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,UUID", &disk.path])
        .output()
        .expect("Failed to execute lsblk");
    println!("{}", String::from_utf8_lossy(&lsblk_out.stdout).trim());
}

fn detail_partition(ctx: &Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => { println!("No partition selected. Use 'select partition <n>' first."); return; }
    };

    let disk_path = match &ctx.selected_disk {
        Some(d) => d.path.as_str(),
        None => { println!("No disk selected. Use 'select disk <n>' first."); return; }
    };

    println!("Partition {}  ({})", partition.index, partition.path);
    println!();

    // parted print for the specific partition
    let parted_out = Command::new("sudo")
        .args(["parted", "-s", disk_path, "print"])
        .output()
        .expect("Failed to execute parted");

    let text = String::from_utf8_lossy(&parted_out.stdout);
    // Print only the header lines and the matching partition line
    for line in text.lines() {
        let is_header = !line.trim().starts_with(|c: char| c.is_ascii_digit());
        let is_our_part = line.trim().starts_with(&partition.index.to_string());
        if is_header || is_our_part {
            println!("{}", line);
        }
    }

    println!();

    // blkid for filesystem/UUID info
    let blkid_out = Command::new("sudo")
        .args(["blkid", "-o", "full", &partition.path])
        .output()
        .expect("Failed to execute blkid");
    let blkid_str = String::from_utf8_lossy(&blkid_out.stdout).trim().to_string();
    if !blkid_str.is_empty() {
        println!("Filesystem info:");
        println!("  {}", blkid_str);
    }

    // lsblk for mount status
    let lsblk_out = Command::new("lsblk")
        .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,UUID", &partition.path])
        .output()
        .expect("Failed to execute lsblk");
    println!();
    println!("{}", String::from_utf8_lossy(&lsblk_out.stdout).trim());
}

fn detail_volume(ctx: &Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => { println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first."); return; }
    };

    println!("Volume  ({})", partition.path);
    println!();

    // df for space usage if mounted
    let df_out = Command::new("df")
        .args(["-h", &partition.path])
        .output();

    if let Ok(o) = df_out {
        let df_str = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !df_str.is_empty() {
            println!("Usage:");
            println!("{}", df_str);
            println!();
        }
    }

    // blkid for filesystem type and UUID
    let blkid_out = Command::new("sudo")
        .args(["blkid", "-o", "full", &partition.path])
        .output()
        .expect("Failed to execute blkid");
    let blkid_str = String::from_utf8_lossy(&blkid_out.stdout).trim().to_string();
    if !blkid_str.is_empty() {
        println!("Filesystem info:");
        println!("  {}", blkid_str);
        println!();
    }

    // Full lsblk row
    let lsblk_out = Command::new("lsblk")
        .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,UUID,RO", &partition.path])
        .output()
        .expect("Failed to execute lsblk");
    println!("{}", String::from_utf8_lossy(&lsblk_out.stdout).trim());
}
