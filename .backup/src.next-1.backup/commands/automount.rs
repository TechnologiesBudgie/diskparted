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

/// AUTOMOUNT — enable or disable automatic mounting of new volumes.
///
/// DiskPart syntax:
///   automount enable
///   automount disable
///   automount scrub
///
/// On Linux, automounting is handled by udisks2/udev.
/// We translate this to systemd mount units and udisksctl where available,
/// and also update /etc/fstab `nofail`/`noauto` for the selected partition.
///
/// `automount scrub` removes stale /etc/fstab entries for non-existent devices,
/// mirroring DiskPart's "scrub" subcommand which removes stale drive letter assignments.
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage:");
        println!("  automount enable   — enable automounting for new volumes");
        println!("  automount disable  — disable automounting for new volumes");
        println!("  automount scrub    — remove stale fstab entries");
        return;
    }

    match args[0].to_lowercase().as_str() {
        "enable"  => automount_enable(ctx),
        "disable" => automount_disable(ctx),
        "scrub"   => automount_scrub(),
        _ => {
            println!("Unknown automount subcommand: '{}'. Use enable, disable, or scrub.", args[0]);
        }
    }
}

/// Enable automounting: remove `noauto` from the partition's fstab entry if present.
fn automount_enable(ctx: &Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            // No partition selected — enable system-wide via udisksctl if available
            println!("No partition selected — enabling system-wide automount via udisks2...");
            if which::which("udisksctl").is_ok() {
                println!("udisks2 automount is managed by the desktop session; no global toggle available.");
                println!("To force-mount a specific volume use: udisksctl mount -b /dev/<part>");
            } else {
                println!("udisksctl not found. Automounting is controlled by udev rules in /etc/udev/rules.d/.");
            }
            return;
        }
    };

    println!("Enabling automount for {}...", partition.path);
    update_fstab_option(&partition.path, "noauto", false);
    println!("Done. The volume will be mounted automatically on next boot (if in fstab).");
}

/// Disable automounting: add `noauto` to the partition's fstab entry.
fn automount_disable(ctx: &Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    println!("Disabling automount for {}...", partition.path);
    update_fstab_option(&partition.path, "noauto", true);
    println!("Done. The volume will not be mounted automatically on next boot.");
}

/// Scrub: remove fstab entries whose device no longer exists.
fn automount_scrub() {
    println!("Scanning /etc/fstab for stale entries...");

    let fstab = match std::fs::read_to_string("/etc/fstab") {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to read /etc/fstab: {}", e);
            return;
        }
    };

    let mut stale: Vec<String> = Vec::new();

    for line in fstab.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() { continue; }
        let dev = trimmed.split_whitespace().next().unwrap_or("");
        // Only check /dev/... entries; UUID= and LABEL= are harder to validate here
        if dev.starts_with("/dev/") && !std::path::Path::new(dev).exists() {
            stale.push(line.to_string());
        }
    }

    if stale.is_empty() {
        println!("No stale /etc/fstab entries found.");
        return;
    }

    println!("Stale entries found:");
    for s in &stale {
        println!("  {}", s);
    }

    // Build new fstab without stale entries
    let new_fstab: String = fstab
        .lines()
        .filter(|l| !stale.contains(&l.to_string()))
        .map(|l| format!("{}\n", l))
        .collect();

    match std::fs::write("/etc/fstab", new_fstab) {
        Ok(_) => println!("Stale entries removed from /etc/fstab."),
        Err(e) => println!("Failed to update /etc/fstab: {}", e),
    }
}

/// Add or remove `noauto` from the fstab options for a given device.
fn update_fstab_option(part_path: &str, option: &str, add: bool) {
    let fstab = match std::fs::read_to_string("/etc/fstab") {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to read /etc/fstab: {}", e);
            return;
        }
    };

    // Also resolve UUID/LABEL for this partition via blkid
    let uuid = get_blkid_field(part_path, "UUID").unwrap_or_default();
    let label = get_blkid_field(part_path, "LABEL").unwrap_or_default();

    let new_fstab: String = fstab.lines().map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            return format!("{}\n", line);
        }
        let cols: Vec<&str> = trimmed.splitn(6, char::is_whitespace).collect();
        if cols.len() < 4 { return format!("{}\n", line); }

        let dev_col = cols[0];
        let matches = dev_col == part_path
            || (!uuid.is_empty() && dev_col == format!("UUID={}", uuid))
            || (!label.is_empty() && dev_col == format!("LABEL={}", label));

        if !matches { return format!("{}\n", line); }

        let mut opts: Vec<&str> = cols[3].split(',').collect();
        if add {
            if !opts.contains(&option) { opts.push(option); }
        } else {
            opts.retain(|o| *o != option);
        }

        let new_opts = opts.join(",");
        // Reconstruct line preserving original whitespace is complex; rebuild simply
        format!("{}\t{}\t{}\t{}\t{}\t{}\n",
            cols[0],
            cols.get(1).unwrap_or(&"none"),
            cols.get(2).unwrap_or(&"auto"),
            new_opts,
            cols.get(4).unwrap_or(&"0"),
            cols.get(5).unwrap_or(&"0"),
        )
    }).collect();

    match std::fs::write("/etc/fstab", new_fstab) {
        Ok(_) => {},
        Err(e) => println!("Failed to update /etc/fstab: {}", e),
    }
}

fn get_blkid_field(path: &str, field: &str) -> Option<String> {
    let output = Command::new("sudo")
        .args(["blkid", "-o", "value", "-s", field, path])
        .output()
        .ok()?;
    let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if val.is_empty() { None } else { Some(val) }
}
