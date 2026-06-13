#!/bin/bash
set -euo pipefail

TARGET="${1:-armv7-unknown-linux-gnueabihf}"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="${PROJECT_DIR}/target/cross/${TARGET}/release"

echo "=========================================="
echo " Cross-compile rustsdlretro"
echo " Target: ${TARGET}"
echo "=========================================="

# Install cross-compilation toolchain
echo "[1/4] Installing cross-compilation toolchain..."
if ! command -v gcc-${TARGET} &> /dev/null; then
    echo "  Installing gcc for ${TARGET}..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y gcc-${TARGET} g++-${TARGET}
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm gcc-${TARGET}
    elif command -v brew &> /dev/null; then
        brew install FiloSottile/musl-cross/musl-cross
    else
        echo "  ERROR: Cannot detect package manager. Please install ${TARGET} toolchain manually."
        exit 1
    fi
fi

# Install cross-compilation dependencies
echo "[2/4] Installing cross-compilation dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get install -y \
        pkg-config \
        libasound2-dev:${TARGET} \
        libudev-dev:${TARGET} 2>/dev/null || true
fi

# Set up cross-compilation environment
echo "[3/4] Setting up cross-compilation environment..."
export CC="${TARGET}-gcc"
export CXX="${TARGET}-g++"
export PKG_CONFIG_PATH="/usr/lib/${TARGET}/pkgconfig:/usr/share/pkgconfig:/usr/local/lib/${TARGET}/pkgconfig:${PKG_CONFIG_PATH:-}"
export PKG_CONFIG_ALLOW_CROSS=1

# Install Rust target
echo "  Adding Rust target ${TARGET}..."
rustup target add "${TARGET}" 2>/dev/null || true

# Build
echo "[4/4] Building..."
cd "${PROJECT_DIR}"

# Build the core library (cdylib)
echo "  Building sdlretro-core..."
cargo build --release --package sdlretro-core --target "${TARGET}"

# Build the frontend binary
echo "  Building sdlretro-frontend..."
cargo build --release --package sdlretro-frontend --target "${TARGET}"

echo ""
echo "=========================================="
echo " Build complete!"
echo "=========================================="
echo ""
echo "Artifacts:"
echo "  Core (shared lib): ${OUTPUT_DIR}/libsdlretro_core.so"
echo "  Frontend (binary): ${OUTPUT_DIR}/sdlretro-frontend"
echo ""
echo "Deploy to target device:"
echo "  scp ${OUTPUT_DIR}/libsdlretro_core.so ${OUTPUT_DIR}/sdlretro-frontend user@device:/"
echo ""
