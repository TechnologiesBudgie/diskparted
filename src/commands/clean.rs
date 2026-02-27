use std::process::Command;
use crate::context::Context;
use crate::utils::confirm;

pub fn run(_args: &[&str], ctx: &mut Context) {
    // Ensure a disk is selected
    let disk = match &ctx.selected_disk {
        Some(d) => d,
        None => {
            println!("No disk selected. Use 'select disk <n>' first.");
            return;
        }
    };

    // Ask user to type the disk path for confirmation
    println!("WARNING: This will erase ALL partitions on {}!", disk.path);
    println!("Type the disk path '{}' to confirm:", disk.path);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim() != disk.path {
        println!("Confirmation failed. Operation cancelled.");
        return;
    }

    println!("Cleaning disk {}...", disk.path);

    // wipe filesystem signatures
    let status = Command::new("wipefs")
        .args(&["-a", &disk.path])
        .status()
        .expect("Failed to execute wipefs");

    if !status.success() {
        println!("Failed to wipe filesystem signatures.");
        return;
    }

    // remove partition table (GPT/MBR)
    let status = Command::new("sgdisk")
        .args(&["--zap-all", &disk.path])
        .status();

    match status {
        Ok(s) if s.success() => println!("Disk {} cleaned successfully.", disk.path),
        _ => println!("Failed to fully clean partition table (you may need root)."),
    }
}