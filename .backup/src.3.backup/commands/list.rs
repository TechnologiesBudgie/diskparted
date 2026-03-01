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

use serde::Deserialize;
use std::process::Command;
use std::path::Path;

use crate::context::{Context, Disk};

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
    tran: Option<String>,
    #[serde(default)]
    rm: Option<bool>,
    #[serde(default)]
    ro: Option<bool>,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    children: Option<Vec<Device>>,
}

pub fn run(args: &[&str], ctx: &Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  list disk");
        println!("  list partition");
        println!("  list volume");
        return;
    }

    match args[0] {
        "disk" => list_disks(),
        "partition" => list_partitions(ctx),
        "volume" => list_volumes(),
        _ => println!("Unknown list target."),
    }
}

// -------------------------
// DISKS
// -------------------------
pub fn get_disks() -> Vec<Disk> {
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,TRAN,RM,RO,MOUNTPOINT"])
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk =
        serde_json::from_slice(&output.stdout).expect("Failed to parse lsblk JSON");

    let mut disks = Vec::new();
    let mut index = 0;

    for device in parsed.blockdevices {
        if device.devtype == "disk" && device.tran.is_some() {
            disks.push(Disk {
                index,
                name: device.name.clone(),
                path: format!("/dev/{}", device.name),
                size: device.size.clone(),
            });
            index += 1;
        }
    }

    disks
}

fn pretty_tran(tran: Option<&str>) -> &'static str {
    match tran {
        Some("usb") => "USB",
        Some("sata") | Some("ata") => "SATA",
        Some("nvme") => "NVMe",
        Some("mmc") => "SD",
        Some(_) => "Other",
        None => "Unknown",
    }
}

fn disk_state(path: &str, size: &str) -> &'static str {
    if !Path::new(path).exists() {
        return "Removed";
    }
    if size == "0B" {
        return "PoweredOff";
    }
    "Online"
}

fn list_disks() {
    let disks = get_disks();

    println!("  Disk ###  Size     Path        Tran   Rem  RO   Status");
    println!("  --------  -------  ----------  -----  ---  ---  --------");

    for disk in disks.iter() {
        let state = disk_state(&disk.path, &disk.size);
        let tran = pretty_tran(Some("sata")); // placeholder if no tran
        let rm = "No";
        let ro = "No";

        println!(
            "  Disk {:<3}  {:<7}  {:<10}  {:<5}  {:<3}  {:<3}  {}",
            disk.index, disk.size, disk.path, tran, rm, ro, state
        );
    }
}

// -------------------------
// PARTITIONS
// -------------------------
fn list_partitions(ctx: &Context) {
    let selected = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected.");
            return;
        }
    };

    if !Path::new(&selected.path).exists() {
        println!("Selected disk {} is Removed.", selected.path);
        return;
    }

    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,RO,MOUNTPOINT"])
        .arg(&selected.path)
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk =
        serde_json::from_slice(&output.stdout).expect("Failed to parse lsblk JSON");

    println!("  Partition ###  Size     Path         RO   Mounted");
    println!("  -------------  -------  -----------  ---  --------");

    let mut found = false;

    for device in parsed.blockdevices {
        if let Some(children) = device.children {
            for part in children.iter().filter(|p| p.devtype == "part") {
                found = true;
                let index = extract_index(&part.name);
                let ro = if part.ro.unwrap_or(false) { "Yes" } else { "No" };
                let mounted = if part.mountpoint.is_some() { "Yes" } else { "No" };

                println!(
                    "  Partition {:<3}  {:<7}  /dev/{:<11}  {:<3}  {}",
                    index, part.size, part.name, ro, mounted
                );
            }
        }
    }

    if !found {
        println!("  (No partitions found on this disk)");
    }
}

// -------------------------
// VOLUMES (mounted filesystems)
// -------------------------
fn list_volumes() {
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,RO"])
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk =
        serde_json::from_slice(&output.stdout).expect("Failed to parse lsblk JSON");

    println!("  Volume ###  Size     Path         RO   Mounted");
    println!("  ----------  -------  -----------  ---  --------");

    let mut index = 0;

    for device in parsed.blockdevices.iter().filter(|d| d.devtype == "disk") {
        if let Some(children) = &device.children {
            for part in children.iter().filter(|p| p.devtype == "part") {
                let ro = if part.ro.unwrap_or(false) { "Yes" } else { "No" };
                let mounted = if part.mountpoint.is_some() { "Yes" } else { "No" };

                println!(
                    "  Volume {:<3}  {:<7}  /dev/{:<11}  {:<3}  {}",
                    index, part.size, part.name, ro, mounted
                );

                index += 1;
            }
        }
    }
}

// -------------------------
// HELPERS
// -------------------------
fn extract_index(name: &str) -> String {
    name.chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}
