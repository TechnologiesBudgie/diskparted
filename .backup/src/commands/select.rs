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
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Deserialize)]
struct LsblkDevice {
    name: String,
    size: String,
    #[serde(rename = "type")]
    devtype: String,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    children: Option<Vec<LsblkDevice>>,
}

/// Central entry point — routes `select disk`, `select partition`, `select volume`
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  select disk <number>");
        println!("  select partition <number>");
        println!("  select volume <number>");
        return;
    }

    match args[0].to_lowercase().as_str() {
        "disk"      => select_disk(&args[1..], ctx),
        "partition" => select_partition(&args[1..], ctx),
        "volume"    => select_volume(&args[1..], ctx),
        _ => {
            println!("Unknown select target: '{}'.", args[0]);
            println!("Usage: select disk|partition|volume <number>");
        }
    }
}

// -----------------------------------------------------------------------
// SELECT DISK
// -----------------------------------------------------------------------
fn select_disk(args: &[&str], ctx: &mut Context) {
    // No number = show current focus
    if args.is_empty() {
        match &ctx.selected_disk {
            Some(d) => println!("Disk {} is the current disk.", d.index),
            None    => println!("There is no disk selected."),
        }
        return;
    }

    let disk_index = match args[0].parse::<u32>() {
        Ok(n)  => n,
        Err(_) => { println!("Invalid disk number."); return; }
    };

    let disks = get_disks();
    match disks.into_iter().find(|d| d.index == disk_index) {
        Some(disk) => {
            println!("Disk {} is now the selected disk.", disk_index);
            // Switching disk clears partition selection (DiskPart behaviour)
            if ctx.selected_disk.as_ref().map(|d| d.index) != Some(disk_index) {
                ctx.selected_partition = None;
            }
            ctx.selected_disk = Some(disk);
        }
        None => println!("The specified disk is not valid."),
    }
}

// -----------------------------------------------------------------------
// SELECT PARTITION
// -----------------------------------------------------------------------
fn select_partition(args: &[&str], ctx: &mut Context) {
    // No number = show current focus
    if args.is_empty() {
        match &ctx.selected_partition {
            Some(p) => println!("Partition {} is the current partition.", p.index),
            None    => println!("There is no partition selected."),
        }
        return;
    }

    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    let part_index: u32 = match args[0].parse() {
        Ok(n)  => n,
        Err(_) => { println!("Invalid partition number."); return; }
    };

    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT"])
        .arg(&disk.path)
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse lsblk JSON");

    let disk_path = disk.path.clone();

    if let Some(device) = parsed.blockdevices.into_iter().next() {
        if let Some(children) = device.children {
            let parts: Vec<LsblkDevice> = children.into_iter()
                .filter(|p| p.devtype == "part")
                .collect();

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
                println!("Partition {} is now the selected partition.", part_index);

                // DiskPart: if partition has a corresponding volume, volume is also selected
                if part.mountpoint.is_some() {
                    println!("  * The corresponding volume is also selected.");
                }

                ctx.selected_partition = Some(Partition {
                    index: part_index,
                    name: part.name.clone(),
                    path: format!("/dev/{}", part.name),
                    size: part.size.clone(),
                });
                return;
            }
        }
    }

    println!("The specified partition is not valid.");
    let _ = disk_path; // suppress unused warning
}

// -----------------------------------------------------------------------
// SELECT VOLUME
// -----------------------------------------------------------------------
fn select_volume(args: &[&str], ctx: &mut Context) {
    // No number = show current focus
    if args.is_empty() {
        // We represent volumes via the selected partition
        match &ctx.selected_partition {
            Some(p) => println!("Volume {} (/dev/{}) is the current volume.", p.index, p.name),
            None    => println!("There is no volume selected."),
        }
        return;
    }

    let vol_index: usize = match args[0].parse() {
        Ok(n)  => n,
        Err(_) => { println!("Invalid volume number."); return; }
    };

    // Build the volume list (same logic as list_volumes)
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT"])
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse lsblk JSON");

    let mut volumes: Vec<(usize, String, String, String, Option<String>)> = Vec::new(); // (idx, disk_name, part_name, size, mountpoint)
    let mut idx = 0usize;

    for device in parsed.blockdevices.iter().filter(|d| d.devtype == "disk") {
        if let Some(children) = &device.children {
            for part in children.iter().filter(|p| p.devtype == "part") {
                volumes.push((idx, device.name.clone(), part.name.clone(), part.size.clone(), part.mountpoint.clone()));
                idx += 1;
            }
        }
    }

    match volumes.into_iter().find(|(i, _, _, _, _)| *i == vol_index) {
        Some((_, disk_name, part_name, size, _)) => {
            let part_path = format!("/dev/{}", part_name);
            let disk_path = format!("/dev/{}", disk_name);

            // Extract partition number suffix
            let part_num: u32 = part_name.chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>()
                .parse()
                .unwrap_or(0);

            println!("Volume {} is now the selected volume.", vol_index);

            // DiskPart: selecting a volume also selects the corresponding disk and partition
            let disks = get_disks();
            if let Some(disk) = disks.into_iter().find(|d| d.path == disk_path) {
                println!("  * Disk {} ({}) is also selected.", disk.index, disk_path);
                ctx.selected_disk = Some(disk);
            }

            println!("  * Partition {} ({}) is also selected.", part_num, part_path);
            ctx.selected_partition = Some(Partition {
                index: part_num,
                name: part_name,
                path: part_path,
                size,
            });
        }
        None => println!("The specified volume is not valid."),
    }
}
