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
use crate::context::Context;
use crate::utils; // FIX: use shared utils::confirm instead of local duplicate

/// Supported filesystems
const SUPPORTED_FS: &[&str] = &[
    "fat", "fat16", "fat32", "vfat", "hfs+", "apfs", "ntfs",
    "ext1", "ext2", "ext3", "ext4", "xfs", "btrfs",
    "exfat", "bcachefs", "hfs", "f2fs", "jfs", "linux-swap",
    "minix", "nilfs2", "reiser4", "reiserfs", "udf", "zfs"
];

pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    if !Path::new(&partition.path).exists() {
        println!("Partition {} does not exist (maybe USB removed?).", partition.path);
        ctx.selected_partition = None;
        return;
    }

    let mut fs_type: Option<String> = None;
    let mut quick = false;
    let mut pool_name: Option<String> = None;

    for arg in args {
        if arg.starts_with("fs=") {
            fs_type = Some(arg.trim_start_matches("fs=").to_lowercase());
        } else if *arg == "quick" {
            quick = true;
        } else if arg.starts_with("name=") {
            pool_name = Some(arg.trim_start_matches("name=").to_string());
        }
    }

    let fs_type = match fs_type {
        Some(f) => f,
        None => {
            println!("Error: Filesystem not specified. Usage: format fs=<filesystem> [quick] [name=<pool>]");
            return;
        }
    };

    if !SUPPORTED_FS.contains(&fs_type.as_str()) {
        println!("Error: Unsupported filesystem '{}'.", fs_type);
        return;
    }

    println!("WARNING: You are about to format {} as {}{}.",
        partition.path, fs_type, if quick { " (quick)" } else { "" });

    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // Attempt to unmount silently
    let _ = Command::new("umount").arg(&partition.path).output();

    // ── ZFS is handled separately: it uses `zpool create`, not mkfs ─────────
    if fs_type == "zfs" {
        return format_zfs(&partition.path, pool_name.as_deref(), quick);
    }

    let mkfs_cmd = match fs_type.as_str() {
        "fat" | "fat16" => "mkfs.fat",
        "fat32" | "vfat" => "mkfs.fat",
        "ntfs" => "mkfs.ntfs",
        "ext1" => "mkfs.ext2",
        "ext2" => "mkfs.ext2",
        "ext3" => "mkfs.ext3",
        "ext4" => "mkfs.ext4",
        "xfs" => "mkfs.xfs",
        "btrfs" => "mkfs.btrfs",
        "exfat" => "mkfs.exfat",
        "f2fs" => "mkfs.f2fs",
        "jfs" => "mkfs.jfs",
        "linux-swap" => "mkswap",
        "udf" => "mkfs.udf",
        "hfs+" | "hfs" => "mkfs.hfsplus",
        "apfs" => "mkfs.apfs",
        "minix" => "mkfs.minix",
        "nilfs2" => "mkfs.nilfs2",
        "reiser4" | "reiserfs" => "mkfs.reiserfs",
        "bcachefs" => "mkfs.bcachefs",
        _ => { println!("Unsupported FS '{}'", fs_type); return; }
    };

    if which::which(mkfs_cmd).is_err() {
        println!("Error: '{}' not found. Install the corresponding filesystem package.", mkfs_cmd);
        return;
    }

    let mut cmd = Command::new(mkfs_cmd);
    cmd.arg(&partition.path);

    if quick {
        match fs_type.as_str() {
            "fat" | "fat16" => { cmd.arg("-F").arg("16"); }
            "fat32" | "vfat" => { cmd.arg("-F").arg("32"); }
            "ntfs" => { cmd.arg("-f"); }
            "ext1" | "ext2" | "ext3" | "ext4" => { cmd.arg("-F"); }
            "linux-swap" => { cmd.arg("-f"); }
            _ => {}
        }
    }

    match cmd.status() {
        Ok(s) if s.success() => println!("Partition {} formatted successfully.", partition.path),
        Ok(s) => println!("Failed to format {}. Exit code: {}", partition.path, s),
        Err(e) => println!("Failed to execute mkfs: {}", e),
    }
}

/// Create a ZFS pool on a single partition using `zpool create`.
///
/// ZFS does not use a traditional mkfs; instead a named pool is created that
/// owns the device.  The pool name defaults to the basename of the device
/// (e.g. "/dev/sdb1" → "sdb1") when not supplied with `name=<pool>`.
///
/// `quick` maps to `-f` (force) which suppresses the "in use" prompt that
/// `zpool` emits when the device previously held another filesystem.
fn format_zfs(dev_path: &str, pool_name: Option<&str>, force: bool) {
    if which::which("zpool").is_err() {
        println!("Error: 'zpool' not found. Install the 'zfsutils-linux' (Debian/Ubuntu) \
                  or 'zfs' (Fedora/Arch) package.");
        return;
    }

    // Derive a safe default pool name from the device basename.
    let default_name: String;
    let name = match pool_name {
        Some(n) if !n.is_empty() => n,
        _ => {
            default_name = Path::new(dev_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("pool0")
                .to_string();
            // Strip partition numbers to produce a cleaner name (sda1 → sda).
            let stripped = default_name.trim_end_matches(|c: char| c.is_ascii_digit());
            // But keep at least one character.
            if stripped.is_empty() { &default_name } else { stripped }
        }
    };

    // Validate pool name: ZFS pool names must start with a letter and contain
    // only alphanumerics, hyphens, underscores, colons, or spaces.
    if !name.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        println!("Error: ZFS pool name '{}' must start with a letter.", name);
        return;
    }

    let mut cmd = Command::new("zpool");
    cmd.arg("create");
    if force {
        cmd.arg("-f");
    }
    cmd.arg(name).arg(dev_path);

    println!("Creating ZFS pool '{}' on {}...", name, dev_path);

    match cmd.status() {
        Ok(s) if s.success() => {
            println!("ZFS pool '{}' created successfully on {}.", name, dev_path);
            println!("The pool is mounted at '/{}'.", name);
            println!("Use 'zpool status {}' to inspect it.", name);
        }
        Ok(s) => println!("zpool create failed for {}. Exit code: {}", dev_path, s),
        Err(e) => println!("Failed to execute zpool: {}", e),
    }
}
