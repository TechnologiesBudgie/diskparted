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

use crate::utils;
use crate::context::Context;

/// Run the DELETE command.
/// Usage:
///   delete partition
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  delete partition");
        return;
    }

    match args[0] {
        "partition" => delete_partition(ctx),
        _ => println!("Unknown delete target."),
    }
}

/// Deletes the currently selected partition
fn delete_partition(ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    let disk_path = match &ctx.selected_disk {
        Some(d) => &d.path,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    if !Path::new(&partition.path).exists() {
        println!("Partition {} no longer exists.", partition.path);
        return;
    }

    println!("WARNING: You are about to delete partition {} on {}!", partition.path, disk_path);
    println!("This operation is irreversible.");

    // Use crate::utils directly
    if !utils::confirm("Do you want to continue?") {
        println!("Aborted.");
        return;
    }

    // Use parted to delete the partition
    let status = Command::new("sudo")
        .arg("parted")
        .arg(disk_path)
        .arg("--script")
        .arg("rm")
        .arg(&partition.index.to_string())
        .status()
        .expect("Failed to execute parted");

    if status.success() {
        println!("Partition {} deleted successfully.", partition.path);
        ctx.selected_partition = None; // clear selection
    } else {
        println!("Failed to delete partition {}.", partition.path);
    }
}
