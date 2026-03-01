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

/// REMOVE — remove the mount point from the selected volume.
///
/// DiskPart syntax:
///   remove [letter=<d>] [mount=<path>] [all] [dismount] [noerr]
///
/// On Linux this calls umount. `all` unmounts all known mountpoints for the
/// partition.  `dismount` is treated as an alias for `all` (forces unmount).
pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    let mut specific_path: Option<String> = None;
    let mut all = false;
    let mut dismount = false;
    let mut noerr = false;

    for arg in args {
        if let Some(letter) = arg.strip_prefix("letter=") {
            let letter = letter.trim_end_matches(':').to_lowercase();
            specific_path = Some(format!("/mnt/{}", letter));
        } else if let Some(path) = arg.strip_prefix("mount=") {
            specific_path = Some(path.to_string());
        } else if arg.eq_ignore_ascii_case("all") {
            all = true;
        } else if arg.eq_ignore_ascii_case("dismount") {
            dismount = true;
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    // dismount implies all
    if dismount { all = true; }

    if all || specific_path.is_none() {
        // Unmount by device path — catches all mountpoints for this partition
        println!("Unmounting all mountpoints for {}...", partition.path);

        let status = Command::new("sudo")
            .args(["umount", "--all-targets", &partition.path])
            .status()
            .expect("Failed to execute umount");

        if status.success() {
            println!("DiskPart successfully removed the drive letter or mount point.");
        } else {
            // Try plain umount as fallback (older util-linux may not have --all-targets)
            let fallback = Command::new("sudo")
                .args(["umount", &partition.path])
                .status()
                .expect("Failed to execute umount");

            if fallback.success() {
                println!("DiskPart successfully removed the drive letter or mount point.");
            } else {
                println!("Failed to unmount {}. It may not be mounted.", partition.path);
                if noerr { println!("(noerr: continuing despite error)"); }
            }
        }
    } else {
        let target = specific_path.unwrap();
        println!("Unmounting {} from {}...", partition.path, target);

        let status = Command::new("sudo")
            .args(["umount", &target])
            .status()
            .expect("Failed to execute umount");

        if status.success() {
            println!("DiskPart successfully removed the mount point '{}'.", target);
        } else {
            println!("Failed to unmount {}.", target);
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    }
}
