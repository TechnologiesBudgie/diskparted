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
use crate::commands::active::get_disk_label;

/// UNIQUEID — display or set the GPT GUID / MBR disk signature.
///
/// DiskPart syntax:
///   uniqueid disk [id={<dword> | <GUID>}] [noerr]
///
/// On Linux:
///   GPT — uses `sgdisk --disk-guid` (display) or `sgdisk --disk-guid=<GUID>` (set)
///   MBR — reads /proc/partitions + sfdisk for the 4-byte disk ID
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() || !args[0].eq_ignore_ascii_case("disk") {
        println!("Usage: uniqueid disk [id={{<dword>|<GUID>}}] [noerr]");
        return;
    }

    let disk = match &ctx.selected_disk {
        Some(d) => d.clone(),
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    let remaining = &args[1..];
    let mut new_id: Option<String> = None;
    let mut noerr = false;

    for arg in remaining {
        if let Some(val) = arg.strip_prefix("id=") {
            new_id = Some(val.to_string());
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    let label = get_disk_label(&disk.path);
    let is_gpt = label.as_deref() == Some("gpt");

    if let Some(ref id) = new_id {
        // --- SET ---
        if is_gpt {
            println!("Setting GPT disk GUID to {}...", id);
            let guid_arg = format!("--disk-guid={}", id);
            let status = Command::new("sudo")
                .args(["sgdisk", &guid_arg, &disk.path])
                .status()
                .expect("Failed to execute sgdisk");

            if status.success() {
                println!("Disk GUID updated successfully.");
            } else {
                println!("Failed to update disk GUID.");
                if noerr { println!("(noerr: continuing despite error)"); }
            }
        } else {
            // MBR: use sfdisk to set the 4-byte disk ID
            // sfdisk accepts the ID via --disk-id
            println!("Setting MBR disk signature to {}...", id);
            let status = Command::new("sudo")
                .args(["sfdisk", "--disk-id", &disk.path, id])
                .status()
                .expect("Failed to execute sfdisk");

            if status.success() {
                println!("Disk signature updated successfully.");
            } else {
                println!("Failed to update disk signature.");
                if noerr { println!("(noerr: continuing despite error)"); }
            }
        }
    } else {
        // --- DISPLAY ---
        if is_gpt {
            let output = Command::new("sudo")
                .args(["sgdisk", "--print-unique-guids", &disk.path])
                .output()
                .expect("Failed to execute sgdisk");

            // Fall back to --info if --print-unique-guids is not supported
            let text = String::from_utf8_lossy(&output.stdout);
            if text.trim().is_empty() {
                // Use sgdisk -p and grep for the disk GUID line
                let output2 = Command::new("sudo")
                    .args(["sgdisk", "-p", &disk.path])
                    .output()
                    .expect("Failed to execute sgdisk");
                let t2 = String::from_utf8_lossy(&output2.stdout);
                let guid_line = t2.lines()
                    .find(|l| l.to_lowercase().contains("disk identifier") || l.to_lowercase().contains("disk guid"))
                    .unwrap_or("(could not read GUID)");
                println!("Disk Identifier: {}", guid_line.trim());
            } else {
                for line in text.lines().filter(|l| l.to_lowercase().contains("disk guid") || l.contains(':')) {
                    println!("{}", line.trim());
                }
            }
        } else {
            // MBR: read disk ID via sfdisk
            let output = Command::new("sudo")
                .args(["sfdisk", "--disk-id", &disk.path])
                .output()
                .expect("Failed to execute sfdisk");

            let sig = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if sig.is_empty() {
                println!("Could not read MBR disk signature for {}.", disk.path);
            } else {
                println!("Disk Identifier (MBR signature): {}", sig);
            }
        }
    }
}
