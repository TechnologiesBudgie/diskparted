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
pub fn run() {
    println!("Copyright 2026 DiskParted Team. Licensed under GNU GPLv3.\n");
    println!("DiskParted version 0.1.1\n");
    println!("Microsoft DiskPart-compatible command list:\n");

    println!("ACTIVE              - Mark the selected partition as active (not implemented yet)");
    println!("ADD                 - Add a mirror to a simple volume (not implemented yet)");
    println!("ASSIGN              - Assign a drive letter or mount point (not implemented yet)");
    println!("ATTACH              - Attach a virtual disk (not implemented yet)");
    println!("ATTRIBUTES          - Display or set disk/volume attributes (not implemented yet)");
    println!("AUTOMOUNT           - Enable/disable automatic mounting (not implemented yet)");
    println!("BREAK               - Break a mirror set (not implemented yet)");
    println!("CLEAN               - Remove all partition information from the disk");
    println!("COMPACT             - Reduce the physical size of a VHD (not implemented yet)");
    println!("CONVERT             - Convert disk format (MBR/GPT) (not implemented yet)");
    println!("CREATE              - Create a partition or volume");
    println!("DELETE              - Delete a partition or volume");
    println!("DETAIL              - Display properties of selected disk/volume (not implemented yet)");
    println!("DETACH              - Detach a virtual disk (not implemented yet)");
    println!("EXIT                - Exit diskparted");
    println!("EXPAND              - Expand a volume (not implemented yet)");
    println!("EXTEND              - Extend a volume (not implemented yet)");
    println!("FILESYSTEMS         - Display supported file systems (not implemented yet)");
    println!("FORMAT              - Format a volume (currently being implemented)");
    println!("GPT                 - Assign GPT attributes (not implemented yet)");
    println!("HELP                - Display this help information");
    println!("IMPORT              - Import a foreign disk group (not implemented yet)");
    println!("INACTIVE            - Mark partition as inactive (not implemented yet)");
    println!("LIST                - Display list of disks or volumes");
    println!("MERGE               - Merge child disk with parent (not implemented yet)");
    println!("ONLINE              - Bring disk online (not implemented yet)");
    println!("OFFLINE             - Take disk offline (not implemented yet)");
    println!("RECOVER             - Refresh disk state (not implemented yet)");
    println!("REM                 - Comment (not implemented yet)");
    println!("REMOVE              - Remove drive letter (not implemented yet)");
    println!("RESCAN              - Rescan disks");
    println!("RETAIN              - Place retain partition (not implemented yet)");
    println!("SAN                 - Display or set SAN policy (not implemented yet)");
    println!("SELECT              - Select a disk or a partition");
    println!("SETID               - Change partition type (not implemented yet)");
    println!("SHRINK              - Shrink a volume (currently being implemented)");
    println!("UNIQUEID            - Display or set disk GUID (not implemented yet)");

    println!("\nCurrently implemented commands:");
    println!("  create");
    println!("  clean");
    println!("  delete partition");
    println!("  exit");
    println!("  format");
    println!("  help");
    println!("  list disk");
    println!("  list volume");
    println!("  rescan");
    println!("  select disk <n>");
    println!("  select partition <n>");
    println!("  shrink volume <MB>");
}
