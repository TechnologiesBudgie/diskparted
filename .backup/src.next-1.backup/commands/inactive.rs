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

/// INACTIVE — marks the selected partition as inactive (clears the boot/esp flag).
///
/// DiskPart syntax:  inactive
pub fn run(_args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    let disk_path = match ctx.selected_disk.as_ref() {
        Some(d) => d.path.clone(),
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    let label = get_disk_label(&disk_path);
    let flag = match label.as_deref() {
        Some("gpt") => "esp",
        _           => "boot",
    };

    println!("Marking partition {} on {} as inactive (clearing flag: {})...", partition.index, disk_path, flag);

    let status = Command::new("sudo")
        .args(["parted", "-s", &disk_path, "set", &partition.index.to_string(), flag, "off"])
        .status()
        .expect("Failed to execute parted");

    if status.success() {
        println!("DiskPart successfully marked the current partition as inactive.");
    } else {
        println!("Failed to mark partition as inactive.");
    }
}
