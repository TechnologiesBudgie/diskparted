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
use crate::commands::active::get_disk_label;

/// SETID — change the partition type field for the selected partition.
///
/// DiskPart syntax:
///   set id={ <byte> | <GUID> } [override] [noerr]
///
/// On Linux:
///   GPT disks  — uses `sgdisk --typecode=<part>:<GUID>`
///   MBR disks  — uses `sfdisk --part-type <disk> <part> <hex_byte>`
///
/// Well-known type IDs (for convenience):
///   linux-data  = 8300 (GPT) / 83 (MBR)
///   linux-swap  = 8200 (GPT) / 82 (MBR)
///   efi         = ef00 (GPT) / ef (MBR)
///   msdata      = ebd0a0a2-b9e5-4433-87c0-68b6b72699c7
///   msr         = e3c9e316-0b5c-4db8-817d-f92df00215ae
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
        println!("Usage: set id={{<hex_byte>|<GUID>|<alias>}} [override] [noerr]");
        println!("Aliases: linux-data, linux-swap, efi, msdata, msr, ntfs, bios-boot");
        return;
    }

    let mut id_val: Option<String> = None;
    let mut noerr = false;
    // override flag: force dismount before changing (we do this automatically on Linux)

    for arg in args {
        if let Some(val) = arg.strip_prefix("id=") {
            id_val = Some(val.to_string());
        } else if arg.eq_ignore_ascii_case("override") {
            // We always dismount silently, so this is a no-op
        } else if arg.eq_ignore_ascii_case("noerr") {
            noerr = true;
        } else {
            println!("Unknown parameter: '{}'. Ignoring.", arg);
        }
    }

    let id_raw = match id_val {
        Some(v) => v,
        None => {
            println!("Error: id= parameter is required.");
            return;
        }
    };

    // Resolve friendly aliases to canonical values
    let id = resolve_alias(&id_raw);

    let label = get_disk_label(&disk_path);
    let is_gpt = label.as_deref() == Some("gpt");

    // Attempt silent unmount before changing type
    let _ = Command::new("sudo").args(["umount", &partition.path]).output();

    if is_gpt {
        // For GPT: use sgdisk --typecode=<partnum>:<GUID>
        // sgdisk accepts both short codes (8300) and full GUIDs
        let typecode_arg = format!("{}:{}", partition.index, id);
        println!("Setting GPT type code for partition {} to '{}'...", partition.index, id);

        let status = Command::new("sudo")
            .args(["sgdisk", "--typecode", &typecode_arg, &disk_path])
            .status()
            .expect("Failed to execute sgdisk");

        if status.success() {
            println!("Partition type set successfully.");
        } else {
            println!("Failed to set partition type.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    } else {
        // For MBR: use sfdisk --part-type <disk> <partnum> <hex>
        // Strip leading 0x if present
        let hex = id.trim_start_matches("0x").trim_start_matches("0X").to_uppercase();
        println!("Setting MBR type byte for partition {} to 0x{}...", partition.index, hex);

        let status = Command::new("sudo")
            .args(["sfdisk", "--part-type", &disk_path, &partition.index.to_string(), &hex])
            .status()
            .expect("Failed to execute sfdisk");

        if status.success() {
            println!("Partition type set successfully.");
        } else {
            println!("Failed to set partition type.");
            if noerr { println!("(noerr: continuing despite error)"); }
        }
    }
}

/// Resolve human-friendly aliases to the raw type ID expected by sgdisk/sfdisk.
fn resolve_alias(id: &str) -> String {
    match id.to_lowercase().as_str() {
        "linux-data"  => "8300".to_string(),
        "linux-swap"  => "8200".to_string(),
        "efi"         => "ef00".to_string(),
        "msdata"      => "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7".to_string(),
        "msr"         => "e3c9e316-0b5c-4db8-817d-f92df00215ae".to_string(),
        "ntfs"        => "0700".to_string(),
        "bios-boot"   => "ef02".to_string(),
        other         => other.to_string(),
    }
}
