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

//! SHRINK
//!
//! Reduces the size of the selected volume by the amount you specify. This
//! command makes free disk space available from the unused space at the end
//! of the volume.
//!
//! Syntax
//! ------
//!   shrink [desired=<n>] [minimum=<n>] [nowait] [noerr]
//!   shrink querymax [noerr]
//!
//! Parameters
//! ----------
//!   desired=<n>  Specifies the desired amount of space in megabytes (MB) to
//!                reduce the size of the volume by.
//!
//!   minimum=<n>  Specifies the minimum amount of space in MB to reduce the
//!                size of the volume by.
//!
//!   querymax     Returns the maximum amount of space in MB by which the
//!                volume can be reduced. This value may change if applications
//!                are currently accessing the volume.
//!
//!   nowait       Forces the command to return immediately while the shrink
//!                process is still in progress. Accepted for DiskPart script
//!                compatibility; on Linux all operations are synchronous so
//!                this flag has no effect.
//!
//!   noerr        For scripting only. When an error is encountered, DiskPart
//!                continues to process commands as if the error did not occur.
//!                Without this parameter, an error causes DiskPart to exit
//!                with an error code.
//!
//! Remarks
//! -------
//!   - You can reduce the size of a volume only if it is formatted using the
//!     NTFS file system or if it does not have a file system.
//!   - If a desired amount is not specified, the volume is reduced by the
//!     minimum amount (if specified).
//!   - If a minimum amount is not specified, the volume is reduced by the
//!     desired amount (if specified).
//!   - If neither a minimum nor a desired amount is specified, the volume is
//!     reduced by as much as possible.
//!   - If a minimum amount is specified but not enough free space is
//!     available, the command fails.
//!
//! Supported file systems (Linux)
//! --------------------------------
//!   ext2 / ext3 / ext4   resize2fs  (volume must be unmounted)
//!   ntfs                 ntfsresize (volume must be unmounted)
//!   btrfs                btrfs filesystem resize (can be online)
//!   xfs                  NOT supported — XFS cannot shrink
//!
//! Reference
//! ---------
//!   https://learn.microsoft.com/en-us/windows-server/administration/
//!   windows-commands/shrink

use std::process::Command;
use crate::context::Context;
use crate::utils::confirm;

pub fn run(args: &[&str], ctx: &mut Context) {
    let mut desired_mb: Option<u64> = None;
    let mut minimum_mb: Option<u64> = None;
    let mut querymax = false;
    let mut noerr = false;
    // nowait: accepted for DiskPart parity, unused on Linux.
    let mut _nowait = false;

    for arg in args {
        let lower = arg.to_lowercase();
        match lower.as_str() {
            "querymax" => querymax = true,
            "nowait" => _nowait = true,
            "noerr" => noerr = true,
            s if s.starts_with("desired=") => {
                match s[8..].parse::<u64>() {
                    Ok(n) => desired_mb = Some(n),
                    Err(_) => {
                        eprintln!("Invalid value for desired: '{}'", &s[8..]);
                        if !noerr {
                            return;
                        }
                    }
                }
            }
            s if s.starts_with("minimum=") => {
                match s[8..].parse::<u64>() {
                    Ok(n) => minimum_mb = Some(n),
                    Err(_) => {
                        eprintln!("Invalid value for minimum: '{}'", &s[8..]);
                        if !noerr {
                            return;
                        }
                    }
                }
            }
            _ => {
                eprintln!("Unknown parameter: '{}'", arg);
                if !noerr {
                    print_usage();
                    return;
                }
            }
        }
    }

    // Resolve selected partition and disk from context.
    let part_dev = match ctx.selected_partition.as_ref() {
        Some(p) => p.clone(),
        None => {
            eprintln!("There is no volume selected.");
            eprintln!("Select a volume with 'select volume <n>' and try again.");
            return;
        }
    };

    let disk_dev = match ctx.selected_disk.as_ref() {
        Some(d) => d.clone(),
        None => {
            eprintln!("There is no disk selected.");
            return;
        }
    };

    let part_num = match parse_part_number(&*part_dev) {
        Some(n) => n,
        None => {
            eprintln!(
                "Could not determine partition number from '{}'. \
                 Select the volume again and retry.",
                part_dev
            );
            return;
        }
    };

    let fstype = detect_fstype(&*part_dev).unwrap_or_default();

    // XFS cannot shrink — refuse early with a clear message.
    if fstype == "xfs" {
        eprintln!("XFS volumes cannot be reduced in size.");
        eprintln!(
            "Back up your data, delete the partition, recreate it at the \
             desired size, format, and restore."
        );
        return;
    }

    // FAT cannot be shrunk with standard Linux tools.
    if matches!(fstype.as_str(), "vfat" | "fat16" | "fat32") {
        eprintln!("FAT file systems cannot be reduced in size on Linux.");
        return;
    }

    // querymax — report the maximum shrinkable space and exit.
    if querymax {
        println!("The maximum amount the selected volume can be reduced is:");
        match query_max_shrink_mb(&*part_dev, &fstype) {
            Some(mb) => println!("  {} MB", mb),
            None => eprintln!(
                "  Could not determine maximum shrink size for '{}' file system.",
                fstype
            ),
        }
        return;
    }

    // Determine how many MB to shrink by, following DiskPart precedence rules.
    let shrink_mb = match (desired_mb, minimum_mb) {
        // Neither specified — shrink as much as possible.
        (None, None) => match query_max_shrink_mb(&*part_dev, &fstype) {
            Some(m) => m,
            None => {
                eprintln!("Could not determine the maximum shrink size.");
                if !noerr {
                    return;
                }
                return;
            }
        },
        // Only desired — use it directly.
        (Some(d), None) => d,
        // Only minimum — use it directly.
        (None, Some(m)) => m,
        // Both — try desired, fall back to minimum if not enough free space.
        (Some(d), Some(m)) => match query_max_shrink_mb(&*part_dev, &fstype) {
            Some(max) if max >= d => d,
            Some(max) if max >= m => {
                println!(
                    "Not enough free space to shrink by {} MB. \
                     Shrinking by minimum {} MB instead.",
                    d, m
                );
                m
            }
            _ => {
                eprintln!(
                    "There is not enough space available on the volume to \
                     complete this operation."
                );
                if !noerr {
                    return;
                }
                return;
            }
        },
    };

    if shrink_mb == 0 {
        eprintln!("There is not enough space available on the volume to complete this operation.");
        return;
    }

    // Volumes other than btrfs must be unmounted.
    if fstype != "btrfs" {
        if let Some(mp) = get_mountpoint(&*part_dev) {
            if !mp.is_empty() {
                eprintln!(
                    "The volume {} is currently in use (mounted at '{}').",
                    part_dev, mp
                );
                eprintln!("Unmount it first:  umount {}", mp);
                if !noerr {
                    return;
                }
                return;
            }
        }
    }

    println!("Shrinking {} by {} MB...", part_dev, shrink_mb);
    println!();
    println!(
        "  WARNING: Shrinking a partition is a destructive operation. \
         Ensure you have a current backup before proceeding."
    );
    println!();

    if !confirm("Proceed with shrink?") {
        println!("No change was made.");
        return;
    }

    let current_size_mb = get_part_size_mib(&*disk_dev, part_num);
    let new_fs_mb = current_size_mb.saturating_sub(shrink_mb);

    // Step 1 — shrink the file system to the new target size.
    if !shrink_filesystem(&*part_dev, &fstype, new_fs_mb, noerr) && !noerr {
        return;
    }

    // Step 2 — shrink the partition with parted to match.
    let start_mb = get_part_start_mib(&*disk_dev, part_num);
    let new_end_mb = start_mb + new_fs_mb;

    let status = Command::new("parted")
        .args([
            "-s",
            &*disk_dev,
            "resizepart",
            &part_num.to_string(),
            &format!("{}MiB", new_end_mb),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            let _ = Command::new("partprobe").arg(&*disk_dev).status();
            println!(
                "DiskPart successfully shrunk the volume by:  {} MB",
                shrink_mb
            );
        }
        Ok(_) => {
            eprintln!("parted resizepart failed.");
            eprintln!(
                "The file system was already shrunk. \
                 Fix the partition table manually to avoid data loss."
            );
            if !noerr {
                return;
            }
        }
        Err(e) => {
            eprintln!("Failed to run parted: {}", e);
            if !noerr {
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// File system shrink
// ---------------------------------------------------------------------------

fn shrink_filesystem(dev: &str, fstype: &str, new_size_mb: u64, noerr: bool) -> bool {
    match fstype {
        "ext2" | "ext3" | "ext4" => {
            // e2fsck is mandatory before resize2fs will shrink.
            let _ = Command::new("e2fsck").args(["-f", "-y", dev]).status();
            run_tool("resize2fs", &[dev, &format!("{}M", new_size_mb)], noerr)
        }

        "ntfs" => run_tool(
            "ntfsresize",
            &["--force", &format!("--size={}M", new_size_mb), dev],
            noerr,
        ),

        "btrfs" => {
            let mp = match get_mountpoint(dev) {
                Some(m) if !m.is_empty() => m,
                _ => {
                    eprintln!(
                        "btrfs filesystem resize requires the volume to be mounted. \
                         Mount {} and retry.",
                        dev
                    );
                    return false;
                }
            };
            run_tool(
                "btrfs",
                &["filesystem", "resize", &format!("{}m", new_size_mb), &mp],
                noerr,
            )
        }

        // No file system — shrink partition only, nothing to do here.
        "" => true,

        other => {
            eprintln!(
                "Shrinking '{}' file systems is not supported. \
                 Only ext2/3/4, ntfs, and btrfs can be shrunk.",
                other
            );
            if !noerr {
                false
            } else {
                false
            }
        }
    }
}

// ---------------------------------------------------------------------------
// querymax helpers
// ---------------------------------------------------------------------------

fn query_max_shrink_mb(dev: &str, fstype: &str) -> Option<u64> {
    match fstype {
        "ext2" | "ext3" | "ext4" => {
            let out = Command::new("dumpe2fs").args(["-h", dev]).output().ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            let mut block_size: u64 = 4096;
            let mut free_blocks: u64 = 0;
            for line in text.lines() {
                if line.starts_with("Block size:") {
                    block_size = line.split_whitespace().last()?.parse().ok()?;
                }
                if line.starts_with("Free blocks:") {
                    free_blocks = line.split_whitespace().last()?.parse().ok()?;
                }
            }
            // Leave 10 % headroom to keep the file system healthy.
            let shrinkable = free_blocks * block_size * 9 / 10;
            Some(shrinkable / 1_048_576)
        }

        "ntfs" => {
            let out = Command::new("ntfsresize")
                .args(["--info", "--force", dev])
                .output()
                .ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("You might resize") {
                    if let Some(bytes_str) = line.split_whitespace().nth(4) {
                        let min_bytes: u64 = bytes_str.parse().ok()?;
                        let current = get_block_device_bytes(dev)?;
                        return Some(current.saturating_sub(min_bytes) / 1_048_576);
                    }
                }
            }
            None
        }

        "btrfs" => {
            let mp = get_mountpoint(dev)?;
            let out = Command::new("btrfs")
                .args(["filesystem", "usage", "-b", &mp])
                .output()
                .ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.trim_start().starts_with("Free (estimated):") {
                    // "   Free (estimated):           1234567890      (min: 9876543)"
                    let n: u64 = line
                        .split_whitespace()
                        .nth(2)?
                        .parse()
                        .ok()?;
                    return Some(n / 1_048_576);
                }
            }
            None
        }

        _ => None,
    }
}

fn get_block_device_bytes(dev: &str) -> Option<u64> {
    let out = Command::new("lsblk")
        .args(["-bno", "SIZE", dev])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .ok()
}

// ---------------------------------------------------------------------------
// Partition geometry helpers
// ---------------------------------------------------------------------------

fn parse_part_number(dev: &str) -> Option<u32> {
    let tail: String = dev
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    tail.parse().ok()
}

fn get_part_size_mib(disk: &str, part_num: u32) -> u64 {
    parted_col(disk, part_num, 3)
}

fn get_part_start_mib(disk: &str, part_num: u32) -> u64 {
    parted_col(disk, part_num, 1)
}

/// Read a field from `parted -m` colon-separated output.
/// Columns: num:start:end:size:fs:name:flags
fn parted_col(disk: &str, part_num: u32, col: usize) -> u64 {
    let out = Command::new("parted")
        .args(["-s", "-m", disk, "unit", "MiB", "print"])
        .output();
    if let Ok(o) = out {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            let cols: Vec<&str> = line.split(':').collect();
            if cols.len() > col {
                if cols[0].trim().parse::<u32>().ok() == Some(part_num) {
                    let val = cols[col].trim().trim_end_matches("MiB");
                    return val.parse().unwrap_or(0);
                }
            }
        }
    }
    0
}

fn detect_fstype(dev: &str) -> Option<String> {
    let out = Command::new("lsblk")
        .args(["-no", "FSTYPE", dev])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn get_mountpoint(dev: &str) -> Option<String> {
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

fn run_tool(tool: &str, args: &[&str], _noerr: bool) -> bool {
    match Command::new(tool).args(args).status() {
        Ok(s) if s.success() => true,
        Ok(s) => {
            eprintln!("{} exited with code {:?}.", tool, s.code());
            false
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("'{}' not found. Install the package that provides it.", tool);
            false
        }
        Err(e) => {
            eprintln!("Failed to launch {}: {}", tool, e);
            false
        }
    }
}

fn print_usage() {
    println!("Syntax:  shrink [desired=<n>] [minimum=<n>] [nowait] [noerr]");
    println!("         shrink querymax [noerr]");
    println!();
    println!("  desired=<n>  MB to shrink by (default: maximum available)");
    println!("  minimum=<n>  Minimum MB to shrink by (fallback if desired unavailable)");
    println!("  querymax     Show maximum shrinkable MB and exit");
    println!("  nowait       Accepted for compatibility; no effect on Linux");
    println!("  noerr        Continue on error (for scripting)");
}
