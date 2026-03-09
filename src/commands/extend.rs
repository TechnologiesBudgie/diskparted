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

//! EXTEND
//!
//! Extends the volume or partition with focus and its file system into free
//! (unallocated) space on a disk.
//!
//! Syntax
//! ------
//!   extend [size=<n>] [disk=<n>] [noerr]
//!   extend filesystem [noerr]
//!
//! Parameters
//! ----------
//!   size=<n>     Amount of space in megabytes (MB) to add to the current
//!                volume or partition. If no size is given, all of the
//!                contiguous free space available on the disk is used.
//!
//!   disk=<n>     Accepted for DiskPart script compatibility. On Linux the
//!                partition's disk is used directly; this value is noted but
//!                not used to re-target the operation.
//!
//!   filesystem   Extends the file system of the volume with focus. For use
//!                only on disks where the file system was not extended with
//!                the volume (i.e. the partition was already grown externally).
//!
//!   noerr        For scripting only. When an error is encountered, DiskPart
//!                continues to process commands as if the error did not occur.
//!                Without this parameter, an error causes DiskPart to exit
//!                with an error code.
//!
//! Remarks
//! -------
//!   - On basic disks, the free space must be on the same disk as the volume
//!     or partition with focus and must immediately follow it.
//!   - If the partition was previously formatted with the NTFS file system,
//!     the file system is automatically extended to fill the larger partition
//!     and no data loss will occur.
//!   - If the partition was previously formatted with a file system other than
//!     NTFS, the command fails with no change to the partition.
//!   - The partition must have an associated volume before it can be extended.
//!
//! Supported file systems (Linux)
//! --------------------------------
//!   ext2 / ext3 / ext4   resize2fs
//!   xfs                  xfs_growfs  (volume must be mounted)
//!   btrfs                btrfs filesystem resize max
//!   ntfs                 ntfsresize
//!
//! Reference
//! ---------
//!   https://learn.microsoft.com/en-us/windows-server/administration/
//!   windows-commands/extend

use std::process::Command;
use crate::context::Context;

pub fn run(args: &[&str], ctx: &mut Context) {
    let mut size_mb: Option<u64> = None;
    let mut filesystem_only = false;
    let mut noerr = false;

    for arg in args {
        let lower = arg.to_lowercase();
        match lower.as_str() {
            "filesystem" => filesystem_only = true,
            "noerr" => noerr = true,
            s if s.starts_with("size=") => {
                match s[5..].parse::<u64>() {
                    Ok(n) => size_mb = Some(n),
                    Err(_) => {
                        eprintln!("Invalid value for size: '{}'", &s[5..]);
                        if !noerr {
                            return;
                        }
                    }
                }
            }
            s if s.starts_with("disk=") => {
                // Accepted for script compatibility; ignored on Linux.
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

    // filesystem subcommand: grow FS only, partition already grown externally.
    if filesystem_only {
        println!("Extending the file system on {}...", part_dev);
        if extend_filesystem(&*part_dev, &fstype, noerr) {
            println!("DiskPart successfully extended the volume.");
        }
        return;
    }

    // Step 1 — grow the partition with parted.
    println!("Extending partition {} on {}...", part_dev, disk_dev);

    let new_end = match size_mb {
        Some(mb) => {
            let end = get_part_end_mib(&*disk_dev, part_num);
            format!("{}MiB", end + mb)
        }
        None => "100%".to_string(),
    };

    let status = Command::new("parted")
        .args(["-s", &*disk_dev, "resizepart", &part_num.to_string(), &new_end])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(_) => {
            eprintln!(
                "parted resizepart failed. \
                 Ensure there is unallocated space immediately after {}.",
                part_dev
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

    // Notify the kernel of the updated partition table.
    let _ = Command::new("partprobe").arg(&*disk_dev).status();

    // Step 2 — grow the file system.
    if fstype.is_empty() {
        println!("Partition extended. No file system detected — skipping FS resize.");
        println!("DiskPart successfully extended the volume.");
        return;
    }

    println!("Extending {} file system on {}...", fstype, part_dev);
    if extend_filesystem(&*part_dev, &fstype, noerr) {
        println!("DiskPart successfully extended the volume.");
    }
}

// ---------------------------------------------------------------------------
// File system resize
// ---------------------------------------------------------------------------

fn extend_filesystem(dev: &str, fstype: &str, noerr: bool) -> bool {
    match fstype {
        "ext2" | "ext3" | "ext4" => {
            // resize2fs without a target size fills the partition automatically.
            run_tool("resize2fs", &[dev], noerr)
        }

        "xfs" => {
            // xfs_growfs operates on the mount point, not the block device.
            let mp = match get_mountpoint(dev) {
                Some(m) if !m.is_empty() => m,
                _ => {
                    eprintln!(
                        "xfs_growfs requires the volume to be mounted. \
                         Mount {} and retry.",
                        dev
                    );
                    return false;
                }
            };
            run_tool("xfs_growfs", &[&mp], noerr)
        }

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
            run_tool("btrfs", &["filesystem", "resize", "max", &mp], noerr)
        }

        "ntfs" => run_tool("ntfsresize", &["--force", dev], noerr),

        "vfat" | "fat16" | "fat32" => {
            eprintln!(
                "The volume does not contain a recognized file system. \
                 FAT file systems cannot be extended on Linux."
            );
            if !noerr {
                return false;
            }
            false
        }

        other => {
            eprintln!(
                "The volume does not contain a recognized file system \
                 (detected: '{}').",
                other
            );
            eprintln!("Resize the file system manually after extending the partition.");
            if !noerr {
                return false;
            }
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn print_usage() {
    println!("Syntax:  extend [size=<n>] [disk=<n>] [noerr]");
    println!("         extend filesystem [noerr]");
    println!();
    println!("  size=<n>     MB to add (default: all contiguous free space)");
    println!("  disk=<n>     Disk number (compatibility; ignored on Linux)");
    println!("  filesystem   Extend file system only (partition already grown)");
    println!("  noerr        Continue on error (for scripting)");
}

/// Extract the trailing decimal partition number from a device path.
/// "/dev/sda3" -> 3, "/dev/nvme0n1p2" -> 2
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

/// Return the current end position (in MiB) of a partition using parted.
fn get_part_end_mib(disk: &str, part_num: u32) -> u64 {
    let out = Command::new("parted")
        .args(["-s", "-m", disk, "unit", "MiB", "print"])
        .output();

    if let Ok(o) = out {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            let cols: Vec<&str> = line.split(':').collect();
            if cols.len() >= 3 {
                if cols[0].trim().parse::<u32>().ok() == Some(part_num) {
                    let end = cols[2].trim().trim_end_matches("MiB");
                    return end.parse().unwrap_or(0);
                }
            }
        }
    }
    0
}

/// Detect the file system type on a block device via lsblk.
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

/// Return the mount point of a block device via lsblk, if any.
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

/// Run an external tool and report success/failure uniformly.
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
