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

//! ENCRYPT  [!]
//!
//! Manage LUKS2 encryption on the selected partition.
//! DiskParted Linux extension — no equivalent in Windows DiskPart.
//!
//! Syntax
//! ------
//!   encrypt setup  [name=<label>]
//!   encrypt open   [name=<label>]
//!   encrypt close  [name=<label>]
//!   encrypt status
//!
//! Subcommands
//! -----------
//!   setup [name=<x>]   Format the selected partition as a LUKS2 container.
//!                      DESTRUCTIVE — all existing data will be lost.
//!                      Prompts interactively for a passphrase.
//!                      name defaults to "luks-<devname>" if omitted.
//!
//!   open  [name=<x>]   Unlock an existing LUKS partition and create a
//!                      device-mapper node at /dev/mapper/<name>.
//!
//!   close [name=<x>]   Close an open LUKS mapping, removing the
//!                      /dev/mapper/<name> node.
//!
//!   status             Show cryptsetup luksDump for the selected partition.
//!
//! Requires
//! --------
//!   cryptsetup  (pacman -S cryptsetup)

use std::process::Command;
use crate::context::Context;
use crate::utils::confirm;

pub fn run(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        print_usage();
        return;
    }

    let part_dev = match ctx.selected_partition.as_ref() {
        Some(p) => p.clone(),
        None => {
            eprintln!("There is no partition selected.");
            eprintln!("Use 'select partition <n>' first.");
            return;
        }
    };

    if !tool_available("cryptsetup", "cryptsetup") {
        return;
    }

    // Parse optional name= from the remaining arguments.
    let name_arg: Option<String> = args
        .iter()
        .skip(1)
        .find(|a| a.to_lowercase().starts_with("name="))
        .map(|a| a[5..].to_string());

    let default_name = format!(
        "luks-{}",
        (*part_dev).trim_start_matches("/dev/")
    );
    let name = name_arg.as_deref().unwrap_or(&default_name);

    match args[0].to_lowercase().as_str() {
        "setup"  => encrypt_setup(&*part_dev, name),
        "open"   => encrypt_open(&*part_dev, name),
        "close"  => encrypt_close(name),
        "status" => encrypt_status(&*part_dev),
        other => {
            eprintln!("Unknown subcommand: '{}'. Valid: setup, open, close, status.", other);
            print_usage();
        }
    }
}

fn encrypt_setup(dev: &str, name: &str) {
    println!("ENCRYPT SETUP");
    println!("  Partition : {}", dev);
    println!("  Name      : {}", name);
    println!();
    println!(
        "  WARNING: This will PERMANENTLY ERASE all data on {}.",
        dev
    );
    println!("           The partition will be formatted as a LUKS2 container.");
    println!();

    if !confirm("Are you absolutely sure you want to encrypt this partition?") {
        println!("No change was made.");
        return;
    }

    println!("You will now be prompted to enter and confirm a passphrase.");
    println!();

    match Command::new("cryptsetup")
        .args(["luksFormat", "--type", "luks2", "--verify-passphrase", dev])
        .status()
    {
        Ok(s) if s.success() => {
            println!();
            println!("LUKS2 container created on {}.", dev);
            println!("Open it with:  encrypt open name={}", name);
        }
        Ok(_) => eprintln!("cryptsetup luksFormat failed."),
        Err(e) => eprintln!("Failed to run cryptsetup: {}", e),
    }
}

fn encrypt_open(dev: &str, name: &str) {
    println!(
        "Opening LUKS partition {} as /dev/mapper/{}...",
        dev, name
    );

    match Command::new("cryptsetup").args(["open", dev, name]).status() {
        Ok(s) if s.success() => {
            println!("Opened: /dev/mapper/{}", name);
            println!(
                "You can now format or mount /dev/mapper/{}",
                name
            );
        }
        Ok(_) => eprintln!(
            "cryptsetup open failed. \
             Check that the passphrase is correct."
        ),
        Err(e) => eprintln!("Failed to run cryptsetup: {}", e),
    }
}

fn encrypt_close(name: &str) {
    println!("Closing LUKS mapping {}...", name);

    match Command::new("cryptsetup").args(["close", name]).status() {
        Ok(s) if s.success() => println!("Closed /dev/mapper/{}", name),
        Ok(_) => eprintln!(
            "cryptsetup close failed. \
             Ensure the mapping is not in use (unmount first)."
        ),
        Err(e) => eprintln!("Failed to run cryptsetup: {}", e),
    }
}

fn encrypt_status(dev: &str) {
    println!("LUKS status for {}:", dev);
    println!();
    let _ = Command::new("cryptsetup").args(["luksDump", dev]).status();
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
    println!("Syntax:  encrypt setup  [name=<label>]");
    println!("         encrypt open   [name=<label>]");
    println!("         encrypt close  [name=<label>]");
    println!("         encrypt status");
    println!();
    println!("  setup    Format partition as LUKS2 — DESTRUCTIVE");
    println!("  open     Unlock and expose as /dev/mapper/<label>");
    println!("  close    Close an open LUKS mapping");
    println!("  status   Show LUKS header information");
}
