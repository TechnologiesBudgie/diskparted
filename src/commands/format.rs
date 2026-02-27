use std::process::Command;
use std::path::Path;
use std::io::{self, Write};
use crate::context::Context;
use which::which;

/// Supported filesystems
const SUPPORTED_FS: &[&str] = &[
    "fat", "fat16", "fat32", "vfat", "hfs+", "apfs", "ntfs",
    "ext1", "ext2", "ext3", "ext4", "xfs", "btrfs",
    "exfat", "bcachefs", "hfs", "f2fs", "jfs", "linux-swap",
    "minix", "nilfs2", "reiser4", "reiserfs", "udf"
];

pub fn run(args: &[&str], ctx: &mut Context) {
    let partition = match &ctx.selected_partition {
        Some(p) => p,
        None => {
            println!("No partition selected. Use 'select partition <n>' first.");
            return;
        }
    };

    if !Path::new(&partition.path).exists() {
        println!("Partition {} does not exist (maybe USB removed?).", partition.path);
        ctx.selected_partition = None;
        return;
    }

    let mut fs_type: Option<String> = None;
    let mut quick = false;

    for arg in args {
        if arg.starts_with("fs=") {
            fs_type = Some(arg.trim_start_matches("fs=").to_lowercase());
        } else if *arg == "quick" {
            quick = true;
        }
    }

    let fs_type = match fs_type {
        Some(f) => f,
        None => {
            println!("Error: Filesystem not specified. Usage: format fs=<filesystem> [quick]");
            return;
        }
    };

    if !SUPPORTED_FS.contains(&fs_type.as_str()) {
        println!("Error: Unsupported filesystem '{}'.", fs_type);
        return;
    }

    println!("WARNING: You are about to format {} as {}{}.", 
        partition.path, fs_type, if quick { " (quick)" } else { "" });
    
    if !confirm("Do you want to continue? (y/N): ") {
        println!("Aborted.");
        return;
    }

    // Attempt to unmount silently
    let _ = Command::new("umount").arg(&partition.path).output();

    let mkfs_cmd = match fs_type.as_str() {
        "fat" | "fat16" => "mkfs.fat",
        "fat32" | "vfat" => "mkfs.fat",
        "ntfs" => "mkfs.ntfs",
        "ext1" => "mkfs.ext2",
        "ext2" => "mkfs.ext2",
        "ext3" => "mkfs.ext3",
        "ext4" => "mkfs.ext4",
        "xfs" => "mkfs.xfs",
        "btrfs" => "mkfs.btrfs",
        "exfat" => "mkfs.exfat",
        "f2fs" => "mkfs.f2fs",
        "jfs" => "mkfs.jfs",
        "linux-swap" => "mkswap",
        "udf" => "mkfs.udf",
        "hfs+" | "hfs" => "mkfs.hfsplus",
        "apfs" => "mkfs.apfs",
        "minix" => "mkfs.minix",
        "nilfs2" => "mkfs.nilfs2",
        "reiser4" | "reiserfs" => "mkfs.reiserfs",
        "bcachefs" => "mkfs.bcachefs",
        _ => { println!("Unsupported FS '{}'", fs_type); return; }
    };

    if which::which(mkfs_cmd).is_err() {
        println!("Error: '{}' not found. Install the corresponding filesystem package.", mkfs_cmd);
        return;
    }

    let mut cmd = Command::new(mkfs_cmd);
    cmd.arg(&partition.path);

    if quick {
        match fs_type.as_str() {
            "fat" | "fat16" => { cmd.arg("-F").arg("16"); }
            "fat32" | "vfat" => { cmd.arg("-F").arg("32"); }
            "ntfs" => { cmd.arg("-f"); }
            "ext1" | "ext2" | "ext3" | "ext4" => { cmd.arg("-F"); }
            "linux-swap" => { cmd.arg("-f"); }
            _ => {}
        }
    }

    match cmd.status() {
        Ok(s) if s.success() => println!("Partition {} formatted successfully.", partition.path),
        Ok(s) => println!("Failed to format {}. Exit code: {}", partition.path, s),
        Err(e) => println!("Failed to execute mkfs: {}", e),
    }
}

/// Simple yes/no confirmation
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
