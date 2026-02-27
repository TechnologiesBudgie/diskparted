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

use crate::context::Context;
use std::process::Command;
use std::path::Path;
use std::io::{self, Write};

/// Usage: shrink volume <size>
/// Example: shrink volume 512M
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.len() < 2 || args[0].to_lowercase() != "volume" {
        println!("Usage:");
        println!("  shrink volume <size>");
        return;
    }

    let size_arg = args[1];

    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => {
            println!("No partition selected. Use `select partition <n>` first.");
            return;
        }
    };

    println!(
        "WARNING: You are about to shrink partition {} to {}.",
        partition.path, size_arg
    );

    if !confirm("Do you want to continue? (y/N): ") {
        println!("Aborted.");
        return;
    }

    if let Err(e) = shrink_volume(size_arg, ctx) {
        println!("Shrink failed: {}", e);
    } else {
        println!("Shrink operation completed successfully.");
    }
}

fn shrink_volume(size_arg: &str, ctx: &mut Context) -> Result<(), String> {
    let partition = ctx.selected_partition.as_ref().unwrap();
    let part_path = &partition.path;

    if !Path::new(part_path).exists() {
        return Err(format!("Partition {} does not exist.", part_path));
    }

    let fs_type = detect_filesystem(part_path);

    // Always unmount before shrinking
    if !silent_unmount(part_path) {
        return Err(format!("Failed to unmount {}. Is it in use?", part_path));
    }

    match fs_type.as_deref() {
        Some("ext4") | Some("ext3") | Some("ext2") => {
            println!("Warning: Shrinking ext filesystem can cause data loss.");
            if !confirm("Proceed? (y/N): ") {
                return Err("Aborted by user.".into());
            }

            let _ = Command::new("e2fsck")
                .args(["-f", "-y", part_path])
                .status();

            let status = Command::new("resize2fs")
                .args([part_path, size_arg])
                .status()
                .map_err(|e| e.to_string())?;

            if !status.success() {
                return Err("Filesystem shrink failed.".into());
            }

            println!("Filesystem shrunk successfully.");
        }

        Some("ntfs") => {
            println!("Warning: Shrinking NTFS can cause data loss.");
            if !confirm("Proceed? (y/N): ") {
                return Err("Aborted by user.".into());
            }

            // Check filesystem first
            let info_status = Command::new("ntfsresize")
                .args(["--info", part_path])
                .status()
                .map_err(|e| e.to_string())?;

            if !info_status.success() {
                return Err("NTFS check failed.".into());
            }

            // Shrink filesystem
            let resize_status = Command::new("ntfsresize")
                .args(["--size", size_arg, part_path])
                .status()
                .map_err(|e| e.to_string())?;

            if !resize_status.success() {
                return Err("NTFS shrink failed.".into());
            }

            println!("NTFS filesystem shrunk successfully.");
        }

        Some(other) => {
            return Err(format!(
                "Filesystem type {} not supported for automatic shrinking.",
                other
            ));
        }

        None => {
            return Err("Unable to detect filesystem type.".into());
        }
    }

    // Resize the partition table entry
    resize_partition_table(part_path, size_arg)?;

    // Refresh kernel partition table
    refresh_partition_table(part_path);

    Ok(())
}

/// Resize partition entry using parted
fn resize_partition_table(part_path: &str, new_size: &str) -> Result<(), String> {
    let (disk, part_number) = split_partition_path(part_path)?;

    println!("Resizing partition table entry...");

    let status = Command::new("parted")
        .args(["-s", &disk, "resizepart", &part_number, new_size])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        println!("Partition entry resized.");
        Ok(())
    } else {
        Err("parted resizepart failed".into())
    }
}

/// Extract disk path + partition number
/// Works for /dev/sda1, /dev/nvme0n1p1, /dev/mmcblk0p1
fn split_partition_path(part_path: &str) -> Result<(String, String), String> {
    let mut number = String::new();
    for c in part_path.chars().rev() {
        if c.is_ascii_digit() {
            number.insert(0, c);
        } else {
            break;
        }
    }

    if number.is_empty() {
        return Err("Unable to parse partition number".into());
    }

    let disk = if part_path.contains("nvme") || part_path.contains("mmcblk") {
        part_path.trim_end_matches(&format!("p{}", number)).to_string()
    } else {
        part_path.trim_end_matches(&number).to_string()
    };

    Ok((disk, number))
}

/// Reload partition table in kernel
fn refresh_partition_table(part_path: &str) {
    let (disk, _) = match split_partition_path(part_path) {
        Ok(v) => v,
        Err(_) => {
            println!("Could not determine parent disk.");
            return;
        }
    };

    println!("Refreshing partition table...");

    let result = Command::new("blockdev")
        .args(["--rereadpt", &disk])
        .status();

    if result.map(|s| s.success()).unwrap_or(false) {
        println!("Partition table reloaded.");
    } else {
        // fallback to partprobe
        let _ = Command::new("partprobe")
            .arg(&disk)
            .status();
        println!("Partition table refresh attempted.");
    }
}

fn detect_filesystem(part_path: &str) -> Option<String> {
    let output = Command::new("lsblk")
        .args(["-no", "FSTYPE", part_path])
        .output()
        .ok()?;

    let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fs.is_empty() {
        None
    } else {
        Some(fs)
    }
}

fn silent_unmount(part_path: &str) -> bool {
    let output = Command::new("umount")
        .arg(part_path)
        .output();

    if let Ok(out) = output {
        let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
        if out.status.success() || stderr.contains("not mounted") {
            return true;
        }
    }

    false
}

fn confirm(prompt: &str) -> bool {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    } else {
        false
    }
}
