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
    // FIX: call lsblk directly so we have access to tran/rm/ro per device,
    // instead of going through get_disks() which drops those fields.
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,TRAN,RM,RO,MOUNTPOINT"])
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk =
        serde_json::from_slice(&output.stdout).expect("Failed to parse lsblk JSON");

    println!("  Disk ###  Size     Path        Tran   Rem  RO   Status");
    println!("  --------  -------  ----------  -----  ---  ---  --------");

    let mut index = 0;

    for device in parsed.blockdevices {
        if device.devtype == "disk" && device.tran.is_some() {
            let path = format!("/dev/{}", device.name);
            let state = disk_state(&path, &device.size);
            let tran = pretty_tran(device.tran.as_deref()); // FIX: was hardcoded to Some("sata")
            let rm = if device.rm.unwrap_or(false) { "Yes" } else { "No" }; // FIX: was hardcoded "No"
            let ro = if device.ro.unwrap_or(false) { "Yes" } else { "No" }; // FIX: was hardcoded "No"

            println!(
                "  Disk {:<3}  {:<7}  {:<10}  {:<5}  {:<3}  {:<3}  {}",
                index, device.size, path, tran, rm, ro, state
            );

            index += 1;
        }
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

    // Collect rows first so we can measure the longest path for dynamic column width
    let mut rows: Vec<(String, String, String, &'static str, &'static str)> = Vec::new();
    for device in parsed.blockdevices {
        if let Some(children) = device.children {
            for part in children.iter().filter(|p| p.devtype == "part") {
                let index = extract_index(&part.name);
                let ro = if part.ro.unwrap_or(false) { "Yes" } else { "No" };
                let mounted = if part.mountpoint.is_some() { "Yes" } else { "No" };
                rows.push((index, part.size.clone(), part.name.clone(), ro, mounted));
            }
        }
    }

    if rows.is_empty() {
        println!("  (No partitions found on this disk)");
        return;
    }

    let path_w = rows.iter()
        .map(|(_, _, name, _, _)| "/dev/".len() + name.len())
        .max().unwrap()
        .max("Path".len());

    println!("  Partition ###  Size     {:<path_w$}  RO   Mounted", "Path", path_w = path_w);
    println!("  -------------  -------  {:<path_w$}  ---  --------", "-".repeat(path_w), path_w = path_w);

    for (index, size, name, ro, mounted) in &rows {
        let full_path = format!("/dev/{}", name);
        println!(
            "  Partition {:<3}  {:<7}  {:<path_w$}  {:<3}  {}",
            index, size, full_path, ro, mounted,
            path_w = path_w
        );
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

    // Collect rows first for dynamic column width
    let mut rows: Vec<(usize, String, String, &'static str, &'static str)> = Vec::new();
    let mut idx = 0;
    for device in parsed.blockdevices.iter().filter(|d| d.devtype == "disk") {
        if let Some(children) = &device.children {
            for part in children.iter().filter(|p| p.devtype == "part") {
                let ro = if part.ro.unwrap_or(false) { "Yes" } else { "No" };
                let mounted = if part.mountpoint.is_some() { "Yes" } else { "No" };
                rows.push((idx, part.size.clone(), part.name.clone(), ro, mounted));
                idx += 1;
            }
        }
    }

    if rows.is_empty() {
        println!("  (no volumes found)");
        return;
    }

    let path_w = rows.iter()
        .map(|(_, _, name, _, _)| "/dev/".len() + name.len())
        .max().unwrap()
        .max("Path".len());

    println!("  Volume ###  Size     {:<path_w$}  RO   Mounted", "Path", path_w = path_w);
    println!("  ----------  -------  {:<path_w$}  ---  --------", "-".repeat(path_w), path_w = path_w);

    for (index, size, name, ro, mounted) in &rows {
        let full_path = format!("/dev/{}", name);
        println!(
            "  Volume {:<3}  {:<7}  {:<path_w$}  {:<3}  {}",
            index, size, full_path, ro, mounted,
            path_w = path_w
        );
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
