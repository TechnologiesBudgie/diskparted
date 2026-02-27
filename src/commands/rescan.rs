use std::process::Command;
use crate::context::Context;

/// Run the RESCAN command
pub fn run(_args: &[&str], _ctx: &mut Context) {
    println!("Rescanning disks...");

    let status = Command::new("sudo")
        .arg("partprobe")  // update kernel partition table
        .status()
        .expect("Failed to execute partprobe");

    if status.success() {
        println!("Disk rescan completed successfully.");
    } else {
        println!("Disk rescan failed.");
    }
}
