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

//! Virtual disk management — replaces ATTACH / DETACH / COMPACT / EXPAND.
//!
//! Supported image formats (via qemu-img + NBD or loop):
//!   qcow2, raw, vdi, vmdk, vhd (vpc), hdd (parallels)
//!
//! Commands:
//!   vdisk create  <file> format=<fmt> size=<MB>
//!   vdisk attach  <file>
//!   vdisk detach  [<file>|all]
//!   vdisk compact <file>
//!   vdisk expand  <file> size=<MB>
//!   vdisk info    <file>
//!   vdisk list

use std::process::Command;
use std::path::Path;
use crate::context::Context;

/// All supported virtual disk formats
const FORMATS: &[(&str, &str)] = &[
    ("qcow2", "QEMU Copy-On-Write v2 (snapshots, compression)"),
    ("raw",   "Raw disk image (dd-compatible)"),
    ("vdi",   "VirtualBox Disk Image"),
    ("vmdk",  "VMware Virtual Machine Disk"),
    ("vhd",   "Virtual Hard Disk (Hyper-V / Azure)"),
    ("hdd",   "Parallels HDD image"),
];

pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        print_usage();
        return;
    }

    match args[0].to_lowercase().as_str() {
        "create"  => vdisk_create(&args[1..]),
        "attach"  => vdisk_attach(&args[1..], ctx),
        "detach"  => vdisk_detach(&args[1..], ctx),
        "compact" => vdisk_compact(&args[1..]),
        "expand"  => vdisk_expand(&args[1..]),
        "info"    => vdisk_info(&args[1..]),
        "list"    => vdisk_list(),
        _ => {
            println!("Unknown vdisk subcommand: '{}'.", args[0]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  vdisk create  <file> format=<fmt> size=<MB>");
    println!("  vdisk attach  <file>");
    println!("  vdisk detach  [<file> | all]");
    println!("  vdisk compact <file>");
    println!("  vdisk expand  <file> size=<MB>");
    println!("  vdisk info    <file>");
    println!("  vdisk list");
    println!();
    println!("Supported formats:");
    for (fmt, desc) in FORMATS {
        println!("  {:<8}  {}", fmt, desc);
    }
}

// -----------------------------------------------------------------------
// CREATE
// -----------------------------------------------------------------------
fn vdisk_create(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: vdisk create <file> format=<fmt> size=<MB>");
        return;
    }

    let file = args[0];
    let mut format: Option<String> = None;
    let mut size_mb: Option<u64> = None;

    for arg in args.iter().skip(1) {
        if let Some(v) = arg.strip_prefix("format=") {
            format = Some(v.to_lowercase());
        } else if let Some(v) = arg.strip_prefix("size=") {
            match v.parse::<u64>() {
                Ok(n) => size_mb = Some(n),
                Err(_) => { println!("Invalid size: '{}'.", v); return; }
            }
        } else {
            println!("Unknown parameter: '{}'.", arg);
            return;
        }
    }

    let format = match format {
        Some(f) => f,
        None => { println!("format= is required. Example: format=qcow2"); return; }
    };

    let size_mb = match size_mb {
        Some(s) => s,
        None => { println!("size= is required. Example: size=10240"); return; }
    };

    if !is_valid_format(&format) {
        println!("Unsupported format '{}'. Supported: qcow2, raw, vdi, vmdk, vhd, hdd", format);
        return;
    }

    if !qemu_img_available() { return; }

    // qemu-img uses "vpc" for VHD and "parallels" for HDD
    let qemu_fmt = to_qemu_format(&format);
    let size_arg = format!("{}M", size_mb);

    println!("Creating {} image '{}' ({} MB)...", format.to_uppercase(), file, size_mb);

    let status = Command::new("qemu-img")
        .args(["create", "-f", qemu_fmt, file, &size_arg])
        .status()
        .expect("Failed to execute qemu-img");

    if status.success() {
        println!("Virtual disk created successfully: {}", file);
    } else {
        println!("Failed to create virtual disk.");
    }
}

// -----------------------------------------------------------------------
// ATTACH
// -----------------------------------------------------------------------
fn vdisk_attach(args: &[&str], _ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage: vdisk attach <file>");
        return;
    }

    let file = args[0];

    if !Path::new(file).exists() {
        println!("File not found: '{}'.", file);
        return;
    }

    if !qemu_img_available() { return; }

    // Load nbd kernel module if not loaded
    let _ = Command::new("modprobe").arg("nbd").arg("max_part=16").status();

    // Find a free /dev/nbdN device
    let nbd_dev = match find_free_nbd() {
        Some(d) => d,
        None => {
            println!("No free NBD device found. Make sure the nbd kernel module is loaded.");
            println!("  Run: modprobe nbd max_part=16");
            return;
        }
    };

    let fmt = detect_format(file).unwrap_or_else(|| "raw".to_string());
    let qemu_fmt = to_qemu_format(&fmt);

    println!("Attaching '{}' as {} to {}...", file, fmt.to_uppercase(), nbd_dev);

    let status = Command::new("qemu-nbd")
        .args(["--connect", &nbd_dev, "-f", qemu_fmt, file])
        .status()
        .expect("Failed to execute qemu-nbd");

    if status.success() {
        // Give the kernel a moment to create partition nodes
        let _ = Command::new("partprobe").arg(&nbd_dev).status();
        println!("Virtual disk attached: {}", nbd_dev);
        println!("Use 'select disk' to select it, or 'list disk' to see it.");

        // Store in context as selected disk if user wants
        println!("Tip: run 'rescan' then 'list disk' to see the new device.");
    } else {
        println!("Failed to attach virtual disk.");
        println!("Make sure qemu-nbd is installed: pacman -S qemu-img");
    }
}

// -----------------------------------------------------------------------
// DETACH
// -----------------------------------------------------------------------
fn vdisk_detach(args: &[&str], ctx: &mut Context) {
    let _ = ctx;

    if args.is_empty() {
        println!("Usage: vdisk detach <nbd-device | file | all>");
        println!("  Examples:");
        println!("    vdisk detach /dev/nbd0");
        println!("    vdisk detach all");
        return;
    }

    if args[0].to_lowercase() == "all" {
        detach_all_nbd();
        return;
    }

    // Accept either /dev/nbdN directly or try to find by filename
    let nbd_dev = if args[0].starts_with("/dev/nbd") {
        args[0].to_string()
    } else {
        // Try to find the NBD device backing this file via /sys
        match find_nbd_for_file(args[0]) {
            Some(d) => d,
            None => {
                println!("Could not find an attached NBD device for '{}'.", args[0]);
                println!("Use 'vdisk list' to see attached devices.");
                return;
            }
        }
    };

    println!("Detaching {}...", nbd_dev);

    let status = Command::new("qemu-nbd")
        .args(["--disconnect", &nbd_dev])
        .status()
        .expect("Failed to execute qemu-nbd");

    if status.success() {
        println!("Virtual disk detached: {}", nbd_dev);
    } else {
        println!("Failed to detach {}. Is it still mounted?", nbd_dev);
    }
}

fn detach_all_nbd() {
    println!("Detaching all NBD devices...");
    let mut count = 0;
    for i in 0..16 {
        let dev = format!("/dev/nbd{}", i);
        if !Path::new(&dev).exists() { continue; }
        // Only disconnect if actually connected (size > 0)
        if !nbd_is_connected(i) { continue; }

        let ok = Command::new("qemu-nbd")
            .args(["--disconnect", &dev])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if ok {
            println!("  Detached {}", dev);
            count += 1;
        }
    }
    if count == 0 {
        println!("No NBD devices were connected.");
    } else {
        println!("{} device(s) detached.", count);
    }
}

// -----------------------------------------------------------------------
// COMPACT
// -----------------------------------------------------------------------
fn vdisk_compact(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: vdisk compact <file>");
        println!("  Reclaims unused space in qcow2/vdi images.");
        println!("  Note: raw and vmdk images cannot be compacted this way.");
        return;
    }

    let file = args[0];

    if !Path::new(file).exists() {
        println!("File not found: '{}'.", file);
        return;
    }

    if !qemu_img_available() { return; }

    let fmt = detect_format(file).unwrap_or_else(|| "raw".to_string());

    if fmt == "raw" {
        println!("Raw images cannot be compacted (they have no sparse metadata).");
        println!("Convert to qcow2 first: vdisk create new.qcow2 format=qcow2 size=<MB>");
        println!("Then use dd or cp --sparse to copy the contents.");
        return;
    }

    println!("Compacting '{}' ({})...", file, fmt.to_uppercase());
    println!("This may take a while for large images.");

    // qemu-img convert with compression re-packs the image
    let tmp = format!("{}.compact_tmp", file);
    let qemu_fmt = to_qemu_format(&fmt);

    let status = Command::new("qemu-img")
        .args(["convert", "-O", qemu_fmt, "-c", file, &tmp])
        .status()
        .expect("Failed to execute qemu-img");

    if !status.success() {
        println!("Compact failed.");
        let _ = std::fs::remove_file(&tmp);
        return;
    }

    // Replace original with compacted version
    if std::fs::rename(&tmp, file).is_ok() {
        let orig_size = std::fs::metadata(file).map(|m| m.len()).unwrap_or(0);
        println!("Compact complete. New size: {} MB", orig_size / 1_048_576);
    } else {
        println!("Compact done but failed to replace original. Compacted file: {}", tmp);
    }
}

// -----------------------------------------------------------------------
// EXPAND
// -----------------------------------------------------------------------
fn vdisk_expand(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: vdisk expand <file> size=<MB>");
        return;
    }

    let file = args[0];
    let mut new_size_mb: Option<u64> = None;

    for arg in args.iter().skip(1) {
        if let Some(v) = arg.strip_prefix("size=") {
            match v.parse::<u64>() {
                Ok(n) => new_size_mb = Some(n),
                Err(_) => { println!("Invalid size: '{}'.", v); return; }
            }
        }
    }

    let new_size_mb = match new_size_mb {
        Some(s) => s,
        None => { println!("size= is required. Example: size=20480"); return; }
    };

    if !Path::new(file).exists() {
        println!("File not found: '{}'.", file);
        return;
    }

    if !qemu_img_available() { return; }

    let size_arg = format!("{}M", new_size_mb);

    println!("Expanding '{}' to {} MB...", file, new_size_mb);
    println!("Note: this only grows the image file. You still need to resize");
    println!("the partition and filesystem inside it separately.");

    let status = Command::new("qemu-img")
        .args(["resize", file, &size_arg])
        .status()
        .expect("Failed to execute qemu-img");

    if status.success() {
        println!("Virtual disk expanded to {} MB.", new_size_mb);
    } else {
        println!("Failed to expand virtual disk.");
    }
}

// -----------------------------------------------------------------------
// INFO
// -----------------------------------------------------------------------
fn vdisk_info(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: vdisk info <file>");
        return;
    }

    let file = args[0];

    if !Path::new(file).exists() {
        println!("File not found: '{}'.", file);
        return;
    }

    if !qemu_img_available() { return; }

    println!("Virtual disk info: {}", file);
    println!();

    // qemu-img info prints nicely formatted output, just pass it through
    let _ = Command::new("qemu-img")
        .args(["info", file])
        .status();
}

// -----------------------------------------------------------------------
// LIST
// -----------------------------------------------------------------------
fn vdisk_list() {
    println!("Attached NBD virtual disks:");
    println!();
    println!("  {:<12}  {:<8}  {}", "Device", "Size", "Backing file");
    println!("  {:<12}  {:<8}  {}", "-".repeat(12), "-------", "-".repeat(30));

    let mut found = false;
    for i in 0..16 {
        let dev = format!("/dev/nbd{}", i);
        if !Path::new(&dev).exists() { continue; }
        if !nbd_is_connected(i) { continue; }

        found = true;
        let size = read_sys_nbd_size(i);
        let backing = read_sys_nbd_backing(i).unwrap_or_else(|| "(unknown)".to_string());
        println!("  {:<12}  {:<8}  {}", dev, size, backing);
    }

    if !found {
        println!("  (no NBD devices currently attached)");
    }

    println!();
    println!("Supported image formats: qcow2, raw, vdi, vmdk, vhd, hdd");
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

fn is_valid_format(fmt: &str) -> bool {
    FORMATS.iter().any(|(f, _)| *f == fmt)
}

fn to_qemu_format(fmt: &str) -> &'static str {
    match fmt {
        "vhd"  => "vpc",       // qemu-img calls VHD "vpc"
        "hdd"  => "parallels", // qemu-img calls Parallels "parallels"
        "qcow2"=> "qcow2",
        "raw"  => "raw",
        "vdi"  => "vdi",
        "vmdk" => "vmdk",
        _      => "raw",
    }
}

fn qemu_img_available() -> bool {
    if which::which("qemu-img").is_err() {
        println!("qemu-img not found. Install it with: pacman -S qemu-img");
        false
    } else {
        true
    }
}

/// Detect format of an existing image file using qemu-img info
fn detect_format(file: &str) -> Option<String> {
    let output = Command::new("qemu-img")
        .args(["info", "--output=json", file])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Parse "format": "qcow2" from the JSON manually (avoid pulling in serde for this)
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("\"format\"") {
            if let Some(val) = line.split(':').nth(1) {
                let fmt = val.trim().trim_matches('"').trim_matches(',').to_lowercase();
                // Map qemu internal names back to user-facing names
                return Some(match fmt.as_str() {
                    "vpc"       => "vhd".to_string(),
                    "parallels" => "hdd".to_string(),
                    other       => other.to_string(),
                });
            }
        }
    }
    None
}

fn find_free_nbd() -> Option<String> {
    for i in 0..16 {
        let dev = format!("/dev/nbd{}", i);
        if !Path::new(&dev).exists() { continue; }
        if !nbd_is_connected(i) {
            return Some(dev);
        }
    }
    None
}

fn nbd_is_connected(n: u32) -> bool {
    // /sys/block/nbdN/size is 0 when disconnected
    let path = format!("/sys/block/nbd{}/size", n);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|sz| sz > 0)
        .unwrap_or(false)
}

fn read_sys_nbd_size(n: u32) -> String {
    let path = format!("/sys/block/nbd{}/size", n);
    let sectors = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let mb = sectors * 512 / 1_048_576;
    format!("{} MB", mb)
}

fn read_sys_nbd_backing(n: u32) -> Option<String> {
    // /sys/block/nbdN/backend (not always present)
    let path = format!("/sys/block/nbd{}/backend", n);
    std::fs::read_to_string(&path).ok().map(|s| s.trim().to_string())
}

fn find_nbd_for_file(file: &str) -> Option<String> {
    for i in 0..16 {
        if let Some(backing) = read_sys_nbd_backing(i) {
            if backing.contains(file) {
                return Some(format!("/dev/nbd{}", i));
            }
        }
    }
    None
}
