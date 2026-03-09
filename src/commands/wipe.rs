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

//! WIPE  [!]
//!
//! Securely erase the selected disk or partition.
//! DiskParted Linux extension — no equivalent in Windows DiskPart.
//!
//! Syntax
//! ------
//!   wipe [method=zeros|random|discard] [passes=<n>]
//!
//! Parameters
//! ----------
//!   method=zeros    Overwrite with zeros using dd (fast; suitable for SSDs
//!                   when combined with full-disk encryption).
//!                   This is the default.
//!
//!   method=random   Overwrite with random data using shred (slow; intended
//!                   for HDDs where overwriting may aid unrecoverability).
//!
//!   method=discard  Send ATA TRIM or NVMe Dataset Management Discard to
//!                   every block via blkdiscard (SSDs/NVMe only; very fast).
//!                   Multiple overwrite passes do not improve security on
//!                   SSDs due to wear levelling — use discard or full-disk
//!                   encryption instead.
//!
//!   passes=<n>      Number of overwrite passes (default: 1).
//!                   Applies to zeros and random methods only.
//!
//! Target
//! ------
//!   The command operates on the selected partition if one is selected,
//!   otherwise on the selected disk.
//!
//! Requires
//! --------
//!   zeros   — dd (coreutils)
//!   random  — shred (coreutils)
//!   discard — blkdiscard (util-linux)

use std::process::Command;
use crate::context::Context;
use crate::utils::confirm;

pub fn run(args: &[&str], ctx: &mut Context) {
    // Prefer selected partition; fall back to the whole disk.
    let target: String = if let Some(p) = ctx.selected_partition.as_ref() {
        p.path.clone()
    } else if let Some(d) = ctx.selected_disk.as_ref() {
        d.path.clone()
    } else {
        eprintln!("No disk or partition is selected.");
        eprintln!("Use 'select disk <n>' or 'select partition <n>' first.");
        return;
    };

    let mut method = "zeros".to_string();
    let mut passes: u32 = 1;

    for arg in args {
        let lower = arg.to_lowercase();
        if let Some(v) = lower.strip_prefix("method=") {
            method = v.to_string();
        } else if let Some(v) = lower.strip_prefix("passes=") {
            match v.parse::<u32>() {
                Ok(n) if n > 0 => passes = n,
                _ => {
                    eprintln!("Invalid value for passes: '{}'", v);
                    return;
                }
            }
        } else {
            eprintln!("Unknown parameter: '{}'", arg);
            print_usage();
            return;
        }
    }

    if !matches!(method.as_str(), "zeros" | "random" | "discard") {
        eprintln!(
            "Unknown method '{}'. Valid options: zeros, random, discard.",
            method
        );
        return;
    }

    println!("WIPE");
    println!("  Target  : {}", target);
    println!("  Method  : {}", method);
    if method != "discard" {
        println!("  Passes  : {}", passes);
    }
    println!();
    println!(
        "  WARNING: This will PERMANENTLY AND IRREVERSIBLY destroy all \
         data on {}.",
        target
    );
    println!();

    if !confirm("Are you absolutely sure you want to wipe this device?") {
        println!("No change was made.");
        return;
    }

    match method.as_str() {
        "zeros"   => wipe_zeros(&target, passes),
        "random"  => wipe_random(&target, passes),
        "discard" => wipe_discard(&target),
        _         => unreachable!(),
    }
}

fn wipe_zeros(target: &str, passes: u32) {
    for pass in 1..=passes {
        println!("Pass {}/{}: writing zeros to {}...", pass, passes, target);
        match Command::new("dd")
            .args([
                "if=/dev/zero",
                &format!("of={}", target),
                "bs=4M",
                "oflag=direct",
                "status=progress",
            ])
            .status()
        {
            Ok(s) if s.success() => {}
            Ok(_) => {
                eprintln!("dd failed on pass {}.", pass);
                return;
            }
            Err(e) => {
                eprintln!("Failed to run dd: {}", e);
                return;
            }
        }
        let _ = Command::new("sync").status();
    }
    println!("Wipe complete: {} overwritten with zeros ({} pass(es)).", target, passes);
}

fn wipe_random(target: &str, passes: u32) {
    if which::which("shred").is_err() {
        eprintln!("'shred' not found. Install coreutils:  pacman -S coreutils");
        return;
    }

    println!(
        "Running shred with {} pass(es) on {}...",
        passes, target
    );
    println!("This may take a very long time on large drives.");

    match Command::new("shred")
        .args(["-v", &format!("-n{}", passes), target])
        .status()
    {
        Ok(s) if s.success() => {
            println!(
                "Wipe complete: {} overwritten with random data ({} pass(es)).",
                target, passes
            );
        }
        Ok(_) => eprintln!("shred failed."),
        Err(e) => eprintln!("Failed to run shred: {}", e),
    }
}

fn wipe_discard(target: &str) {
    if which::which("blkdiscard").is_err() {
        eprintln!("'blkdiscard' not found. Install util-linux:  pacman -S util-linux");
        return;
    }

    println!("Sending discard (TRIM) to {}...", target);

    match Command::new("blkdiscard").arg(target).status() {
        Ok(s) if s.success() => {
            println!("Discard complete. All blocks on {} marked as free.", target);
        }
        Ok(_) => {
            eprintln!("blkdiscard failed.");
            eprintln!(
                "The device may not support TRIM/discard. \
                 Try method=zeros instead."
            );
        }
        Err(e) => eprintln!("Failed to run blkdiscard: {}", e),
    }
}

fn print_usage() {
    println!("Syntax:  wipe [method=zeros|random|discard] [passes=<n>]");
    println!();
    println!("  method=zeros    Overwrite with zeros via dd (default)");
    println!("  method=random   Overwrite with random data via shred");
    println!("  method=discard  ATA TRIM / NVMe discard (SSDs only)");
    println!("  passes=<n>      Overwrite passes for zeros/random (default: 1)");
}
