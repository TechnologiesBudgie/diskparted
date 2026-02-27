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
    if args.is_empty() {
        println!("Usage:\n  shrink [volume <size>] | querymax");
        return;
    }

    match args[0].to_lowercase().as_str() {
        "volume" => {
            if args.len() < 2 {
                println!("Usage: shrink volume <size>");
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

        "querymax" => {
            let partition = match &ctx.selected_partition {
                Some(p) => p,
                None => {
                    println!("No partition selected. Use `select partition <n>` first.");
                    return;
                }
            };

            let fs_type = match detect_filesystem(&partition.path) {
                Some(f) => f,
                None => {
                    println!("Unable to detect filesystem type.");
                    return;
                }
            };

            match get_max_shrink(&partition.path, &fs_type) {
                Ok(bytes) => println!(
                    "Maximum shrinkable space: {} MB",
                    bytes / (1024 * 1024)
                ),
                Err(e) => println!("Failed to determine maximum shrink: {}", e),
            }
        }

        _ => {
            println!("Unknown argument. Usage:\n  shrink [volume <size>] | querymax");
        }
    }
}

fn shrink_volume(size_arg: &str, ctx: &mut Context) -> Result<(), String> {
    let partition = ctx.selected_partition.as_ref().unwrap();
    let part_path = &partition.path;

    if !Path::new(part_path).exists() {
        return Err(format!("Partition {} does not exist.", part_path));
    }

    let fs_type = detect_filesystem(part_path).ok_or("Unable to detect filesystem type.")?;

    // Unmount first
    if !silent_unmount(part_path) {
        return Err(format!("Failed to unmount {}. Is it in use?", part_path));
    }

    match fs_type.as_str() {
        "ext2" | "ext3" | "ext4" => {
            println!("Warning: Shrinking ext filesystem can cause data loss.");
            if !confirm("Proceed? (y/N): ") {
                return Err("Aborted by user.".into());
            }

            let _ = Command::new("e2fsck")
                .args(["-f", "-y", part_path])
                .status();

            let requested_bytes = parse_size_bytes(size_arg)?;
            let max_bytes = get_max_shrink(part_path, &fs_type)?;
            if requested_bytes > max_bytes {
                return Err(format!(
                    "Requested size too small; maximum safe shrink is {} MB",
                    max_bytes / (1024 * 1024)
                ));
            }

            // Convert bytes to blocks (assume 4 KB)
            let blocks = requested_bytes / 4096;

            let status = Command::new("resize2fs")
                .args([part_path, &format!("{}", blocks)])
                .status()
                .map_err(|e| e.to_string())?;

            if !status.success() {
                return Err("Filesystem shrink failed.".into());
            }

            println!("Filesystem shrunk successfully.");
        }

        "ntfs" => {
            println!("Warning: Shrinking NTFS can cause data loss.");
            if !confirm("Proceed? (y/N): ") {
                return Err("Aborted by user.".into());
            }

            let info_output = Command::new("ntfsresize")
                .args(["--info", part_path])
                .output()
                .map_err(|e| e.to_string())?;
            let info_str = String::from_utf8_lossy(&info_output.stdout);
            let max_shrink_bytes = parse_ntfs_max_shrink(&info_str)?;

            let requested_bytes = parse_size_bytes(size_arg)?;
            if requested_bytes > max_shrink_bytes {
                return Err(format!(
                    "Requested size {} is too small; maximum safe shrink is {} bytes.",
                    size_arg, max_shrink_bytes
                ));
            }

            let resize_status = Command::new("ntfsresize")
                .args(["--size", &format!("{}B", requested_bytes), part_path])
                .status()
                .map_err(|e| e.to_string())?;

            if !resize_status.success() {
                return Err("NTFS shrink failed.".into());
            }

            println!("NTFS filesystem shrunk successfully.");
        }

        other => return Err(format!("Filesystem type {} not supported.", other)),
    }

    resize_partition_table_with_buffer(part_path)?;
    refresh_partition_table(part_path);

    Ok(())
}

/// Get maximum shrinkable size in bytes for ext or NTFS
fn get_max_shrink(part_path: &str, fs_type: &str) -> Result<u64, String> {
    match fs_type {
        "ext2" | "ext3" | "ext4" => {
            let out = Command::new("resize2fs")
                .arg("-P")
                .arg(part_path)
                .output()
                .map_err(|e| e.to_string())?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("Estimated minimum size of the filesystem") {
                    if let Some(pos) = line.find("blocks") {
                        let num_str = line[..pos].trim().split_whitespace().last().unwrap();
                        if let Ok(num) = num_str.parse::<u64>() {
                            return Ok(num * 4096); // 4 KB blocks -> bytes
                        }
                    }
                }
            }
            Err("Failed to parse resize2fs output".into())
        }
        "ntfs" => {
            let out = Command::new("ntfsresize")
                .args(["--info", part_path])
                .output()
                .map_err(|e| e.to_string())?;
            parse_ntfs_max_shrink(&String::from_utf8_lossy(&out.stdout))
        }
        _ => Err(format!("Filesystem {} not supported", fs_type)),
    }
}

/// Resize partition entry using parted with small buffer (2 MB)
fn resize_partition_table_with_buffer(part_path: &str) -> Result<(), String> {
    let (disk, part_number) = split_partition_path(part_path)?;
    println!("Resizing partition table entry...");

    let parted_output = Command::new("parted")
        .args(["-m", &disk, "unit", "B", "print"])
        .output()
        .map_err(|e| e.to_string())?;
    let parted_str = String::from_utf8_lossy(&parted_output.stdout);

    let mut end_bytes = None;
    for line in parted_str.lines() {
        if line.starts_with(&part_number) {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 {
                let s = fields[2].trim_end_matches("B");
                if let Ok(n) = s.parse::<u64>() {
                    end_bytes = Some(n);
                }
            }
        }
    }

    let end_bytes = end_bytes.ok_or("Failed to read partition end from parted.")?;
    let new_end = end_bytes.saturating_add(2 * 1024 * 1024); // 2 MB buffer

    let status = Command::new("parted")
        .args(["-s", &disk, "resizepart", &part_number, &format!("{}", new_end)])
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
        let _ = Command::new("partprobe").arg(&disk).status();
        println!("Partition table refresh attempted.");
    }
}

fn detect_filesystem(part_path: &str) -> Option<String> {
    let output = Command::new("lsblk")
        .args(["-no", "FSTYPE", part_path])
        .output()
        .ok()?;
    let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fs.is_empty() { None } else { Some(fs) }
}

fn silent_unmount(part_path: &str) -> bool {
    let output = Command::new("umount").arg(part_path).output();
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

fn parse_size_bytes(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();
    if s.ends_with("G") {
        s[..s.len() - 1].parse::<u64>().map(|v| v * 1024 * 1024 * 1024).map_err(|e| e.to_string())
    } else if s.ends_with("M") {
        s[..s.len() - 1].parse::<u64>().map(|v| v * 1024 * 1024).map_err(|e| e.to_string())
    } else if s.ends_with("K") {
        s[..s.len() - 1].parse::<u64>().map(|v| v * 1024).map_err(|e| e.to_string())
    } else {
        s.parse::<u64>().map_err(|e| e.to_string())
    }
}

fn parse_ntfs_max_shrink(info: &str) -> Result<u64, String> {
    for line in info.lines() {
        if line.contains("You might resize at") || line.contains("minimum size") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for p in parts {
                if let Ok(val) = p.parse::<u64>() {
                    return Ok(val);
                }
            }
        }
    }
    Err("Could not parse maximum shrinkable NTFS size.".into())
}
