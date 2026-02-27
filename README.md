# DiskParted

**DiskParted** is a Linux-native, Rust-based tool inspired by Microsoft DiskPart. Its goal is to provide a familiar CLI for managing disks, partitions, and volumes on Linux systems. DiskParted aims to give power users, new GNU/Linux users, and sysadmins a safe and scriptable way to handle disk operations without relying on GUI tools.

**Version:** 0.1.1
**Authors:** Infomanraf, Raphaël Larocque
**License:** GNU GPLv3

---

## Purpose

DiskParted exists to:

* Bring **DiskPart-like functionality to GNU/Linux**, allowing users to manage disks and partitions with commands similar to Microsoft Windows.
* Enable **scripting and automation** of disk operations in Linux environments.
* Provide a **modular and extensible platform** where additional disk management features can be implemented over time.

---

## Current Status

DiskParted implements a subset of DiskPart commands. Currently **fully functional commands** include:

* `create` – Create a partition or volume
* `clean` – Remove all partition information from the disk
* `delete partition` – Delete a partition
* `exit` – Exit the tool
* `format` – Format a volume
* `help` – Display help information
* `list disk` – List all disks
* `list volume` – List all volumes
* `rescan` – Rescan disks
* `select disk <n>` – Select a disk
* `select partition <n>` – Select a partition
* `shrink volume <MB>` – Shrink a volume

Other commands, like `active`, `assign`, `extend`, `merge`, and `gpt`, are planned but **not yet implemented**.

---

## Roadmap

The development of DiskParted is planned in phases:

**Phase 1 – Core Functionality (Current)**

* Basic disk and partition operations (`create`, `delete`, `clean`)
* Interactive shell interface
* Disk/volume selection and listing
* Basic formatting and shrinking

**Phase 2 – Advanced Features**

* Full support for **GPT and MBR operations** (`gpt`, `set id`, `uniqueid`)
* Volume extensions and mirror management (`extend`, `break`, `add`)
* Drive letter and mount point assignments (`assign`, `remove`)
* Virtual disk (VHD) support (`attach`, `detach`, `expand`, `merge`)

**Phase 3 – Automation & Scripting**

* Batch scripting support
* Logging and reporting for operations
* Integration with Linux automation tools

**Phase 4 – Extra Utilities**

* RAID and SAN management (`repair`, `recover`, `san`)
* File system information and compatibility checks (`filesystems`)
* Advanced partition attributes and retention features (`attributes`, `retain`)

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

## Contribution

DiskParted is under active development. Contributions are welcome, especially for:

* Implementing missing DiskPart commands
* Improving safety checks and error handling
* Adding support for virtual disks and advanced volume management
* Adding support for new file systems
* Adding new features
* Well, pretty much anything
