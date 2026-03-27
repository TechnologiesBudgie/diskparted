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

/// Filesystems supported for formatting (used both for display and by format.rs)
pub const SUPPORTED_FS: &[(&str, &str)] = &[
    ("ext4",       "Fourth Extended Filesystem (Linux default)"),
    ("ext3",       "Third Extended Filesystem"),
    ("ext2",       "Second Extended Filesystem"),
    ("fat32",      "FAT32 (Windows/cross-platform)"),
    ("fat16",      "FAT16"),
    ("exfat",      "exFAT (flash drives, large files)"),
    ("ntfs",       "NTFS (Windows)"),
    ("btrfs",      "B-Tree Filesystem (Linux, snapshots)"),
    ("xfs",        "XFS (high-performance Linux)"),
    ("f2fs",       "Flash-Friendly Filesystem"),
    ("jfs",        "Journaled Filesystem"),
    ("udf",        "Universal Disk Format (optical/cross-platform)"),
    ("hfs+",       "HFS+ (macOS, read-only on Linux)"),
    ("linux-swap", "Linux swap space"),
    ("minix",      "Minix Filesystem"),
    ("nilfs2",     "NILFS2 (log-structured)"),
    ("reiserfs",   "ReiserFS"),
    ("bcachefs",   "Bcachefs (modern Linux)"),
    ("zfs",        "ZFS (pooled storage, checksums, snapshots)"),
];

/// Run the FILESYSTEMS command.
/// Displays the current filesystem on the selected volume and lists supported formats.
pub fn run(_args: &[&str], ctx: &Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    // Detect current filesystem via lsblk -o FSTYPE
    let current_fs = detect_fs(&partition.path);

    println!("Current file system");
    println!();
    match &current_fs {
        Some(fs) if !fs.is_empty() => {
            println!("  Type         : {}", fs.to_uppercase());
            println!("  Partition    : {}", partition.path);
            println!("  Size         : {}", partition.size);
        }
        _ => {
            println!("  Type         : (none / unformatted)");
            println!("  Partition    : {}", partition.path);
            println!("  Size         : {}", partition.size);
        }
    }

    println!();
    println!("File systems supported for formatting:");
    println!();
    println!("  {:<14}  {:<6}  {}",  "Type", "Avail", "Description");
    println!("  {:<14}  {:<6}  {}",  "-".repeat(14), "-----", "-".repeat(40));

    for (fs_name, description) in SUPPORTED_FS {
        let available = if mkfs_available(fs_name) { "Yes" } else { "No" };
        let current_marker = match &current_fs {
            Some(cf) if cf.to_lowercase() == fs_name.to_lowercase() => " *",
            _ => "  ",
        };
        println!("  {:<14}  {:<6}  {}{}",
            fs_name, available, description, current_marker);
    }

    println!();
    println!("  * = current filesystem on this volume");
    println!("  Avail = mkfs tool present on this system");
}

/// Detect the current filesystem type on a partition using lsblk.
fn detect_fs(path: &str) -> Option<String> {
    let output = Command::new("lsblk")
        .args(["-n", "-o", "FSTYPE", path])
        .output()
        .ok()?;

    let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fs.is_empty() {
        None
    } else if fs.eq_ignore_ascii_case("zfs_member") {
        // lsblk reports ZFS vdevs as "zfs_member"; normalise to "zfs" for consistency.
        Some("zfs".to_string())
    } else {
        Some(fs)
    }
}

/// Check if the mkfs tool for a given filesystem is available on the system.
fn mkfs_available(fs: &str) -> bool {
    let cmd = match fs {
        "fat16" | "fat32" | "vfat" => "mkfs.fat",
        "ntfs"                      => "mkfs.ntfs",
        "ext2"                      => "mkfs.ext2",
        "ext3"                      => "mkfs.ext3",
        "ext4"                      => "mkfs.ext4",
        "xfs"                       => "mkfs.xfs",
        "btrfs"                     => "mkfs.btrfs",
        "exfat"                     => "mkfs.exfat",
        "f2fs"                      => "mkfs.f2fs",
        "jfs"                       => "mkfs.jfs",
        "linux-swap"                => "mkswap",
        "udf"                       => "mkfs.udf",
        "hfs+"                      => "mkfs.hfsplus",
        "minix"                     => "mkfs.minix",
        "nilfs2"                    => "mkfs.nilfs2",
        "reiserfs"                  => "mkfs.reiserfs",
        "bcachefs"                  => "mkfs.bcachefs",
        // ZFS uses `zpool` rather than an mkfs tool.
        "zfs"                       => "zpool",
        _                           => return false,
    };
    which::which(cmd).is_ok()
}
