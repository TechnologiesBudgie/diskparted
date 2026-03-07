# DiskParted

**DiskParted** is a Linux-native, Rust-based tool inspired by Microsoft DiskPart. Its goal is to provide a familiar CLI for managing disks, partitions, and volumes on Linux systems. DiskParted aims to give power users, new GNU/Linux users, and sysadmins a safe and scriptable way to handle disk operations without relying on GUI tools.

**Version:** 1.0.1  
**Author:** DiskParted Team  
**License:** GNU GPLv3  

---

## Purpose

DiskParted exists to:

- Bring **DiskPart-like functionality to GNU/Linux**, allowing users to manage disks and partitions with commands similar to Microsoft Windows.
- Enable **scripting and automation** of disk operations in Linux environments.
- Provide a **modular and extensible platform** where additional disk management features can be implemented over time.

---

## Current Status

| Symbol | Meaning |
|--------|---------|
| `[+]`  | Fully implemented |
| `[-]`  | Not implementable on Linux (Windows VDS/LDM/Hyper-V only) — running the command explains why and suggests alternatives |
| `[~]`  | Partial stub — works with caveats, see command output |
| `[ ]`  | Planned, not yet implemented |

| Command       | Status | Notes |
|---------------|--------|-------|
| `ACTIVE`      | `[+]`  | Sets boot/esp flag on selected partition via parted |
| `ADD`         | `[+]`  | Adds a mirror disk to a volume via mdadm RAID-1 |
| `ASSIGN`      | `[+]`  | Assigns a mount point to the selected volume |
| `ATTACH`      | `[-]`  | Windows VDS only — use `vdisk attach` instead |
| `ATTRIBUTES`  | `[-]`  | Windows VDS/NTFS only — partial via `blockdev` |
| `AUTOMOUNT`   | `[+]`  | Manages fstab/udev automounting |
| `BREAK`       | `[+]`  | Breaks an mdadm RAID-1 mirror |
| `CLEAN`       | `[+]`  | Wipes all partition info from selected disk |
| `COMPACT`     | `[-]`  | Windows VDS only — use `vdisk compact` instead |
| `CONVERT`     | `[+]`  | Converts disk MBR↔GPT, or upgrades ext2/3/4 in-place |
| `CREATE`      | `[+]`  | Creates a partition (primary/efi/msr/extended/logical) |
| `DELETE`      | `[+]`  | Deletes a partition or volume |
| `DETAIL`      | `[+]`  | Shows detailed info on selected disk/partition/volume |
| `DETACH`      | `[-]`  | Windows VDS only — use `vdisk detach` instead |
| `EXIT`        | `[+]`  | Exits diskparted |
| `EXPAND`      | `[-]`  | Windows VDS only — use `vdisk expand` instead |
| `EXTEND`      | `[ ]`  | Planned |
| `FILESYSTEMS` | `[+]`  | Shows current filesystem and supported formats for selected volume |
| `FORMAT`      | `[+]`  | Formats the selected partition |
| `GPT`         | `[+]`  | Sets GPT attribute bits on selected partition via sgdisk |
| `HELP`        | `[+]`  | Displays help information |
| `IMPORT`      | `[+]`  | Imports a foreign LVM volume group or ZFS pool |
| `INACTIVE`    | `[+]`  | Clears boot/esp flag from selected partition |
| `LIST`        | `[+]`  | Lists disks, partitions, or volumes |
| `MERGE`       | `[-]`  | Windows Hyper-V only — see `qemu-img commit` for qcow2 |
| `OFFLINE`     | `[+]`  | Unmounts/spins down the selected disk or volume |
| `ONLINE`      | `[+]`  | Spins up/mounts the selected disk or volume |
| `RECOVER`     | `[+]`  | Runs partprobe + fsck + mdadm reassemble |
| `REM`         | `[+]`  | Comment line, no-op (for script compatibility) |
| `REMOVE`      | `[+]`  | Removes a mount point from the selected volume |
| `REPAIR`      | `[+]`  | Repairs a RAID/LVM/ZFS volume on the selected disk |
| `RESCAN`      | `[+]`  | Rescans disks via partprobe |
| `RETAIN`      | `[~]`  | Windows LDM concept — use `active` instead |
| `SAN`         | `[~]`  | Windows VDS concept — use `automount` instead |
| `SELECT`      | `[+]`  | Selects a disk, partition, or volume |
| `SET ID`      | `[+]`  | Changes partition type (GPT GUID or MBR byte) via sgdisk/sfdisk |
| `SHRINK`      | `[ ]`  | Planned |
| `UNIQUEID`    | `[+]`  | Displays or sets disk GUID (GPT) or MBR signature |
| `VDISK`       | `[+]`  | Manages virtual disk images (qcow2/raw/vdi/vmdk/vhd/hdd) via qemu-nbd |

---

## Virtual Disk Support

DiskParted adds a `vdisk` command that replaces and extends the Windows-only `ATTACH`/`DETACH`/`COMPACT`/`EXPAND` commands with cross-format virtual disk management:

```
vdisk create  <file> format=<fmt> size=<MB>   Create a new virtual disk image
vdisk attach  <file>                           Attach image as /dev/nbdN
vdisk detach  <file | /dev/nbdN | all>         Detach an attached image
vdisk compact <file>                           Reclaim unused space
vdisk expand  <file> size=<MB>                 Grow an image
vdisk info    <file>                           Show image metadata
vdisk list                                     List all attached NBD devices
```

Supported formats: `qcow2`, `raw`, `vdi`, `vmdk`, `vhd`, `hdd`

Requires: `qemu-img` — `pacman -S qemu-img`

---

## Installation

Build from source using Rust:

```bash
git clone https://codeberg.org/Infomanraf/diskparted.git
cd diskparted
cargo build --release
sudo cp target/release/diskparted /usr/local/sbin/
```

Run:

```bash
sudo diskparted
```

---

## Dependencies

| Tool        | Used by | Install |
|-------------|---------|---------|
| `parted`    | create, delete, convert | `pacman -S parted` |
| `sgdisk`    | gpt, setid, uniqueid, clean | `pacman -S gptfdisk` |
| `wipefs`    | clean | `pacman -S util-linux` |
| `partprobe` | rescan, recover | `pacman -S parted` |
| `lsblk`     | list, select, detail | `pacman -S util-linux` |
| `mdadm`     | add, break, recover, repair | `pacman -S mdadm` |
| `qemu-img`  | vdisk | `pacman -S qemu-img` |
| `qemu-nbd`  | vdisk attach/detach | `pacman -S qemu-img` |

---

## Contribution

DiskParted is under active development. Contributions are welcome, especially for:

- Implementing `EXTEND` and `SHRINK` (the last two unimplemented core commands)
- Improving safety checks and error handling
- Adding support for new filesystems
- Testing on non-Arch distributions
