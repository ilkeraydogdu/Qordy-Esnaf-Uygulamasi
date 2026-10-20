#!/usr/bin/env bash
set -e

# ==============================================================================
# QORDY Esnaf POS - Windows Distribution & Installer Build Automation
# Cross-compiles native 64-bit Rust binary and packages Windows Setup .exe
# ==============================================================================

echo "================================================================="
echo "  QORDY Esnaf POS v3.0 - Windows Release & Installer Builder"
echo "================================================================="

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "1. Checking cross-compilation toolchain..."
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "ERROR: MinGW-w64 is required. Install with: brew install mingw-w64"
    exit 1
fi

if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    echo "Installing Rust target x86_64-pc-windows-gnu..."
    rustup target add x86_64-pc-windows-gnu
fi

if ! command -v makensis &> /dev/null; then
    echo "ERROR: NSIS compiler (makensis) is required. Install with: brew install makensis"
    exit 1
fi

echo "2. Compiling optimized release binary for Windows (x86_64-pc-windows-gnu)..."
cargo build --release --target x86_64-pc-windows-gnu

EXE_PATH="$ROOT_DIR/target/x86_64-pc-windows-gnu/release/qordy_pos_rs.exe"
if [ ! -f "$EXE_PATH" ]; then
    echo "ERROR: Compiled executable not found at $EXE_PATH"
    exit 1
fi

EXE_SIZE=$(ls -lh "$EXE_PATH" | awk '{print $5}')
echo "   Binary compiled successfully: $EXE_PATH ($EXE_SIZE)"

echo "3. Creating distribution installer with NSIS..."
mkdir -p "$ROOT_DIR/dist/installer"
cd "$ROOT_DIR/installer"
makensis setup.nsi

INSTALLER_PATH="$ROOT_DIR/dist/installer/QORDY_Esnaf_POS_v3.0.0_Setup.exe"
if [ -f "$INSTALLER_PATH" ]; then
    INSTALLER_SIZE=$(ls -lh "$INSTALLER_PATH" | awk '{print $5}')
    echo "================================================================="
    echo "  SUCCESS: Windows Installer Ready for Corporate Distribution!"
    echo "  Installer: $INSTALLER_PATH ($INSTALLER_SIZE)"
    echo "================================================================="
else
    echo "ERROR: Failed to generate installer."
    exit 1
fi
