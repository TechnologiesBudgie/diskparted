# DiskParted — Implementation Roadmap

Legend:
- `[+]` Implemented (DiskPart-compatible)
- `[-]` Not implementable on Linux (Windows VDS/LDM/Hyper-V only)
- `[~]` Partial stub
- `[!]` Linux extension — not in Windows DiskPart, unique to DiskParted

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
| EXTEND       | `[+]`  | parted resizepart + resize2fs / xfs_growfs / btrfs / ntfsresize |
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
| SHRINK       | `[+]`  | e2fsck + resize2fs / ntfsresize / btrfs filesystem resize + parted |
| UNIQUEID     | `[+]`  | sgdisk -u (GPT) / sfdisk --disk-id (MBR) |

---

## Virtual Disk Manager: VDISK

DiskParted adds `vdisk` as a Linux-native replacement for Windows VHD commands:

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

## Linux Extensions  [!]

Commands with no Windows DiskPart equivalent, added by DiskParted:

| Command        | Status | Notes |
|----------------|--------|-------|
| BENCHMARK      | `[!]`  | dd sequential read/write throughput test |
| ENCRYPT        | `[!]`  | cryptsetup LUKS2 setup / open / close / status |
| SMART          | `[!]`  | smartctl health summary and self-test |
| SNAPSHOT       | `[!]`  | LVM lvcreate snapshot / Btrfs subvolume snapshot |
| WIPE           | `[!]`  | dd zeros / shred random / blkdiscard (TRIM) |

---

## Remaining Work

### Planned features
- Script/batch mode (`diskparted < script.txt` or `diskparted -c "command"`)
- `--noerr` global flag (continue on errors, DiskPart scripting feature)
- Color output (green `[+]`, red errors, yellow warnings)
- Tab completion improvements (argument-level completion)
- Operation log file (`--log=<path>`)
