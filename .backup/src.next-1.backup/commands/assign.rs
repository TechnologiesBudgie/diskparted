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

/// ASSIGN — assign a mount point to the selected volume.
///
/// DiskPart syntax:
///   assign [letter=<d>] [mount=<path>] [noerr]
///
/// On Linux, drive letters don't exist. `letter=X` maps to /mnt/X.
/// `mount=<path>` mounts directly to the given path.
/// With no arguments, auto-mounts to /mnt/<partition-name>.
pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    let mut mount_path: Option<String> = None;
    let mut noerr = false;

    for arg in args {
        if let Some(letter) = arg.strip_prefix("letter=") {
            let letter = letter.trim_end_matches(':').to_lowercase();
            mount_path = Some(format!("/mnt/{}", letter));
        } else if let Some(path) = arg.strip_prefix("mount=") {
            mount_path = Some(path.to_string());
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    let target = mount_path.unwrap_or_else(|| format!("/mnt/{}", partition.name));

    if !Path::new(&target).exists() {
        let mkdir = Command::new("sudo")
            .args(["mkdir", "-p", &target])
            .status()
            .expect("Failed to execute mkdir");

        if !mkdir.success() {
            println!("Failed to create mount point: {}", target);
            if !noerr { return; }
        }
    }

    println!("Mounting {} at {}...", partition.path, target);

    let status = Command::new("sudo")
        .args(["mount", &partition.path, &target])
        .status()
        .expect("Failed to execute mount");

    if status.success() {
        println!("DiskPart successfully assigned the mount point.");
        println!("  Partition : {}", partition.path);
        println!("  Mount     : {}", target);
    } else {
        println!("Failed to mount {} at {}.", partition.path, target);
        println!("Hint: the filesystem may need to be specified manually (sudo mount -t <fstype> ...)");
        if noerr { println!("(noerr: continuing despite error)"); }
    }
}
