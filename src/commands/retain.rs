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
use crate::context::Context;

/// RETAIN — place a retained partition under a simple dynamic volume.
///
/// DiskPart syntax:  retain
///
/// This command prepares a dynamic simple volume for use as a boot or system
/// volume by placing a retained partition beneath it. Dynamic volumes and the
/// retain concept are exclusive to Windows LDM (Logical Disk Manager) and
/// have no equivalent on Linux.
pub fn run(_args: &[&str], _ctx: &mut Context) {
    println!("The RETAIN command is not supported on Linux.");
    println!();
    println!("RETAIN places a retained partition under a dynamic simple volume");
    println!("so it can serve as a system/boot volume. Dynamic volumes (LDM) are");
    println!("a Windows-only concept with no Linux equivalent.");
    println!();
    println!("If you need to mark a partition as bootable on Linux, use:");
    println!("  active    — sets the boot/esp flag on the selected partition");
}
