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

//! # Unimplementable DiskPart Commands
//!
//! The commands in this file are part of the Microsoft DiskPart specification
//! but **cannot be meaningfully implemented on Linux**, even with workarounds.
//! Each function explains why and, where possible, suggests a Linux alternative.
//!
//! Commands covered here:
//!   - ATTACH / DETACH   (VHD/VHDX virtual disk management — Windows VDS only)
//!   - COMPACT           (VHD space reclamation — Windows VDS only)
//!   - EXPAND            (VHD expansion — Windows VDS only)
//!   - MERGE             (VHD differencing disk merge — Windows VDS only)
//!   - ATTRIBUTES        (Windows disk/volume attribute flags — no Linux mapping)
//!   - CONVERT DYNAMIC   (Windows LDM dynamic disks — Linux has no equivalent)
//!   - CONVERT BASIC     (reverse of the above — same reason)

use crate::context::Context;

// ---------------------------------------------------------------------------
// ATTACH / DETACH
// ---------------------------------------------------------------------------

/// ATTACH — attach a virtual hard disk (VHD/VHDX).
///
/// This command mounts a Windows VHD or VHDX file as a disk.
/// VHD/VHDX is a Microsoft virtual disk format managed by the Windows
/// Virtual Disk Service (VDS). Linux cannot natively attach VHD files
/// through a DiskPart-compatible interface.
///
/// Linux alternative:
///   Raw/QCOW2/VMDK images can be attached with:
///     sudo losetup -fP <image.img>
///   VHD files specifically can be converted and attached with:
///     qemu-img convert -f vpc -O raw input.vhd output.img
///     sudo losetup -fP output.img
pub fn attach(_args: &[&str], _ctx: &mut Context) {
    println!("ATTACH is not supported on Linux.");
    println!();
    println!("ATTACH mounts a Windows VHD/VHDX virtual disk file. This requires the");
    println!("Windows Virtual Disk Service (VDS), which does not exist on Linux.");
    println!();
    println!("Linux alternatives:");
    println!("  Raw images    : sudo losetup -fP <image.img>");
    println!("  VHD images    : qemu-img convert -f vpc -O raw input.vhd output.img");
    println!("                  sudo losetup -fP output.img");
    println!("  QCOW2/VMDK    : sudo qemu-nbd --connect=/dev/nbd0 <image>");
}

/// DETACH — detach a virtual hard disk (VHD/VHDX).
/// See ATTACH for explanation.
pub fn detach(_args: &[&str], _ctx: &mut Context) {
    println!("DETACH is not supported on Linux.");
    println!();
    println!("DETACH unmounts a Windows VHD/VHDX virtual disk. This requires the");
    println!("Windows Virtual Disk Service (VDS), which does not exist on Linux.");
    println!();
    println!("Linux alternatives:");
    println!("  Loop devices  : sudo losetup -d /dev/loopN");
    println!("  NBD devices   : sudo qemu-nbd --disconnect /dev/nbd0");
}

// ---------------------------------------------------------------------------
// COMPACT
// ---------------------------------------------------------------------------

/// COMPACT — reduce the physical size of a dynamically expanding VHD.
///
/// COMPACT reclaims unused space inside a dynamically expanding VHD file,
/// shrinking it to the minimum size needed. This is a VHD-specific operation
/// with no Linux equivalent at the DiskPart level.
///
/// Linux alternative:
///   qemu-img convert -O qcow2 input.qcow2 compacted.qcow2
pub fn compact(_args: &[&str], _ctx: &mut Context) {
    println!("COMPACT is not supported on Linux.");
    println!();
    println!("COMPACT reclaims unused space in a Windows dynamically expanding VHD.");
    println!("This requires the Windows Virtual Disk Service (VDS).");
    println!();
    println!("Linux alternative (QCOW2 compaction):");
    println!("  qemu-img convert -O qcow2 input.qcow2 compacted.qcow2");
}

// ---------------------------------------------------------------------------
// EXPAND
// ---------------------------------------------------------------------------

/// EXPAND — expand the maximum size of a VHD.
///
/// Increases the maximum size of a virtual hard disk file.
/// VHD expansion is handled by Windows VDS / Hyper-V tooling.
///
/// Linux alternative:
///   qemu-img resize <image> +<size>
pub fn expand(_args: &[&str], _ctx: &mut Context) {
    println!("EXPAND is not supported on Linux.");
    println!();
    println!("EXPAND increases the maximum size of a Windows VHD virtual disk.");
    println!("This requires the Windows Virtual Disk Service (VDS).");
    println!();
    println!("Linux alternative:");
    println!("  qemu-img resize <image.qcow2> +10G");
    println!("  (then resize the partition and filesystem inside)");
}

// ---------------------------------------------------------------------------
// MERGE
// ---------------------------------------------------------------------------

/// MERGE — merge a child differencing VHD with its parent.
///
/// MERGE collapses a VHD differencing chain — a Windows Hyper-V concept
/// where snapshots are stored as delta disks chained to a parent VHD.
/// There is no Linux equivalent at the partition manager level.
///
/// Linux alternative (QCOW2 backing files):
///   qemu-img commit child.qcow2   (merges child changes into parent)
pub fn merge(_args: &[&str], _ctx: &mut Context) {
    println!("MERGE is not supported on Linux.");
    println!();
    println!("MERGE collapses a Windows VHD differencing disk chain into its parent.");
    println!("This is a Hyper-V/VDS concept with no Linux equivalent.");
    println!();
    println!("Linux alternative (QCOW2 backing files):");
    println!("  qemu-img commit child.qcow2   — merges child changes into its backing file");
}

// ---------------------------------------------------------------------------
// ATTRIBUTES
// ---------------------------------------------------------------------------

/// ATTRIBUTES — display or set disk/volume attributes.
///
/// DiskPart's ATTRIBUTES command manages Windows-specific flags:
///   - Disk: read-only, offline
///   - Volume: hidden, no default drive letter, shadow copy, read-only
///
/// The disk read-only flag maps to the Linux `blockdev --setro` / `--setrw`
/// command, but the volume-level flags (hidden, no drive letter, shadow copy)
/// are Windows NTFS/VDS concepts with no Linux equivalent.
///
/// The GPT partition-level read-only attribute can be set with 'gpt attributes='.
pub fn attributes(_args: &[&str], _ctx: &mut Context) {
    println!("ATTRIBUTES is not fully supported on Linux.");
    println!();
    println!("Windows disk/volume attributes (hidden, no-default-drive-letter,");
    println!("shadow-copy) are Windows VDS/NTFS concepts with no Linux equivalent.");
    println!();
    println!("Partial Linux equivalents:");
    println!("  Disk read-only  : sudo blockdev --setro /dev/<disk>");
    println!("  Disk read-write : sudo blockdev --setrw /dev/<disk>");
    println!("  GPT attributes  : use 'gpt attributes=<n>' in diskparted");
}
