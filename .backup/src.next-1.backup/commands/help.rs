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
    println!("Copyright 2026 DiskParted Team. Licensed under GNU GPLv3.");
    println!("DiskParted version 0.1.2");
    println!();
    println!("Microsoft DiskPart-compatible commands:");
    println!();

    // Legend
    println!("  [+] Implemented    [-] Not on Linux    [~] Partial/stub");
    println!();

    let commands: &[(&str, &str, &str)] = &[
        // cmd            status  description
        ("ACTIVE",        "[+]", "Mark selected partition as active (sets boot/esp flag)"),
        ("ADD",           "[+]", "Add mirror disk to volume via mdadm RAID-1"),
        ("ASSIGN",        "[+]", "Assign mount point to selected volume"),
        ("ATTACH",        "[-]", "Attach a VHD virtual disk (Windows VDS only)"),
        ("ATTRIBUTES",    "[-]", "Disk/volume attribute flags (Windows VDS only)"),
        ("AUTOMOUNT",     "[+]", "Enable/disable automounting (fstab / udev)"),
        ("BREAK",         "[+]", "Break mdadm RAID-1 mirror"),
        ("CLEAN",         "[+]", "Remove all partition information from selected disk"),
        ("COMPACT",       "[-]", "Shrink a VHD file (Windows VDS only)"),
        ("CONVERT",       "[+]", "Convert disk MBR↔GPT, or upgrade ext2/3/4 in-place"),
        ("CREATE",        "[+]", "Create a partition (primary/efi/msr/extended/logical)"),
        ("DELETE",        "[+]", "Delete a partition or volume"),
        ("DETAIL",        "[+]", "Display properties of selected disk/partition/volume"),
        ("DETACH",        "[-]", "Detach a VHD virtual disk (Windows VDS only)"),
        ("EXIT",          "[+]", "Exit diskparted"),
        ("EXPAND",        "[-]", "Expand a VHD file (Windows VDS only)"),
        ("EXTEND",        "[ ]", "Extend a volume (not yet implemented)"),
        ("FILESYSTEMS",   "[+]", "Display current and supported filesystems for selected volume"),
        ("FORMAT",        "[+]", "Format the selected partition"),
        ("GPT",           "[+]", "Assign GPT attribute bits to selected partition"),
        ("HELP",          "[+]", "Display this help information"),
        ("IMPORT",        "[+]", "Import foreign LVM volume group or ZFS pool"),
        ("INACTIVE",      "[+]", "Clear boot/esp flag from selected partition"),
        ("LIST",          "[+]", "List disks, partitions, or volumes"),
        ("MERGE",         "[-]", "Merge VHD differencing disk (Windows Hyper-V only)"),
        ("OFFLINE",       "[+]", "Take selected disk or volume offline (unmount/spin-down)"),
        ("ONLINE",        "[+]", "Bring selected disk or volume online (spin-up/mount)"),
        ("RECOVER",       "[+]", "Refresh disk state, run fsck, reassemble RAID"),
        ("REM",           "[+]", "Comment line — no-op (for script compatibility)"),
        ("REMOVE",        "[+]", "Remove mount point from selected volume"),
        ("REPAIR",        "[+]", "Repair RAID/LVM/ZFS volume on selected disk"),
        ("RESCAN",        "[+]", "Rescan disks (partprobe)"),
        ("RETAIN",        "[~]", "Retain partition (Windows LDM only — use 'active' instead)"),
        ("SAN",           "[~]", "SAN policy (Windows VDS only — use 'automount' instead)"),
        ("SELECT",        "[+]", "Select a disk, partition, or volume"),
        ("SET ID",        "[+]", "Change partition type field (GPT GUID or MBR byte)"),
        ("SHRINK",        "[ ]", "Shrink a volume (not yet implemented)"),
        ("UNIQUEID",      "[+]", "Display or set disk GUID (GPT) or MBR signature"),
    ];

    for (cmd, status, desc) in commands {
        println!("  {status} {cmd:<14}  {desc}");
    }

    println!();
    println!("  [-] = Not implementable on Linux (Windows VDS/LDM/Hyper-V concepts).");
    println!("        Running these commands will explain why and suggest alternatives.");
    println!("  [~] = Partially supported — see command output for details.");
    println!("  [ ] = Planned but not yet implemented.");
    println!();
    println!("Reference: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/diskpart");
}
