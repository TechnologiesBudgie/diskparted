#!/usr/bin/env bash
# build.sh — detect distro, install dependencies, and build diskparted
set -euo pipefail

###############################################################################
# Helpers
###############################################################################
info()    { printf '\033[1;34m[build]\033[0m %s\n' "$*"; }
success() { printf '\033[1;32m[build]\033[0m %s\n' "$*"; }
warn()    { printf '\033[1;33m[build]\033[0m %s\n' "$*"; }
die()     { printf '\033[1;31m[build]\033[0m ERROR: %s\n' "$*" >&2; exit 1; }

###############################################################################
# Detect distro / package manager
###############################################################################
detect_distro() {
    if [ -f /etc/os-release ]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        DISTRO_ID="${ID:-unknown}"
        DISTRO_ID_LIKE="${ID_LIKE:-}"
        DISTRO_NAME="${NAME:-unknown}"
    elif [ -f /etc/redhat-release ]; then
        DISTRO_ID="rhel"
        DISTRO_NAME="$(cat /etc/redhat-release)"
    elif [ -f /etc/debian_version ]; then
        DISTRO_ID="debian"
        DISTRO_NAME="Debian"
    elif command -v sw_vers &>/dev/null; then
        DISTRO_ID="macos"
        DISTRO_NAME="macOS $(sw_vers -productVersion)"
    else
        DISTRO_ID="unknown"
        DISTRO_NAME="Unknown"
    fi
    info "Detected: $DISTRO_NAME (id=$DISTRO_ID)"
}

###############################################################################
# Build deps:    curl gcc (for rustup + linking)
# Runtime deps:  parted partprobe(parted) e2fsprogs(resize2fs/e2fsck/dumpe2fs)
#                xfsprogs btrfs-progs ntfs-3g(ntfsresize) cryptsetup
#                smartmontools(smartctl) lvm2 qemu-utils(qemu-img/qemu-nbd)
#                sgdisk(gdisk) util-linux(lsblk/blkdiscard/wipefs) shred(coreutils)
###############################################################################
install_system_deps() {
    # Helper: run with sudo if not already root
    _sudo() {
        if [ "$(id -u)" -eq 0 ]; then
            "$@"
        else
            sudo "$@"
        fi
    }

    # Normalise ID_LIKE so we can match families
    local id_like_lower
    id_like_lower="$(echo "${DISTRO_ID_LIKE:-}" | tr '[:upper:]' '[:lower:]')"
    local id_lower
    id_lower="$(echo "${DISTRO_ID}" | tr '[:upper:]' '[:lower:]')"

    case "$id_lower" in
    # ── Debian / Ubuntu family ────────────────────────────────────────────────
    debian|ubuntu|linuxmint|pop|elementary|kali|parrot|zorin|raspbian|tails|mx|deepin|neon)
        info "Using apt-get (Debian/Ubuntu family)"
        _sudo apt-get update -qq
        _sudo apt-get install -y --no-install-recommends \
            curl gcc build-essential \
            parted \
            e2fsprogs \
            xfsprogs \
            btrfs-progs \
            ntfs-3g \
            cryptsetup \
            smartmontools \
            lvm2 \
            qemu-utils \
            gdisk \
            util-linux \
            coreutils
        ;;

    # ── Fedora / RHEL / CentOS family ─────────────────────────────────────────
    fedora|rhel|centos|rocky|almalinux|ol|scientific|nobara|eurolinux)
        if command -v dnf &>/dev/null; then
            info "Using dnf (Fedora/RHEL family)"
            _sudo dnf install -y \
                curl gcc \
                parted \
                e2fsprogs \
                xfsprogs \
                btrfs-progs \
                ntfsprogs \
                cryptsetup \
                smartmontools \
                lvm2 \
                qemu-img \
                gdisk \
                util-linux \
                coreutils
        else
            info "Using yum (legacy RHEL/CentOS)"
            _sudo yum install -y \
                curl gcc \
                parted \
                e2fsprogs \
                xfsprogs \
                btrfs-progs \
                ntfsprogs \
                cryptsetup \
                smartmontools \
                lvm2 \
                qemu-img \
                gdisk \
                util-linux \
                coreutils
        fi
        ;;

    # ── openSUSE / SLES ────────────────────────────────────────────────────────
    opensuse*|sles|suse)
        info "Using zypper (openSUSE/SLES)"
        _sudo zypper --non-interactive install \
            curl gcc \
            parted \
            e2fsprogs \
            xfsprogs \
            btrfsprogs \
            ntfs-3g \
            cryptsetup \
            smartmontools \
            lvm2 \
            qemu-tools \
            gptfdisk \
            util-linux \
            coreutils
        ;;

    # ── Arch / Manjaro / EndeavourOS ──────────────────────────────────────────
    arch|manjaro|endeavouros|garuda|artix|blackarch|parabola|crystal)
        info "Using pacman (Arch family)"
        _sudo pacman -Sy --noconfirm --needed \
            curl gcc base-devel \
            parted \
            e2fsprogs \
            xfsprogs \
            btrfs-progs \
            ntfs-3g \
            cryptsetup \
            smartmontools \
            lvm2 \
            qemu-base \
            gptfdisk \
            util-linux \
            coreutils
        ;;

    # ── Alpine ────────────────────────────────────────────────────────────────
    alpine)
        info "Using apk (Alpine)"
        _sudo apk add --no-cache \
            curl gcc musl-dev \
            parted \
            e2fsprogs e2fsprogs-extra \
            xfsprogs \
            btrfs-progs \
            ntfs-3g ntfs-3g-progs \
            cryptsetup \
            smartmontools \
            lvm2 \
            qemu-img \
            sgdisk \
            util-linux \
            coreutils
        ;;

    # ── Void Linux ────────────────────────────────────────────────────────────
    void)
        info "Using xbps-install (Void)"
        _sudo xbps-install -Sy \
            curl gcc \
            parted \
            e2fsprogs \
            xfsprogs \
            btrfs-progs \
            ntfs-3g \
            cryptsetup \
            smartmontools \
            lvm2 \
            qemu \
            gptfdisk \
            util-linux \
            coreutils
        ;;

    # ── Gentoo ────────────────────────────────────────────────────────────────
    gentoo)
        info "Using emerge (Gentoo)"
        _sudo emerge --ask=n \
            net-misc/curl sys-devel/gcc \
            sys-block/parted \
            sys-fs/e2fsprogs \
            sys-fs/xfsprogs \
            sys-fs/btrfs-progs \
            sys-fs/ntfs3g \
            sys-fs/cryptsetup \
            sys-apps/smartmontools \
            sys-fs/lvm2 \
            app-emulation/qemu \
            sys-apps/gptfdisk \
            sys-apps/util-linux \
            sys-apps/coreutils
        ;;

    # ── NixOS ─────────────────────────────────────────────────────────────────
    nixos)
        warn "NixOS detected — please ensure dependencies are in your environment."
        warn "Consider using: nix-shell -p curl gcc rustup parted e2fsprogs xfsprogs"
        warn "  btrfs-progs ntfs3g cryptsetup smartmontools lvm2 qemu gdisk util-linux"
        ;;

    # ── Slackware ─────────────────────────────────────────────────────────────
    slackware)
        warn "Slackware detected — please install the following manually:"
        warn "  curl gcc parted e2fsprogs xfsprogs btrfs-progs ntfs-3g"
        warn "  cryptsetup smartmontools lvm2 qemu gdisk util-linux"
        ;;

    # ── macOS ─────────────────────────────────────────────────────────────────
    macos)
        if command -v brew &>/dev/null; then
            info "Using Homebrew (macOS)"
            brew install curl smartmontools qemu
            warn "Note: parted, cryptsetup, lvm2, e2fsprogs, xfsprogs, btrfs-progs,"
            warn "  and ntfs-3g are Linux-only. Some features will not work on macOS."
        else
            warn "Homebrew not found. Install it from https://brew.sh or ensure Xcode CLT is installed."
            xcode-select --install 2>/dev/null || true
        fi
        ;;

    # ── Fallback: check ID_LIKE ───────────────────────────────────────────────
    *)
        if echo "$id_like_lower" | grep -qE 'debian|ubuntu'; then
            info "ID_LIKE matches Debian family — using apt-get"
            _sudo apt-get update -qq
            _sudo apt-get install -y --no-install-recommends \
                curl gcc build-essential \
                parted e2fsprogs xfsprogs btrfs-progs ntfs-3g cryptsetup \
                smartmontools lvm2 qemu-utils gdisk util-linux coreutils
        elif echo "$id_like_lower" | grep -qE 'rhel|fedora|centos'; then
            info "ID_LIKE matches RHEL family — using dnf/yum"
            if command -v dnf &>/dev/null; then
                _sudo dnf install -y \
                    curl gcc parted e2fsprogs xfsprogs btrfs-progs ntfsprogs \
                    cryptsetup smartmontools lvm2 qemu-img gdisk util-linux coreutils
            else
                _sudo yum install -y \
                    curl gcc parted e2fsprogs xfsprogs btrfs-progs ntfsprogs \
                    cryptsetup smartmontools lvm2 qemu-img gdisk util-linux coreutils
            fi
        elif echo "$id_like_lower" | grep -qE 'arch'; then
            info "ID_LIKE matches Arch family — using pacman"
            _sudo pacman -Sy --noconfirm --needed \
                curl gcc base-devel parted e2fsprogs xfsprogs btrfs-progs ntfs-3g \
                cryptsetup smartmontools lvm2 qemu-base gptfdisk util-linux coreutils
        elif echo "$id_like_lower" | grep -qE 'suse'; then
            info "ID_LIKE matches openSUSE family — using zypper"
            _sudo zypper --non-interactive install \
                curl gcc parted e2fsprogs xfsprogs btrfsprogs ntfs-3g \
                cryptsetup smartmontools lvm2 qemu-tools gptfdisk util-linux coreutils
        else
            warn "Unknown distro '$DISTRO_NAME'. Skipping automatic dependency install."
            warn "Please ensure the following are installed, then re-run:"
            warn "  Build:   curl, gcc (or clang)"
            warn "  Runtime: parted, e2fsprogs, xfsprogs, btrfs-progs, ntfs-3g,"
            warn "           cryptsetup, smartmontools, lvm2, qemu-utils, gdisk,"
            warn "           util-linux, coreutils"
        fi
        ;;
    esac
}

###############################################################################
# Install Rust via rustup (if not already present)
###############################################################################
install_rust() {
    if command -v cargo &>/dev/null; then
        info "Rust/Cargo already installed: $(cargo --version)"
        return
    fi

    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

    # Make cargo available in this session
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"

    success "Rust installed: $(rustc --version)"
}

###############################################################################
# Build
###############################################################################
build() {
    # Ensure cargo is on PATH (covers both fresh installs and existing ones)
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi

    command -v cargo &>/dev/null || die "cargo not found. Rust installation may have failed."

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"

    info "Building diskparted (release)…"
    cargo build --release

    BINARY="$SCRIPT_DIR/target/release/diskparted"
    [ -f "$BINARY" ] || die "Build succeeded but binary not found at $BINARY"

    success "Build complete → $BINARY"
    info  "Run ./install.sh (as root) to install to /usr/local/sbin"
}

###############################################################################
# Main
###############################################################################
main() {
    detect_distro
    install_system_deps
    install_rust
    build
}

main "$@"
