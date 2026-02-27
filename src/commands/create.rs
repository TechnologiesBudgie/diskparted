/*
 * DiskParted - A Disk Management Tool
 * Copyright (C) 2026 Allan D.
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

pub fn run(args: &[&str], ctx: &mut Context) {
    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    if args.is_empty() || args[0] != "partition" {
        println!("Usage: create partition primary [size=<MB>]");
        return;
    }

    // Default size is full disk
    let mut size_arg = "100%".to_string();

    // Parse optional size argument: create partition primary size=500
    for arg in args.iter().skip(2) { // skip "partition primary"
        if arg.starts_with("size=") {
            size_arg = arg.trim_start_matches("size=").to_string();
        }
    }

    println!(
        "Creating primary partition on {} with size {}...",
        disk.path, size_arg
    );

    // Check for partition table (capture both stdout and stderr)
    let label_check = Command::new("parted")
        .args(&["-s", &disk.path, "print"])
        .output()
        .expect("Failed to run parted to check disk label");

    let stdout = String::from_utf8_lossy(&label_check.stdout);
    let stderr = String::from_utf8_lossy(&label_check.stderr);

    if stdout.contains("unrecognised disk label") || stderr.contains("unrecognised disk label") {
        // Create GPT label
        let status = Command::new("parted")
            .args(&["-s", &disk.path, "mklabel", "gpt"])
            .status()
            .expect("Failed to create GPT label");

        if status.success() {
            println!("Disk label created: GPT");
        } else {
            println!("Failed to create disk label. Make sure you have root privileges.");
            return;
        }
    }

    // Create the primary partition
    let status = Command::new("parted")
        .args(&["-s", &disk.path, "mkpart", "primary", "0%", &size_arg])
        .status()
        .expect("Failed to run parted mkpart");

    if status.success() {
        println!("Partition created successfully.");
    } else {
        println!("Failed to create partition. Make sure you have root privileges.");
    }
}
