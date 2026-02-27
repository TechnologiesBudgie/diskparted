use serde::Deserialize;
use std::process::Command;

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
    children: Option<Vec<Device>>,
}

pub fn run(args: &[&str], ctx: &Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  list disk");
        println!("  list volume");
        return;
    }

    match args[0] {
        "disk" => list_disks(),
        "volume" => list_partitions(ctx),
        _ => println!("Unknown list target."),
    }
}

pub fn get_disks() -> Vec<Disk> {
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE"])
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk =
        serde_json::from_slice(&output.stdout).expect("Failed to parse lsblk JSON");

    let mut disks = Vec::new();
    let mut index = 0;

    for device in parsed.blockdevices {
        if device.devtype == "disk" {
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

fn list_disks() {
    let disks = get_disks();

    println!("  Disk ###  Size     Path");
    println!("  --------  -------  --------");

    for disk in disks {
        println!(
            "  Disk {:<3}  {:<7}  {}",
            disk.index,
            disk.size,
            disk.path
        );
    }
}

fn list_partitions(ctx: &Context) {
    let selected = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected.");
            return;
        }
    };

    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE"])
        .arg(&selected.path)
        .output()
        .expect("Failed to execute lsblk");

    let parsed: Lsblk =
        serde_json::from_slice(&output.stdout).expect("Failed to parse lsblk JSON");

    println!("  Volume ###  Size     Path");
    println!("  ----------  -------  --------");

    for device in parsed.blockdevices {
        if let Some(children) = device.children {
            for (i, part) in children.iter().enumerate() {
                if part.devtype == "part" {
                    println!(
                        "  Volume {:<3}  {:<7}  /dev/{}",
                        i,
                        part.size,
                        part.name
                    );
                }
            }
        }
    }
}
