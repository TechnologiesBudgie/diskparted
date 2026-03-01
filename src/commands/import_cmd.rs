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

/// IMPORT — import a foreign disk group / volume group / ZFS pool.
///
/// DiskPart syntax:  import [noerr]
///
/// On Linux, DiskPart's "foreign disk group" concept maps to:
///   - LVM: `vgimport` / `vgscan` to import a VG from a moved disk
///   - ZFS: `zpool import` to import a pool from a moved disk
///
/// This command scans for importable groups and imports them all,
/// which matches DiskPart's "import all foreign disks" behaviour.
/// Pass `vg=<name>` or `pool=<name>` to import a specific group.
pub fn run(args: &[&str], _ctx: &mut Context) {
    let mut target_vg:   Option<String> = None;
    let mut target_pool: Option<String> = None;
    let mut noerr = false;

    for arg in args {
        if let Some(v) = arg.strip_prefix("vg=") {
            target_vg = Some(v.to_string());
        } else if let Some(p) = arg.strip_prefix("pool=") {
            target_pool = Some(p.to_string());
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    let mut imported_anything = false;

    // --- LVM ---
    if which::which("vgscan").is_ok() {
        println!("Scanning for LVM volume groups...");
        let _ = Command::new("sudo").args(["vgscan", "--cache"]).status();

        let vgs = list_lvm_foreign_vgs();

        if vgs.is_empty() && target_vg.is_none() {
            println!("  No foreign LVM volume groups found.");
        } else {
            let to_import: Vec<String> = match &target_vg {
                Some(name) => vec![name.clone()],
                None       => vgs,
            };

            for vg in &to_import {
                println!("  Importing LVM volume group '{}'...", vg);
                let status = Command::new("sudo")
                    .args(["vgimport", vg])
                    .status()
                    .expect("Failed to execute vgimport");

                if status.success() {
                    // Activate the VG after importing
                    let _ = Command::new("sudo")
                        .args(["vgchange", "-ay", vg])
                        .status();
                    println!("  Volume group '{}' imported and activated.", vg);
                    imported_anything = true;
                } else {
                    println!("  Failed to import volume group '{}'.", vg);
                    if noerr { println!("  (noerr: continuing)"); }
                }
            }
        }
    }

    println!();

    // --- ZFS ---
    if which::which("zpool").is_ok() {
        println!("Scanning for importable ZFS pools...");

        let status = match &target_pool {
            Some(name) => Command::new("sudo")
                .args(["zpool", "import", name])
                .status()
                .expect("Failed to execute zpool"),
            None => Command::new("sudo")
                .args(["zpool", "import", "-a"])
                .status()
                .expect("Failed to execute zpool"),
        };

        if status.success() {
            println!("  ZFS pool(s) imported successfully.");
            imported_anything = true;
        } else {
            println!("  No importable ZFS pools found, or import failed.");
            if noerr { println!("  (noerr: continuing)"); }
        }
    }

    if !imported_anything {
        println!();
        println!("No foreign disk groups were imported.");
        println!("Make sure the disk containing the volume group or pool is connected and visible.");
    }
}

/// Returns a list of LVM volume groups that are not currently active (foreign/exported).
fn list_lvm_foreign_vgs() -> Vec<String> {
    let output = match Command::new("sudo")
        .args(["vgs", "--noheadings", "-o", "vg_name,vg_attr"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let name = cols.next()?;
            let attrs = cols.next().unwrap_or("");
            // 'x' in the attr string means exported/foreign
            if attrs.contains('x') || attrs.contains('e') {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}
