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
use crate::context::{Context, Partition};
use crate::commands::list::get_disks;
use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize)]
struct Lsblk {
    blockdevices: Vec<Device>,
}

#[derive(Deserialize)]
struct Device {
    name: String,
    size: String,
    #[serde(rename = "type")]
    devtype: String,
    #[serde(default)]
    children: Option<Vec<Device>>,
}

pub fn run(args: &[&str], ctx: &mut Context) {
    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    if args.len() != 2 || args[0] != "partition" {
        println!("Usage: select partition <number>");
        return;
    }

    let part_index = match args[1].parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid partition number.");
            return;
        }
    };

    // List partitions of the selected disk
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE"])
        .arg(&disk.path)
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse lsblk JSON");

    if let Some(device) = parsed.blockdevices.into_iter().next() {
        if let Some(children) = device.children {
            if let Some(part) = children.into_iter().find(|p| p.devtype == "part" && p.name.ends_with(&part_index.to_string())) {
                let partition = Partition {
                    index: part_index,
                    name: part.name.clone(),
                    path: format!("/dev/{}", part.name),
                    size: part.size.clone(),
                };
                println!("Partition {} selected: {}", part_index, partition.path);
                ctx.selected_partition = Some(partition);
                return;
            }
        }
    }

    println!("Partition not found.");
}
