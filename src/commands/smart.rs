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

//! SMART  [!]
//!
//! Display S.M.A.R.T. health data for the selected disk.
//! DiskParted Linux extension — no equivalent in Windows DiskPart.
//!
//! Syntax
//! ------
//!   smart
//!   smart full
//!   smart test [short|long|conveyance]
//!
//! Subcommands
//! -----------
//!   (none)             Print overall health and key SMART attributes.
//!   full               Pass through the complete `smartctl -a` output.
//!   test <type>        Run a background self-test.
//!                        short      ~2 min, basic electrical/mechanical check
//!                        long       full read-scan (hours on large drives)
//!                        conveyance after-shipping damage check (if supported)
//!
//! Requires
//! --------
//!   smartmontools  (pacman -S smartmontools)

use std::process::Command;
use crate::context::Context;

pub fn run(args: &[&str], ctx: &mut Context) {
    let disk_dev = match ctx.selected_disk.as_ref() {
        Some(d) => d.clone(),
        None => {
            eprintln!("There is no disk selected.");
            eprintln!("Use 'select disk <n>' first.");
            return;
        }
    };

    if !tool_available("smartctl", "smartmontools") {
        return;
    }

    match args.first().map(|s| s.to_lowercase()).as_deref() {
        None | Some("") => smart_summary(&*disk_dev),
        Some("full") => smart_full(&*disk_dev),
        Some("test") => {
            let kind = args.get(1).copied().unwrap_or("short");
            smart_test(&*disk_dev, kind);
        }
        Some(other) => {
            eprintln!("Unknown subcommand: '{}'. Valid: full, test.", other);
            print_usage();
        }
    }
}

fn smart_summary(disk: &str) {
    println!("S.M.A.R.T. status for {}:", disk);
    println!();

    // Overall health assessment.
    if let Ok(out) = Command::new("smartctl").args(["-H", disk]).output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.contains("SMART overall-health") || line.contains("result:") {
                println!("  {}", line.trim());
            }
        }
    }

    println!();

    // Print a curated subset of SMART attributes that matter most.
    let key_attrs = [
        "Reallocated_Sector_Ct",
        "Reported_Uncorrect",
        "Command_Timeout",
        "Current_Pending_Sector",
        "Offline_Uncorrectable",
        "Power_On_Hours",
        "Temperature_Celsius",
        "SSD_Life_Left",
        "Wear_Leveling_Count",
        "Media_Wearout_Indicator",
    ];

    if let Ok(out) = Command::new("smartctl").args(["-A", disk]).output() {
        let mut header_printed = false;
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if !key_attrs.iter().any(|k| line.contains(k)) {
                continue;
            }
            if !header_printed {
                println!(
                    "  {:<34} {:>5}  {:>5}  {:>5}  {}",
                    "Attribute", "Value", "Worst", "Thresh", "Raw"
                );
                println!("  {}", "-".repeat(70));
                header_printed = true;
            }
            // Columns: ID  ATTR  FLAG  VALUE  WORST  THRESH  TYPE  UPDATED  FAILED  RAW
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 10 {
                println!(
                    "  {:<34} {:>5}  {:>5}  {:>5}  {}",
                    cols[1],
                    cols[3],
                    cols[4],
                    cols[5],
                    cols[9..].join(" ")
                );
            }
        }
        if !header_printed {
            println!("  No key attributes found.");
            println!("  The disk may use NVMe — run 'smart full' for NVMe-specific data.");
        }
    }

    println!();
    println!("  Tip: 'smart full' for the complete attribute list.");
    println!("       'smart test short' to initiate a quick self-test.");
}

fn smart_full(disk: &str) {
    println!("Full S.M.A.R.T. report for {}:", disk);
    println!();
    let _ = Command::new("smartctl").args(["-a", disk]).status();
}

fn smart_test(disk: &str, kind: &str) {
    let test_type = match kind.to_lowercase().as_str() {
        "short" => "short",
        "long" => "long",
        "conveyance" => "conveyance",
        other => {
            eprintln!(
                "Unknown test type '{}'. Valid options: short, long, conveyance.",
                other
            );
            return;
        }
    };

    println!("Starting {} self-test on {}...", test_type, disk);
    println!("The test runs in the background on the drive firmware.");
    println!("Run 'smart' again in a few minutes to check progress.");
    println!();

    match Command::new("smartctl")
        .args(["-t", test_type, disk])
        .status()
    {
        Ok(s) if s.success() => println!("Self-test initiated successfully."),
        Ok(_) => eprintln!(
            "smartctl returned an error. \
             The drive may not support this test type."
        ),
        Err(e) => eprintln!("Failed to run smartctl: {}", e),
    }
}

fn tool_available(tool: &str, package: &str) -> bool {
    if which::which(tool).is_err() {
        eprintln!("'{}' not found.", tool);
        eprintln!("Install it with:  pacman -S {}", package);
        false
    } else {
        true
    }
}

fn print_usage() {
    println!("Syntax:  smart");
    println!("         smart full");
    println!("         smart test [short|long|conveyance]");
}
