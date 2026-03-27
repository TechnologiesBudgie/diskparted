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

/// Run the CREATE command.
///
/// Supported syntax (mirrors DiskPart):
///   create partition primary   [size=<MB>] [offset=<KB>]
///   create partition efi       [size=<MB>] [offset=<KB>]
///   create partition msr       [size=<MB>] [offset=<KB>]
///   create partition extended  [size=<MB>] [offset=<KB>]
///   create partition logical   [size=<MB>] [offset=<KB>]
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        print_usage();
        return;
    }

    match args[0] {
        "partition" => create_partition(&args[1..], ctx),
        _ => {
            println!("Unknown create target: '{}'.", args[0]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  create partition primary  [size=<MB>] [offset=<KB>]");
    println!("  create partition efi      [size=<MB>] [offset=<KB>]");
    println!("  create partition msr      [size=<MB>] [offset=<KB>]");
    println!("  create partition extended [size=<MB>] [offset=<KB>]");
    println!("  create partition logical  [size=<MB>] [offset=<KB>]");
}

fn create_partition(args: &[&str], ctx: &mut Context) {
    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    if args.is_empty() {
        print_usage();
        return;
    }

    let part_type = args[0].to_lowercase();
    let mut size_mb: Option<u64> = None;
    let mut offset_kb: Option<u64> = None;

    for arg in args.iter().skip(1) {
        if let Some(val) = arg.strip_prefix("size=") {
            match val.parse::<u64>() {
                Ok(n) => size_mb = Some(n),
                Err(_) => { println!("Invalid size value: '{}'.", val); return; }
            }
        } else if let Some(val) = arg.strip_prefix("offset=") {
            match val.parse::<u64>() {
                Ok(n) => offset_kb = Some(n),
                Err(_) => { println!("Invalid offset value: '{}'.", val); return; }
            }
        } else {
            println!("Unknown parameter: '{}'.", arg);
            return;
        }
    }

    let disk_path = disk.path.clone();

    match part_type.as_str() {
        "primary"  => create_primary(disk_path, size_mb, offset_kb),
        "efi"      => create_efi(disk_path, size_mb, offset_kb),
        "msr"      => create_msr(disk_path, size_mb, offset_kb),
        "extended" => create_extended(disk_path, size_mb, offset_kb),
        "logical"  => create_logical(disk_path, size_mb, offset_kb),
        _ => {
            println!("Unknown partition type: '{}'.", part_type);
            print_usage();
        }
    }
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Convert size_mb + optional offset_kb into parted start/end strings.
fn parted_bounds(size_mb: Option<u64>, offset_kb: Option<u64>) -> (String, String) {
    let start = match offset_kb {
        Some(kb) => format!("{}KiB", kb),
        None     => "0%".to_string(),
    };
    let end = match size_mb {
        Some(mb) => match offset_kb {
            Some(kb) => format!("{}KiB", kb + mb * 1024),
            None     => format!("{}MiB", mb),
        },
        None => "100%".to_string(),
    };
    (start, end)
}

/// Ensure a partition table exists, creating GPT if absent.
///
/// NOTE: All parted calls use `LC_ALL=C` so diagnostic output is always in
/// English, regardless of the system locale (e.g. French "étiquette de
/// disque inconnue" vs English "unrecognised disk label").
fn ensure_partition_table(disk_path: &str) -> bool {
    let check = Command::new("parted")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .args(["-s", disk_path, "print"])
        .output();

    let has_table = match check {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout);
            let err = String::from_utf8_lossy(&o.stderr);
            // With LC_ALL=C parted always outputs the English string.
            !out.contains("unrecognised disk label") && !err.contains("unrecognised disk label")
        }
        Err(_) => false,
    };

    if !has_table {
        println!("No partition table found. Creating GPT label...");
        let status = Command::new("parted")
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .args(["-s", disk_path, "mklabel", "gpt"])
            .status()
            .expect("Failed to execute parted");

        if !status.success() {
            println!("Failed to create partition table.");
            return false;
        }
        println!("GPT partition table created.");
    }

    true
}

fn run_parted(args: &[&str]) -> bool {
    Command::new("parted")
        .env("LC_ALL", "C").env("LANG", "C")
        .args(args)
        .status()
        .expect("Failed to execute parted")
        .success()
}

/// Returns the highest partition number on the disk by parsing parted output.
fn get_last_partition_number(disk_path: &str) -> Option<String> {
    let output = Command::new("parted")
        .env("LC_ALL", "C").env("LANG", "C")
        .args(["-s", disk_path, "print"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let last = text.lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter_map(|t| t.parse::<u32>().ok())
        .max()?;

    Some(last.to_string())
}

/// Returns "gpt" or "msdos" by parsing parted output.
fn get_disk_label(disk_path: &str) -> Option<String> {
    let output = Command::new("parted")
        .env("LC_ALL", "C").env("LANG", "C")
        .args(["-s", disk_path, "print"])
        .output()
        .ok()?;

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|l| l.trim_start().starts_with("Partition Table:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_lowercase())
}

// -----------------------------------------------------------------------
// Partition type implementations
// -----------------------------------------------------------------------

fn create_primary(disk_path: String, size_mb: Option<u64>, offset_kb: Option<u64>) {
    if !ensure_partition_table(&disk_path) { return; }

    let (start, end) = parted_bounds(size_mb, offset_kb);
    println!("Creating primary partition on {}...", disk_path);

    if run_parted(&["-s", &disk_path, "mkpart", "primary", &start, &end]) {
        println!("DiskPart succeeded in creating the specified partition.");
    } else {
        println!("DiskPart failed to create the specified partition.");
    }
}

fn create_efi(disk_path: String, size_mb: Option<u64>, offset_kb: Option<u64>) {
    if !ensure_partition_table(&disk_path) { return; }

    let size_mb = size_mb.unwrap_or(100); // DiskPart default is 100 MB
    let (start, end) = parted_bounds(Some(size_mb), offset_kb);

    println!("Creating EFI system partition ({} MB) on {}...", size_mb, disk_path);

    if !run_parted(&["-s", &disk_path, "mkpart", "EFI", "fat32", &start, &end]) {
        println!("DiskPart failed to create the specified partition.");
        return;
    }

    // Set the esp flag on the new partition
    if let Some(num) = get_last_partition_number(&disk_path) {
        let _ = run_parted(&["-s", &disk_path, "set", &num, "esp", "on"]);
    }

    println!("DiskPart succeeded in creating the specified partition.");
    println!("  Type : EFI System");
    println!("  Size : {} MB", size_mb);
}

fn create_msr(disk_path: String, size_mb: Option<u64>, offset_kb: Option<u64>) {
    if !ensure_partition_table(&disk_path) { return; }

    let size_mb = size_mb.unwrap_or(16); // DiskPart default is 16 MB
    let (start, end) = parted_bounds(Some(size_mb), offset_kb);

    println!("Creating Microsoft Reserved partition ({} MB) on {}...", size_mb, disk_path);

    if !run_parted(&["-s", &disk_path, "mkpart", "MSR", &start, &end]) {
        println!("DiskPart failed to create the specified partition.");
        return;
    }

    // Set the Microsoft Reserved GUID type (e3c9e316-...) via sgdisk
    if let Some(num) = get_last_partition_number(&disk_path) {
        let type_arg = format!("{}:e3c9e316-0b5c-4db8-817d-f92df00215ae", num);
        let _ = Command::new("sgdisk")
            .args(["--typecode", &type_arg, &disk_path])
            .status();
    }

    println!("DiskPart succeeded in creating the specified partition.");
    println!("  Type : Microsoft Reserved");
    println!("  Size : {} MB", size_mb);
    println!("  Note : MSR partitions cannot be formatted or mounted.");
}

fn create_extended(disk_path: String, size_mb: Option<u64>, offset_kb: Option<u64>) {
    if !ensure_partition_table(&disk_path) { return; }

    if get_disk_label(&disk_path).as_deref() == Some("gpt") {
        println!("The selected disk has a GPT partition table.");
        println!("Extended partitions are only supported on MBR disks.");
        println!("Use 'create partition primary' instead.");
        return;
    }

    let (start, end) = parted_bounds(size_mb, offset_kb);
    println!("Creating extended partition on {}...", disk_path);

    if run_parted(&["-s", &disk_path, "mkpart", "extended", &start, &end]) {
        println!("DiskPart succeeded in creating the specified partition.");
    } else {
        println!("DiskPart failed to create the specified partition.");
    }
}

fn create_logical(disk_path: String, size_mb: Option<u64>, offset_kb: Option<u64>) {
    if !ensure_partition_table(&disk_path) { return; }

    if get_disk_label(&disk_path).as_deref() == Some("gpt") {
        println!("The selected disk has a GPT partition table.");
        println!("Logical partitions are only supported on MBR disks.");
        return;
    }

    let (start, end) = parted_bounds(size_mb, offset_kb);
    println!("Creating logical partition on {}...", disk_path);

    if run_parted(&["-s", &disk_path, "mkpart", "logical", &start, &end]) {
        println!("DiskPart succeeded in creating the specified partition.");
    } else {
        println!("DiskPart failed to create the specified partition.");
    }
}
