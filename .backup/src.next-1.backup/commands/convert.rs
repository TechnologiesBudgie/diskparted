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
use crate::utils;

/// CONVERT — convert a disk between partition table formats, or upgrade ext filesystems.
///
/// DiskPart syntax:
///   convert gpt   [noerr]
///   convert mbr   [noerr]
///   convert basic          (stub — dynamic disks not on Linux)
///   convert dynamic        (stub — dynamic disks not on Linux)
///
/// Linux-only extensions (no DiskPart equivalent):
///   convert ext2-to-ext3   — upgrade ext2 filesystem on selected partition to ext3
///   convert ext3-to-ext4   — upgrade ext3 filesystem on selected partition to ext4
///   convert ext2-to-ext4   — upgrade ext2 filesystem on selected partition to ext4
///
/// WARNING: convert gpt/mbr requires the disk to have NO partitions (or uses
/// sgdisk's hybrid conversion).  Always back up data first.
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        print_usage();
        return;
    }

    let noerr = args.iter().any(|a| a.eq_ignore_ascii_case("noerr"));

    match args[0].to_lowercase().as_str() {
        "gpt"          => convert_to_gpt(ctx, noerr),
        "mbr"          => convert_to_mbr(ctx, noerr),
        "basic"        => println!("Dynamic-to-basic conversion is not supported on Linux (dynamic disks are a Windows-only concept)."),
        "dynamic"      => println!("Basic-to-dynamic conversion is not supported on Linux (dynamic disks are a Windows-only concept)."),
        "ext2-to-ext3" => convert_ext(ctx, "ext3", noerr),
        "ext3-to-ext4" => convert_ext(ctx, "ext4", noerr),
        "ext2-to-ext4" => convert_ext(ctx, "ext4", noerr),
        _ => {
            println!("Unknown convert target: '{}'.", args[0]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  convert gpt          [noerr]   — convert selected disk to GPT");
    println!("  convert mbr          [noerr]   — convert selected disk to MBR");
    println!("  convert ext2-to-ext3 [noerr]   — upgrade ext2 partition to ext3");
    println!("  convert ext3-to-ext4 [noerr]   — upgrade ext3 partition to ext4");
    println!("  convert ext2-to-ext4 [noerr]   — upgrade ext2 partition to ext4");
}

fn convert_to_gpt(ctx: &Context, noerr: bool) {
    let disk = match &ctx.selected_disk {
        Some(d) => d.clone(),
        None => { println!("No disk selected. Use 'select disk <n>' first."); return; }
    };

    println!("WARNING: Converting {} from MBR to GPT.", disk.path);
    println!("The disk must have no partitions, or data may be lost.");
    println!("sgdisk will attempt a non-destructive conversion if partitions exist,");
    println!("but this is NOT guaranteed to be safe. BACK UP YOUR DATA FIRST.");

    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // sgdisk -g converts MBR to GPT non-destructively (best effort)
    let status = Command::new("sudo")
        .args(["sgdisk", "-g", &disk.path])
        .status()
        .expect("Failed to execute sgdisk");

    if status.success() {
        println!("Disk {} converted to GPT successfully.", disk.path);
        println!("Run 'rescan' to refresh the partition table.");
    } else {
        println!("Conversion to GPT failed.");
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}

fn convert_to_mbr(ctx: &Context, noerr: bool) {
    let disk = match &ctx.selected_disk {
        Some(d) => d.clone(),
        None => { println!("No disk selected. Use 'select disk <n>' first."); return; }
    };

    println!("WARNING: Converting {} from GPT to MBR.", disk.path);
    println!("GPT supports more than 4 partitions and disks > 2TB; MBR does not.");
    println!("This conversion may be destructive. BACK UP YOUR DATA FIRST.");

    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // sgdisk -m converts GPT to MBR (hybrid/destructive)
    let status = Command::new("sudo")
        .args(["sgdisk", "-m", &disk.path])
        .status()
        .expect("Failed to execute sgdisk");

    if status.success() {
        println!("Disk {} converted to MBR successfully.", disk.path);
        println!("Run 'rescan' to refresh the partition table.");
    } else {
        println!("Conversion to MBR failed.");
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}

fn convert_ext(ctx: &Context, target_fs: &str, noerr: bool) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => { println!("No partition selected. Use 'select partition <n>' first."); return; }
    };

    // Detect current filesystem
    let current_fs = detect_fs(&partition.path);
    println!("Current filesystem on {}: {}", partition.path, current_fs.as_deref().unwrap_or("(unknown)"));

    println!("Upgrading {} to {}...", partition.path, target_fs);
    println!("This is an in-place upgrade and is generally safe, but back up data first.");

    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // Unmount first
    let _ = Command::new("sudo").args(["umount", &partition.path]).output();

    // tune2fs -O <features> upgrades the filesystem in-place
    let features = match target_fs {
        "ext3" => "has_journal",
        "ext4" => "extents,uninit_bg,dir_index,filetype,sparse_super,large_file",
        _      => { println!("Unknown target filesystem."); return; }
    };

    let status = Command::new("sudo")
        .args(["tune2fs", "-O", features, &partition.path])
        .status()
        .expect("Failed to execute tune2fs");

    if status.success() {
        // Run e2fsck to ensure consistency after the upgrade
        println!("Running fsck to ensure consistency...");
        let _ = Command::new("sudo")
            .args(["e2fsck", "-f", "-y", &partition.path])
            .status();

        println!("Filesystem on {} successfully upgraded to {}.", partition.path, target_fs);
    } else {
        println!("Failed to upgrade filesystem to {}.", target_fs);
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}

fn detect_fs(path: &str) -> Option<String> {
    let output = Command::new("lsblk")
        .args(["-n", "-o", "FSTYPE", path])
        .output()
        .ok()?;
    let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fs.is_empty() { None } else { Some(fs) }
}
