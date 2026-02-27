/*
 * DiskParted - A Disk Management Tool
 * Copyright (C) 2026 Raphaël Larocque
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
use std::io::{self, Write};
use crate::context::Context;

pub fn run(_args: &[&str], ctx: &mut Context) {
    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    println!("WARNING: This will erase ALL partitions on {}!", disk.path);

    // Ask for Y/N confirmation
    if !confirm("Do you want to continue? (y/N): ") {
        println!("Operation cancelled.");
        return;
    }

    println!("Cleaning disk {}...", disk.path);

    // wipe filesystem signatures (safely handles MBR & GPT)
    let wipe_status = Command::new("wipefs")
        .args(&["-a", &disk.path])
        .status();

    if wipe_status.is_err() || !wipe_status.unwrap().success() {
        println!("Failed to wipe filesystem signatures.");
        return;
    }

    // remove partition table quietly
    let sgdisk_status = Command::new("sgdisk")
        .args(&["--zap-all", "--quiet", &disk.path])
        .status();

    match sgdisk_status {
        Ok(s) if s.success() => println!("Disk {} cleaned successfully.", disk.path),
        _ => println!("Failed to fully clean partition table (you may need root)."),
    }
}

/// Simple Y/N confirmation
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
