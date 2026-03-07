# DiskParted — Implementation Roadmap

Legend:
- `[+]` Implemented
- `[-]` Not implementable on Linux (Windows VDS/LDM/Hyper-V only)
- `[~]` Partial stub
- `[ ]` Planned, not yet implemented

---

## DiskPart Command Status

| Command      | Status | Notes |
|--------------|--------|-------|
| ACTIVE       | `[+]`  | Sets boot/esp flag via parted |
| ADD          | `[+]`  | mdadm RAID-1 mirror add |
| ASSIGN       | `[+]`  | Mount point assignment |
| ATTACH       | `[-]`  | Windows VDS only — use `vdisk attach` |
| ATTRIBUTES   | `[-]`  | Windows VDS/NTFS only |
| AUTOMOUNT    | `[+]`  | fstab / udev |
| BREAK        | `[+]`  | mdadm RAID-1 mirror break |
| CLEAN        | `[+]`  | wipefs + sgdisk --zap-all |
| COMPACT      | `[-]`  | Windows VDS only — use `vdisk compact` |
| CONVERT      | `[+]`  | MBR↔GPT + ext2/3/4 upgrade |
| CREATE       | `[+]`  | primary / efi / msr / extended / logical |
| DELETE       | `[+]`  | parted rm |
| DETAIL       | `[+]`  | disk / partition / volume info |
| DETACH       | `[-]`  | Windows VDS only — use `vdisk detach` |
| EXIT         | `[+]`  | |
| EXPAND       | `[-]`  | Windows VDS only — use `vdisk expand` |
| EXTEND       | `[ ]`  | resize2fs / xfs_growfs — not yet implemented |
| FILESYSTEMS  | `[+]`  | lsblk FSTYPE + mkfs availability check |
| FORMAT       | `[+]`  | mkfs.* for all major filesystems |
| GPT          | `[+]`  | sgdisk --attributes |
| HELP         | `[+]`  | |
| IMPORT       | `[+]`  | LVM vgimport / ZFS zpool import |
| INACTIVE     | `[+]`  | Clears boot/esp flag |
| LIST         | `[+]`  | disk / partition / volume |
| MERGE        | `[-]`  | Windows Hyper-V only |
| OFFLINE      | `[+]`  | umount + hdparm spindown |
| ONLINE       | `[+]`  | hdparm spinup + mount |
| RECOVER      | `[+]`  | partprobe + fsck + mdadm --assemble |
| REM          | `[+]`  | No-op comment for scripting |
| REMOVE       | `[+]`  | umount + remove fstab entry |
| REPAIR       | `[+]`  | mdadm / LVM / ZFS repair |
| RESCAN       | `[+]`  | partprobe |
| RETAIN       | `[~]`  | Windows LDM stub — suggests `active` |
| SAN          | `[~]`  | Windows VDS stub — suggests `automount` |
| SELECT       | `[+]`  | disk / partition / volume with cross-selection |
| SET ID       | `[+]`  | sgdisk --typecode / sfdisk |
| SHRINK       | `[ ]`  | resize2fs / ntfsresize — not yet implemented |
| UNIQUEID     | `[+]`  | sgdisk -u (GPT) / sfdisk --disk-id (MBR) |

---

## Linux Extension: VDISK

DiskParted adds `vdisk` as a Linux-native replacement for the Windows VHD commands:

| Subcommand     | Status | Notes |
|----------------|--------|-------|
| vdisk create   | `[+]`  | qemu-img create |
| vdisk attach   | `[+]`  | qemu-nbd → /dev/nbdN |
| vdisk detach   | `[+]`  | qemu-nbd --disconnect |
| vdisk compact  | `[+]`  | qemu-img convert -c |
| vdisk expand   | `[+]`  | qemu-img resize |
| vdisk info     | `[+]`  | qemu-img info |
| vdisk list     | `[+]`  | /sys/block/nbdN inspection |

Supported formats: `qcow2`, `raw`, `vdi`, `vmdk`, `vhd`, `hdd`

---

## Remaining Work

### High priority
- **EXTEND** — grow a partition/volume after `create` or disk expansion
  - ext2/3/4: `resize2fs`
  - xfs: `xfs_growfs`
  - ntfs: `ntfsresize`
  - btrfs: `btrfs filesystem resize`
- **SHRINK** — shrink a partition/volume before `delete` or disk replacement
  - ext2/3/4: `resize2fs`
  - ntfs: `ntfsresize`
  - btrfs: `btrfs filesystem resize`
  - Note: must unmount first for most filesystems; XFS cannot shrink

### Lower priority
- Script/batch mode (`diskparted < script.txt`)
- Operation logging
- `--noerr` flag support (continue on errors, DiskPart scripting feature)
- Color output / TUI mode
