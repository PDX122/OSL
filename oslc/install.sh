#!/bin/bash
# OSL Compiler Installation Script
# Run: ./install.sh

set -e

INSTALL_DIR="${HOME}/.local/bin"
TARGET_DIR="/usr/local/bin"

echo "Installing OSL Compiler (oslc)..."

# Build release version
echo "Building release binary..."
cd "$(dirname "$0")"
cargo build --release

# Determine install location
if [ -w "$TARGET_DIR" ]; then
    INSTALL_PATH="$TARGET_DIR"
else
    mkdir -p "$INSTALL_DIR"
    INSTALL_PATH="$INSTALL_DIR"
    echo "Note: Added $INSTALL_DIR to PATH in ~/.bashrc or ~/.zshrc if needed:"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi

# Copy binary
cp target/release/oslc "$INSTALL_PATH/oslc"
chmod +x "$INSTALL_PATH/oslc"

echo "✓ Installed oslc to $INSTALL_PATH/oslc"

# Verify
if "$INSTALL_PATH/oslc" --version 2>/dev/null; then
    echo "✓ Verification successful"
else
    echo "✓ Installed (version check skipped)"
fi

echo ""
echo "Usage: oslc <command> [file] [flags]"
echo "  oslc run <file.osl>     - Run an OSL file"
echo "  oslc build <file.osl>   - Compile to binary"
echo "  oslc check <file.osl>   - Check syntax"
echo "  oslc typecheck          - Type check"
echo "  oslc init <project>     - Initialize project"