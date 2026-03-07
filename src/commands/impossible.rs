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
    println!("ATTACH manages Windows VHD/VHDX files via Windows VDS.");
    println!("On Linux, use the 'vdisk' command instead:");
    println!();
    println!("  vdisk attach <file.qcow2|.raw|.vdi|.vmdk|.vhd|.hdd>");
    println!("  vdisk list");
    println!();
    println!("Supported formats: qcow2, raw, vdi, vmdk, vhd, hdd");
    println!("Requires: qemu-img  (pacman -S qemu-img)");
}

/// DETACH — detach a virtual hard disk (VHD/VHDX).
/// See ATTACH for explanation.
pub fn detach(_args: &[&str], _ctx: &mut Context) {
    println!("DETACH manages Windows VHD/VHDX files via Windows VDS.");
    println!("On Linux, use the 'vdisk' command instead:");
    println!();
    println!("  vdisk detach <file | /dev/nbdN | all>");
    println!("  vdisk list");
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
    println!("COMPACT manages Windows VHD files via Windows VDS.");
    println!("On Linux, use the 'vdisk' command instead:");
    println!();
    println!("  vdisk compact <file.qcow2|.vdi|...>");
    println!();
    println!("Reclaims unused space by re-packing the image with qemu-img.");
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
    println!("EXPAND manages Windows VHD files via Windows VDS.");
    println!("On Linux, use the 'vdisk' command instead:");
    println!();
    println!("  vdisk expand <file> size=<MB>");
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
    println!("MERGE merges a Hyper-V differencing VHD with its parent.");
    println!("This is a Windows Hyper-V-only concept with no direct Linux equivalent.");
    println!();
    println!("For qcow2 backing files on Linux:");
    println!("  qemu-img commit <child.qcow2>      — commit child changes into parent");
    println!("  qemu-img rebase -b '' <child.qcow2> — flatten into standalone image");
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
