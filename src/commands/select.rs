use crate::context::Context;
use crate::commands::list::get_disks;

pub fn run(args: &[&str], ctx: &mut Context) {
    if args.len() != 2 || args[0] != "disk" {
        println!("Usage: select disk <number>");
        return;
    }

    let disk_index = match args[1].parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid disk number.");
            return;
        }
    };

    let disks = get_disks();

    if let Some(disk) = disks.into_iter().find(|d| d.index == disk_index) {
        println!("Disk {} is now the selected disk.", disk_index);
        println!("-> {}", disk.path);
        ctx.selected_disk = Some(disk);
    } else {
        println!("Disk not found.");
    }
}
