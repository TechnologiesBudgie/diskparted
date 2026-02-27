use crate::context::Context;
use std::process::Command;
use std::path::Path;
use std::io::{self, Write};

/// Run the `shrink` command
/// Usage: shrink volume <size>
pub fn run(args: &[&str], ctx: &mut Context) {
    if args.len() < 2 || args[0].to_lowercase() != "volume" {
        println!("Usage:");
        println!("  shrink volume <size>");
        return;
    }

    let size_arg = args[1];

    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => {
            println!("No partition selected. Use `select partition <n>` first.");
            return;
        }
    };

    println!("WARNING: You are about to shrink partition {} by {}.", partition.path, size_arg);
    if !confirm("Do you want to continue? (y/N): ") {
        println!("Aborted.");
        return;
    }

    shrink_volume(size_arg, ctx);
}

fn shrink_volume(size_arg: &str, ctx: &mut Context) {
    let partition = ctx.selected_partition.as_ref().unwrap();
    let part_path = &partition.path;

    if !Path::new(part_path).exists() {
        println!("Partition {} does not exist.", part_path);
        return;
    }

    // Detect filesystem type
    let fs_type = detect_filesystem(part_path);
    match fs_type.as_deref() {
        Some("ext4") | Some("ext3") | Some("ext2") => {
            // Try to unmount, ignore "not mounted"
            let umount_status = Command::new("umount")
                .arg(part_path)
                .output();

            if let Ok(output) = umount_status {
                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                if output.status.success() || stderr.contains("not mounted") {
                    // fine
                } else {
                    println!("Failed to unmount {}. Is it in use?", part_path);
                    return;
                }
            } else {
                println!("Failed to run umount command.");
                return;
            }

            println!("Warning: Shrinking a partition can cause data loss, are you sure you want to continue?");
            if !confirm("Proceed? (y/N): ") {
                println!("Aborted.");
                return;
            }

            // Call resize2fs
            let status = Command::new("resize2fs")
                .arg(part_path)
                .arg(size_arg)
                .status();

            match status {
                Ok(s) if s.success() => println!("Filesystem shrunk successfully."),
                Ok(_) => println!("Failed to shrink filesystem."),
                Err(e) => println!("Error executing resize2fs: {}", e),
            }
        }
        Some("ntfs") => {
            println!("NTFS shrinking not implemented yet.");
        }
        Some(other) => {
            println!("Filesystem type {} not supported for automatic shrinking.", other);
        }
        None => {
            println!("Unable to detect filesystem type.");
        }
    }
}

/// Detect filesystem type using `lsblk -no FSTYPE <partition>`
fn detect_filesystem(part_path: &str) -> Option<String> {
    let output = Command::new("lsblk")
        .args(["-no", "FSTYPE", part_path])
        .output()
        .ok()?;

    let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fs.is_empty() {
        None
    } else {
        Some(fs)
    }
}

/// Simple yes/no prompt
fn confirm(prompt: &str) -> bool {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    } else {
        false
    }
}
