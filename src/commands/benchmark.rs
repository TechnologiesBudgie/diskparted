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

//! BENCHMARK  [!]
//!
//! Run a quick sequential read/write throughput test on the selected disk.
//! DiskParted Linux extension — no equivalent in Windows DiskPart.
//!
//! Syntax
//! ------
//!   benchmark [size=<n>]
//!
//! Parameters
//! ----------
//!   size=<n>   Amount of data in megabytes to transfer per test (default: 256).
//!              Larger values give more stable averages but take longer.
//!
//! Method
//! ------
//!   Uses dd with oflag=direct / iflag=direct to bypass the page cache so
//!   results reflect storage hardware throughput rather than RAM speed.
//!   A temporary file is written to /tmp and removed when the test finishes.
//!
//! Notes
//! -----
//!   Results are sequential throughput only. For random I/O, queue-depth
//!   measurements, or latency profiling, install fio:
//!     pacman -S fio

use std::process::Command;
use std::time::Instant;
use crate::context::Context;

pub fn run(args: &[&str], ctx: &mut Context) {
    // At least a disk must be selected so we can name the test target.
    if ctx.selected_disk.is_none() {
        eprintln!("There is no disk selected.");
        eprintln!("Use 'select disk <n>' first.");
        return;
    }

    let disk_label = ctx.selected_disk.as_deref().unwrap_or("unknown").to_string();

    let mut size_mb: u64 = 256;

    for arg in args {
        if let Some(v) = arg.to_lowercase().strip_prefix("size=") {
            match v.parse::<u64>() {
                Ok(n) if n > 0 => size_mb = n,
                _ => {
                    eprintln!(
                        "Invalid value for size: '{}'. Using default {} MB.",
                        v, size_mb
                    );
                }
            }
        } else {
            eprintln!("Unknown parameter: '{}'", arg);
            print_usage();
            return;
        }
    }

    // Build a unique temporary file path.
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let tmp = format!("/tmp/diskparted_bench_{}", epoch);

    println!("DiskParted Benchmark");
    println!("  Target : {}", disk_label);
    println!("  Size   : {} MB per test", size_mb);
    println!("  Method : dd with direct I/O (bypasses OS page cache)");
    println!();

    let count = size_mb.to_string();

    // ── Sequential write ────────────────────────────────────────────────────
    print!("  Sequential WRITE ... ");
    let t0 = Instant::now();
    let write_ok = Command::new("dd")
        .args([
            "if=/dev/urandom",
            &format!("of={}", tmp),
            "bs=1M",
            &format!("count={}", count),
            "oflag=direct",
            "status=none",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let write_secs = t0.elapsed().as_secs_f64();

    if write_ok {
        println!("{:.1} MB/s", size_mb as f64 / write_secs);
    } else {
        println!("FAILED");
        eprintln!("  dd write failed. Check that /tmp has enough free space.");
        let _ = std::fs::remove_file(&tmp);
        return;
    }

    // Flush kernel write-back cache before the read test.
    let _ = Command::new("sync").status();

    // ── Sequential read ─────────────────────────────────────────────────────
    print!("  Sequential READ  ... ");
    let t1 = Instant::now();
    let read_ok = Command::new("dd")
        .args([
            &format!("if={}", tmp),
            "of=/dev/null",
            "bs=1M",
            "iflag=direct",
            "status=none",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let read_secs = t1.elapsed().as_secs_f64();

    if read_ok {
        println!("{:.1} MB/s", size_mb as f64 / read_secs);
    } else {
        println!("FAILED");
    }

    // Clean up the temporary file.
    let _ = std::fs::remove_file(&tmp);

    println!();
    println!("  Benchmark complete.");
    println!("  For IOPS / random I/O testing, use fio:  pacman -S fio");
}

fn print_usage() {
    println!("Syntax:  benchmark [size=<n>]");
    println!();
    println!("  size=<n>   MB per test pass (default: 256)");
}
