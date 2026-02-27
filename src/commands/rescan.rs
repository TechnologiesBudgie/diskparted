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

/// Run the RESCAN command
pub fn run(_args: &[&str], _ctx: &mut Context) {
    println!("Rescanning disks...");

    let status = Command::new("sudo")
        .arg("partprobe")  // update kernel partition table
        .status()
        .expect("Failed to execute partprobe");

    if status.success() {
        println!("Disk rescan completed successfully.");
    } else {
        println!("Disk rescan failed.");
    }
}
