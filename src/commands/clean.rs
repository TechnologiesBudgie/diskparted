use crate::context::Context;
use crate::utils::confirm;

pub fn run(_args: &[&str], ctx: &mut Context) {
    match &ctx.selected_disk {
        Some(disk) => {
            if !confirm(&format!(
                "This will erase ALL partitions on {}. Continue?",
                disk.path
            )) {
                println!("Operation cancelled.");
                return;
            }

            println!("Cleaning {}...", disk.path);

            // Real implementation will go here later
            println!("(Simulated) All partitions removed.");
        }
        None => {
            println!("No disk selected.");
        }
    }
}