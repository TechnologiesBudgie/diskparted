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
