#!/usr/bin/env bash
# install.sh — install the diskparted binary to /usr/local/sbin
# Must be run as root (or with sudo).
set -euo pipefail

INSTALL_DIR="/usr/local/sbin"
BINARY_NAME="diskparted"

###############################################################################
# Helpers
###############################################################################
info()    { printf '\033[1;34m[install]\033[0m %s\n' "$*"; }
success() { printf '\033[1;32m[install]\033[0m %s\n' "$*"; }
die()     { printf '\033[1;31m[install]\033[0m ERROR: %s\n' "$*" >&2; exit 1; }

###############################################################################
# Checks
###############################################################################

# Root check
if [ "$(id -u)" -ne 0 ]; then
    die "This script must be run as root. Try: sudo ./install.sh"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/target/release/$BINARY_NAME"

if [ ! -f "$BINARY" ]; then
    die "Binary not found at '$BINARY'. Please run ./build.sh first."
fi

###############################################################################
# Install
###############################################################################

info "Installing $BINARY_NAME to $INSTALL_DIR …"

# Create install dir if it somehow doesn't exist
install -d "$INSTALL_DIR"

# Copy binary with correct permissions (755, owned by root)
install -o root -m 0755 "$BINARY" "$INSTALL_DIR/$BINARY_NAME"

success "$BINARY_NAME installed to $INSTALL_DIR/$BINARY_NAME"
info  "Run 'diskparted' (as root) to start."

###############################################################################
# Runtime dependency check
###############################################################################
info "Checking runtime dependencies…"

RUNTIME_DEPS=(
    # tool          # package (common name)
    "parted"        # parted
    "partprobe"     # parted
    "resize2fs"     # e2fsprogs
    "e2fsck"        # e2fsprogs
    "dumpe2fs"      # e2fsprogs
    "xfs_growfs"    # xfsprogs
    "btrfs"         # btrfs-progs
    "ntfsresize"    # ntfs-3g / ntfsprogs
    "cryptsetup"    # cryptsetup
    "smartctl"      # smartmontools
    "lvcreate"      # lvm2
    "lvremove"      # lvm2
    "lvdisplay"     # lvm2
    "lvconvert"     # lvm2
    "vgscan"        # lvm2
    "qemu-img"      # qemu-utils / qemu-img
    "qemu-nbd"      # qemu-utils / qemu-nbd
    "sgdisk"        # gdisk / gptfdisk
    "lsblk"         # util-linux
    "blkdiscard"    # util-linux
    "wipefs"        # util-linux
    "shred"         # coreutils
    "dd"            # coreutils
)

MISSING=()
i=0
while [ $i -lt ${#RUNTIME_DEPS[@]} ]; do
    tool="${RUNTIME_DEPS[$i]}"
    if ! command -v "$tool" &>/dev/null; then
        MISSING+=("$tool")
    fi
    i=$(( i + 1 ))
done

if [ ${#MISSING[@]} -eq 0 ]; then
    success "All runtime dependencies found."
else
    printf '\033[1;33m[install]\033[0m WARNING: The following runtime tools were not found:\n'
    for t in "${MISSING[@]}"; do
        printf '  \033[1;33m•\033[0m %s\n' "$t"
    done
    printf '\033[1;33m[install]\033[0m Install them via your package manager. Common packages:\n'
    printf '  Debian/Ubuntu: parted e2fsprogs xfsprogs btrfs-progs ntfs-3g cryptsetup\n'
    printf '                 smartmontools lvm2 qemu-utils gdisk util-linux coreutils\n'
    printf '  Fedora/RHEL:   parted e2fsprogs xfsprogs btrfs-progs ntfsprogs cryptsetup\n'
    printf '                 smartmontools lvm2 qemu-img gdisk util-linux coreutils\n'
    printf '  Arch:          parted e2fsprogs xfsprogs btrfs-progs ntfs-3g cryptsetup\n'
    printf '                 smartmontools lvm2 qemu-base gptfdisk util-linux coreutils\n'
fi
