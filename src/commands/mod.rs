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

// ── DiskPart-compatible commands ───────────────────────────────────────────
pub mod active;       // ACTIVE      — mark partition as bootable (boot/esp flag)
pub mod add;          // ADD         — add mirror disk via mdadm RAID-1
pub mod assign;       // ASSIGN      — mount partition (drive letter → /mnt/<x>)
pub mod automount;    // AUTOMOUNT   — manage fstab noauto / udev automounting
pub mod break_cmd;    // BREAK       — break mdadm RAID-1 mirror
pub mod clean;        // CLEAN       — wipefs + sgdisk --zap-all
pub mod convert;      // CONVERT     — MBR↔GPT + ext2/3/4 in-place upgrade
pub mod create;       // CREATE      — create partition via parted
pub mod delete;       // DELETE      — delete partition or volume
pub mod detail;       // DETAIL      — detailed info on disk / partition / volume
pub mod extend;       // EXTEND      — grow partition + file system
pub mod filesystems;  // FILESYSTEMS — list FS types for selected volume
pub mod format;       // FORMAT      — mkfs.* for all major file systems
pub mod gpt;          // GPT         — set GPT attribute bits via sgdisk
pub mod help;         // HELP        — print command reference
pub mod import_cmd;   // IMPORT      — import LVM VGs or ZFS pools
pub mod inactive;     // INACTIVE    — clear boot/esp flag from partition
pub mod list;         // LIST        — list disks / partitions / volumes
pub mod offline;      // OFFLINE     — unmount / spin-down disk or volume
pub mod online;       // ONLINE      — spin-up / mount disk or volume
pub mod recover;      // RECOVER     — partprobe + fsck + mdadm reassemble
pub mod rem;          // REM         — comment line (no-op)
pub mod remove;       // REMOVE      — unmount partition (reverse of assign)
pub mod repair;       // REPAIR      — mdadm / LVM / ZFS repair
pub mod rescan;       // RESCAN      — partprobe
pub mod retain;       // RETAIN      — stub (Windows LDM concept, not on Linux)
pub mod san;          // SAN         — stub (Windows VDS concept, not on Linux)
pub mod select;       // SELECT      — select disk / partition / volume
pub mod setid;        // SET ID      — change partition type via sgdisk/sfdisk
pub mod shrink;       // SHRINK      — reduce partition + file system
pub mod uniqueid;     // UNIQUEID    — display/set disk GUID or MBR signature

// ── Unimplementable Windows-only stubs ────────────────────────────────────
pub mod impossible;   // ATTACH / DETACH / COMPACT / EXPAND / MERGE / ATTRIBUTES

// ── Virtual disk manager ───────────────────────────────────────────────────
pub mod vdisk;        // VDISK — create/attach/detach/compact/expand/info/list

// ── Linux extensions (no DiskPart equivalent) ─────────────────────────────
pub mod benchmark;    // BENCHMARK  — sequential read/write speed test
pub mod encrypt;      // ENCRYPT    — LUKS2 setup/open/close/status
pub mod smart;        // SMART      — S.M.A.R.T. health data via smartctl
pub mod snapshot;     // SNAPSHOT   — LVM / Btrfs snapshot management
pub mod wipe;         // WIPE       — secure erase via dd / shred / blkdiscard
