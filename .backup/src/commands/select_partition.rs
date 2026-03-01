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

    let part_index: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid partition number.");
            return;
        }
    };

    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE"])
        .arg(&disk.path)
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse lsblk JSON");

    if let Some(device) = parsed.blockdevices.into_iter().next() {
        if let Some(children) = device.children {
            // FIX: ends_with(&part_index.to_string()) is ambiguous — "sda1" would
            // match when asking for partition 1 even if "sda11" also exists.
            // Instead, collect all partitions in order and select by 1-based index,
            // which matches the numbers shown by `list partition`.
            let parts: Vec<Device> = children.into_iter()
                .filter(|p| p.devtype == "part")
                .collect();

            // part_index is 1-based to match what `list partition` displays
            let target = parts.into_iter().find(|p| {
                let suffix: String = p.name.chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                suffix.parse::<u32>().ok() == Some(part_index)
            });

            if let Some(part) = target {
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

    println!("Partition {} not found.", part_index);
}
