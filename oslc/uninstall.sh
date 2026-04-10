#!/bin/bash
# OSL Compiler Uninstallation Script
# Run: ./uninstall.sh

set -e

INSTALL_DIR="${HOME}/.local/bin"
TARGET_DIR="/usr/local/bin"
CARGO_BIN="${HOME}/.cargo/bin"

echo "Uninstalling OSL Compiler (oslc)..."

REMOVED=0

# Try cargo bin location first
if [ -f "$CARGO_BIN/oslc" ]; then
    rm "$CARGO_BIN/oslc"
    echo "✓ Removed $CARGO_BIN/oslc"
    REMOVED=1
fi

# Try user install location
if [ -f "$INSTALL_DIR/oslc" ]; then
    rm "$INSTALL_DIR/oslc"
    echo "✓ Removed $INSTALL_DIR/oslc"
    REMOVED=1
fi

# Try system install location
if [ -f "$TARGET_DIR/oslc" ]; then
    sudo rm "$TARGET_DIR/oslc"
    echo "✓ Removed $TARGET_DIR/oslc"
    REMOVED=1
fi

# Also remove cached packages if user wants
if [ "$1" = "--purge" ] || [ "$1" = "-p" ]; then
    echo "Removing cached packages..."
    rm -rf "${HOME}/.oslc"
    echo "✓ Removed ${HOME}/.oslc"
    REMOVED=1
fi

if [ $REMOVED -eq 0 ]; then
    echo "oslc not found in known locations."
    echo "Searched:"
    echo "  $CARGO_BIN/oslc"
    echo "  $INSTALL_DIR/oslc"
    echo "  $TARGET_DIR/oslc"
    exit 1
fi

echo ""
echo "✓ Uninstallation complete"
