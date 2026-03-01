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

/// ACTIVE — marks the selected partition as active (sets the boot/esp flag).
///
/// DiskPart syntax:  active
///
/// On GPT disks this sets the `esp` flag; on MBR disks it sets `boot`.
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

    println!("Marking partition {} on {} as active (flag: {})...", partition.index, disk_path, flag);

    let status = Command::new("sudo")
        .args(["parted", "-s", &disk_path, "set", &partition.index.to_string(), flag, "on"])
        .status()
        .expect("Failed to execute parted");

    if status.success() {
        println!("DiskPart successfully marked the current partition as active.");
        println!("Note: only sets the flag — does not verify the partition contains boot files.");
    } else {
        println!("Failed to mark partition as active.");
    }
}

pub fn get_disk_label(disk_path: &str) -> Option<String> {
    let output = Command::new("sudo")
        .args(["parted", "-s", disk_path, "print"])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|l| l.trim_start().starts_with("Partition Table:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_lowercase())
}
