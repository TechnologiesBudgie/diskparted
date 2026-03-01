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

/// GPT — assign GPT attribute bits to the selected partition.
///
/// DiskPart syntax:
///   gpt attributes=<n>
///
/// Common attribute values:
///   0x0000000000000001  — Required partition (do not delete)
///   0x8000000000000000  — No auto-assign drive letter
///   0x4000000000000000  — Hide partition from mount manager
///   0x2000000000000000  — Shadow copy partition
///   0x1000000000000000  — Read-only
///
/// On Linux this uses `sgdisk --attributes=<partnum>:set:<bit>`.
/// sgdisk uses bit numbers (0–63), so we convert the bitmask to the highest set bit.
pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p.clone(),
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    let disk_path = match ctx.selected_disk.as_ref() {
        Some(d) => d.path.clone(),
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    if args.is_empty() {
        print_usage();
        return;
    }

    let mut attr_val: Option<u64> = None;

    for arg in args {
        if let Some(val) = arg.strip_prefix("attributes=") {
            // Accept 0x… hex or decimal
            let parsed = if val.starts_with("0x") || val.starts_with("0X") {
                u64::from_str_radix(val.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
            } else {
                val.parse::<u64>().ok()
            };
            attr_val = parsed;
            if attr_val.is_none() {
                println!("Invalid attributes value: '{}'. Must be a hex (0x…) or decimal number.", val);
                return;
            }
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    let mask = match attr_val {
        Some(v) => v,
        None => { print_usage(); return; }
    };

    if mask == 0 {
        println!("Attribute value is 0 — nothing to set.");
        return;
    }

    // sgdisk --attributes takes individual bit numbers.
    // We set each bit that is present in the mask.
    let mut any_set = false;
    for bit in 0u64..64 {
        if mask & (1u64 << bit) != 0 {
            let attr_arg = format!("{}:set:{}", partition.index, bit);
            println!("Setting GPT attribute bit {} on partition {}...", bit, partition.index);

            let status = Command::new("sudo")
                .args(["sgdisk", &format!("--attributes={}", attr_arg), &disk_path])
                .status()
                .expect("Failed to execute sgdisk");

            if status.success() {
                any_set = true;
            } else {
                println!("Failed to set attribute bit {}.", bit);
            }
        }
    }

    if any_set {
        println!("GPT attributes updated successfully.");
        println!("Warning: changing GPT attributes may prevent drive letter assignment or mounting.");
    } else {
        println!("No attributes were set successfully.");
    }
}

fn print_usage() {
    println!("Usage: gpt attributes=<n>");
    println!();
    println!("Common values (can be combined with bitwise OR):");
    println!("  0x0000000000000001  Required partition");
    println!("  0x8000000000000000  No auto drive letter");
    println!("  0x4000000000000000  Hide from mount manager");
    println!("  0x2000000000000000  Shadow copy");
    println!("  0x1000000000000000  Read-only");
}
