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

/// BREAK — break a mirrored (RAID-1) volume.
///
/// DiskPart syntax:
///   break disk=<n> [nokeep] [noerr]
///
/// DiskPart breaks a mirrored volume: one half stays as the volume, the other
/// becomes a free simple volume (nokeep = both halves become simple volumes).
///
/// On Linux this stops the md RAID array that contains the selected partition.
/// After stopping:
///   - Without `nokeep`: the array is degraded/stopped but the member data
///     is left intact — one leg keeps the data and remains accessible.
///   - With `nokeep`:    the array is stopped AND the superblock is wiped from
///     all members, leaving both as independent unformatted partitions.
pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No volume selected. Use 'select volume <n>' or 'select partition <n>' first.");
            return;
        }
    };

    let mut nokeep = false;
    let mut noerr  = false;

    for arg in args {
        if arg.eq_ignore_ascii_case("nokeep") {
            nokeep = true;
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else if arg.starts_with("disk=") {
            // disk= is accepted for DiskPart syntax compatibility but we auto-detect
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    if !which::which("mdadm").is_ok() {
        println!("Error: 'mdadm' is not installed. Install it with: sudo apt install mdadm");
        return;
    }

    // Find which md array the selected partition belongs to
    let md_dev = match find_md_array_for(&partition.path) {
        Some(d) => d,
        None => {
            println!("The selected volume ({}) is not part of a software RAID array.", partition.path);
            println!("Only mirrored (RAID-1) volumes created with 'add disk=<n>' can be broken.");
            return;
        }
    };

    // Get all member devices of this array
    let members = get_md_members(&md_dev);

    println!("Found RAID array {} with members:", md_dev);
    for m in &members {
        println!("  {}", m);
    }

    if nokeep {
        println!();
        println!("WARNING: 'nokeep' specified — the array will be stopped and ALL member");
        println!("superblocks will be wiped. Both halves become raw, unformatted partitions.");
    } else {
        println!();
        println!("The array will be stopped. Member data is preserved on each partition.");
        println!("The first member will remain accessible as an independent partition.");
    }

    if !utils::confirm("Do you want to break this mirror?") {
        println!("Aborted.");
        return;
    }

    // Unmount first
    let _ = Command::new("sudo").args(["umount", &md_dev]).output();

    // Stop the array
    let stop_status = Command::new("sudo")
        .args(["mdadm", "--stop", &md_dev])
        .status()
        .expect("Failed to execute mdadm");

    if !stop_status.success() {
        println!("Failed to stop RAID array {}.", md_dev);
        if noerr { println!("(noerr: continuing despite error)"); }
        return;
    }

    println!("RAID array {} stopped.", md_dev);

    if nokeep {
        // Wipe mdadm superblocks from all members so they can be used independently
        for member in &members {
            println!("Wiping RAID superblock from {}...", member);
            let _ = Command::new("sudo")
                .args(["mdadm", "--zero-superblock", member])
                .status();
        }
        println!("All member superblocks wiped. Partitions are now independent.");
    } else {
        println!("Member partitions retain their data.");
        if let Some(first) = members.first() {
            println!("First member ({}) can be mounted directly.", first);
        }
    }

    println!();
    println!("Remember to update /etc/mdadm/mdadm.conf if this array was persistent.");
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

/// Get the list of member devices in an md array via mdadm --detail.
fn get_md_members(md_dev: &str) -> Vec<String> {
    let output = match Command::new("sudo")
        .args(["mdadm", "--detail", md_dev])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| l.trim().contains("/dev/") && !l.contains("Array Device"))
        .filter_map(|l| l.split_whitespace().last().map(|s| s.to_string()))
        .filter(|s| s.starts_with("/dev/"))
        .collect()
}
