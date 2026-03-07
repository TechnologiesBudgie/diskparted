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

// ── Existing commands ──────────────────────────────────────────────────────
pub mod clean;
pub mod create;
pub mod delete;
pub mod filesystems;
pub mod format;
pub mod help;
pub mod list;
pub mod repair;
pub mod rescan;
pub mod select;   // handles select disk / partition / volume

// ── New commands ───────────────────────────────────────────────────────────
pub mod active;       // ACTIVE   — mark partition as bootable (boot/esp flag)
pub mod add;          // ADD      — add mirror disk via mdadm RAID-1
pub mod assign;       // ASSIGN   — mount partition (drive letter → /mnt/<x>)
pub mod automount;    // AUTOMOUNT — manage fstab noauto / udev automounting
pub mod break_cmd;    // BREAK    — break mdadm RAID-1 mirror
pub mod convert;      // CONVERT  — MBR↔GPT + ext2/3/4 in-place upgrade
pub mod detail;       // DETAIL   — detailed info on disk / partition / volume
pub mod gpt;          // GPT      — set GPT attribute bits via sgdisk
pub mod import_cmd;   // IMPORT   — import LVM VGs or ZFS pools
pub mod inactive;     // INACTIVE — clear boot/esp flag from partition
pub mod offline;      // OFFLINE  — unmount / spin-down disk or volume
pub mod online;       // ONLINE   — spin-up / mount disk or volume
pub mod recover;      // RECOVER  — partprobe + fsck + mdadm reassemble
pub mod rem;          // REM      — comment line (no-op)
pub mod remove;       // REMOVE   — unmount partition (reverse of assign)
pub mod retain;       // RETAIN   — stub (Windows LDM concept, not on Linux)
pub mod san;          // SAN      — stub (Windows VDS concept, not on Linux)
pub mod setid;        // SETID    — change partition type via sgdisk/sfdisk
pub mod uniqueid;     // UNIQUEID — display/set disk GUID or MBR signature

// ── Unimplementable commands ───────────────────────────────────────────────
pub mod impossible;   // ATTACH/DETACH/COMPACT/EXPAND/MERGE/ATTRIBUTES stubs

// ── Virtual disk manager ───────────────────────────────────────────────────
pub mod vdisk;        // VDISK — create/attach/detach/compact/expand/info/list
