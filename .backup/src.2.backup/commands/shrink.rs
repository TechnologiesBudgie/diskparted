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
/// Example: shrink volume 20G
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

    shrink_volume(size_arg, ctx);
}

fn shrink_volume(size_arg: &str, ctx: &mut Context) {
    let partition = ctx.selected_partition.as_ref().unwrap();
    let part_path = &partition.path;

    if !Path::new(part_path).exists() {
        println!("Partition {} does not exist.", part_path);
        return;
    }

    let fs_type = detect_filesystem(part_path);

    match fs_type.as_deref() {
        Some("ext4") | Some("ext3") | Some("ext2") => {
            if !silent_unmount(part_path) {
                println!("Failed to unmount {}. Is it in use?", part_path);
                return;
            }

            println!("Warning: Shrinking ext filesystem can cause data loss.");
            if !confirm("Proceed? (y/N): ") {
                println!("Aborted.");
                return;
            }

            // Recommended: filesystem check before shrinking
            let _ = Command::new("e2fsck")
                .args(["-f", "-y", part_path])
                .status();

            let status = Command::new("resize2fs")
                .args([part_path, size_arg])
                .status();

            match status {
                Ok(s) if s.success() => println!("Filesystem shrunk successfully."),
                Ok(_) => println!("Failed to shrink filesystem."),
                Err(e) => println!("Error executing resize2fs: {}", e),
            }
        }

        Some("ntfs") => {
            if !silent_unmount(part_path) {
                println!("Failed to unmount {}. Is it in use?", part_path);
                return;
            }

            println!("Warning: Shrinking NTFS can cause data loss.");
            if !confirm("Proceed? (y/N): ") {
                println!("Aborted.");
                return;
            }

            // Check filesystem first
            let info = Command::new("ntfsresize")
                .args(["--info", part_path])
                .status();

            if info.is_err() || !info.unwrap().success() {
                println!("NTFS check failed. Aborting.");
                return;
            }

            let status = Command::new("ntfsresize")
                .args(["--size", size_arg, part_path])
                .status();

            match status {
                Ok(s) if s.success() => println!("NTFS filesystem shrunk successfully."),
                Ok(_) => println!("Failed to shrink NTFS filesystem."),
                Err(e) => println!("Error executing ntfsresize: {}", e),
            }
        }

        Some(other) => {
            println!(
                "Filesystem type {} not supported for automatic shrinking.",
                other
            );
        }

        None => {
            println!("Unable to detect filesystem type.");
        }
    }
}

/// Detect filesystem type using lsblk
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

/// Silent unmount helper (no spam if not mounted)
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

/// Simple yes/no confirmation
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
