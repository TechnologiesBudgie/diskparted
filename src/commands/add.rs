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
use crate::utils;

/// ADD — add a mirror disk to a simple volume (software RAID-1 via mdadm).
///
/// DiskPart syntax:
///   add disk=<n> [noerr]
///
/// DiskPart's ADD command mirrors a simple volume onto a second disk,
/// creating a mirrored (RAID-1) volume. On Linux the closest equivalent is
/// creating a RAID-1 array with mdadm.
///
/// Two scenarios are handled:
///   1. The selected partition is already an md RAID member:
///      — adds `disk=<n>` as a new member to the existing array.
///   2. The selected partition is a plain partition:
///      — creates a NEW md RAID-1 array between the current partition
///        and the specified disk (the entire disk is used as the second leg).
///
/// disk=<n>  — index of the disk to add as a mirror (from 'list disk').
pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    if args.is_empty() {
        println!("Usage: add disk=<n> [noerr]");
        println!("  disk=<n>  index of the mirror disk (from 'list disk')");
        return;
    }

    let mut mirror_disk_index: Option<usize> = None;
    let mut noerr = false;

    for arg in args {
        if let Some(val) = arg.strip_prefix("disk=") {
            match val.parse::<usize>() {
                Ok(n) => mirror_disk_index = Some(n),
                Err(_) => { println!("Invalid disk index: '{}'.", val); return; }
            }
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    let mirror_idx = match mirror_disk_index {
        Some(n) => n,
        None => { println!("Error: disk= parameter is required."); return; }
    };

    // Resolve the target disk path from the index
    let mirror_disk_path = match get_disk_path_by_index(mirror_idx) {
        Some(p) => p,
        None => {
            println!("Disk {} not found. Run 'list disk' to see available disks.", mirror_idx);
            return;
        }
    };

    if !which::which("mdadm").is_ok() {
        println!("Error: 'mdadm' is not installed. Install it with: sudo apt install mdadm");
        return;
    }

    // Check whether the partition is already an md member
    if let Some(md_dev) = find_md_array_for(&partition.path) {
        // Scenario 1: add the new disk to the existing RAID array
        println!("Partition {} is part of RAID array {}.", partition.path, md_dev);
        println!("Adding {} as a new mirror member to {}...", mirror_disk_path, md_dev);

        if !utils::confirm("This will begin RAID resync. Continue?") {
            println!("Aborted."); return;
        }

        let status = Command::new("sudo")
            .args(["mdadm", "--manage", &md_dev, "--add", &mirror_disk_path])
            .status()
            .expect("Failed to execute mdadm");

        if status.success() {
            println!("Disk {} added to {}. Resync has started.", mirror_disk_path, md_dev);
            println!("Monitor progress: cat /proc/mdstat");
        } else {
            println!("Failed to add disk to RAID array.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }

    } else {
        // Scenario 2: create a new RAID-1 array from the current partition + new disk
        println!("Creating new RAID-1 mirror between {} and {}...", partition.path, mirror_disk_path);
        println!("WARNING: All data on {} will be erased.", mirror_disk_path);

        if !utils::confirm("Do you want to continue?") {
            println!("Aborted."); return;
        }

        // Find an available /dev/md* device name
        let md_dev = find_free_md_device();

        let status = Command::new("sudo")
            .args([
                "mdadm", "--create", &md_dev,
                "--level=mirror",
                "--raid-devices=2",
                &partition.path,
                &mirror_disk_path,
                "--run",          // don't wait for confirmation
            ])
            .status()
            .expect("Failed to execute mdadm");

        if status.success() {
            println!("RAID-1 mirror created at {}.", md_dev);
            println!("Initial sync started. Monitor: cat /proc/mdstat");
            println!();
            println!("To make this persistent, run:");
            println!("  sudo mdadm --detail --scan >> /etc/mdadm/mdadm.conf");
            println!("  sudo update-initramfs -u");
        } else {
            println!("Failed to create RAID-1 mirror.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    }
}

/// Find the md RAID device that contains the given partition as a member.
fn find_md_array_for(part_path: &str) -> Option<String> {
    let mdstat = std::fs::read_to_string("/proc/mdstat").ok()?;
    let part_name = part_path.trim_start_matches("/dev/");

    for line in mdstat.lines() {
        if line.starts_with("md") && line.contains(part_name) {
            let md_name = line.split_whitespace().next()?;
            return Some(format!("/dev/{}", md_name));
        }
    }
    None
}

/// Find a free /dev/mdN device name.
fn find_free_md_device() -> String {
    for n in 0..128 {
        let path = format!("/dev/md{}", n);
        if !std::path::Path::new(&path).exists() {
            return path;
        }
    }
    "/dev/md127".to_string()
}

/// Get the /dev path of disk at the given list index.
fn get_disk_path_by_index(index: usize) -> Option<String> {
    use std::process::Command;
    let output = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE,TRAN"])
        .output().ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let disks: Vec<&str> = text.lines()
        .filter(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            cols.get(1) == Some(&"disk") && cols.get(2).is_some()
        })
        .collect();

    disks.get(index).map(|line| {
        let name = line.split_whitespace().next().unwrap_or("");
        format!("/dev/{}", name)
    })
}
