#!/bin/bash
set -euo pipefail

TARGET="${1:-armv7-unknown-linux-gnueabihf}"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="${PROJECT_DIR}/target/cross/${TARGET}/release"

echo "=========================================="
echo " Cross-compile rustsdlretro"
echo " Target: ${TARGET}"
echo "=========================================="

# Map Rust target triples to Debian cross-compiler package names
declare -A TARGET_TO_DEB_PKG
TARGET_TO_DEB_PKG["armv7-unknown-linux-gnueabihf"]="arm-linux-gnueabihf"
TARGET_TO_DEB_PKG["armv7-unknown-linux-musleabihf"]="arm-linux-gnueabihf"
TARGET_TO_DEB_PKG["aarch64-unknown-linux-gnu"]="aarch64-linux-gnu"
TARGET_TO_DEB_PKG["aarch64-unknown-linux-musl"]="aarch64-linux-gnu"
TARGET_TO_DEB_PKG["x86_64-unknown-linux-gnu"]="x86_64-linux-gnu"
TARGET_TO_DEB_PKG["x86_64-unknown-linux-musl"]="x86_64-linux-gnu"
TARGET_TO_DEB_PKG["i686-unknown-linux-gnu"]="i686-linux-gnu"
TARGET_TO_DEB_PKG["riscv64gc-unknown-linux-gnu"]="riscv64-linux-gnu"
TARGET_TO_DEB_PKG["thumbv7neon-unknown-linux-gnueabihf"]="arm-linux-gnueabihf"
TARGET_TO_DEB_PKG["thumbv7neon-unknown-linux-musleabihf"]="arm-linux-gnueabihf"

get_deb_prefix() {
    local target="$1"
    echo "${TARGET_TO_DEB_PKG[$target]:-$target}"
}

# Install cross-compilation toolchain
echo "[1/4] Installing cross-compilation toolchain..."
DEB_PREFIX="$(get_deb_prefix "${TARGET}")"
if ! command -v "${DEB_PREFIX}-gcc" &> /dev/null; then
    echo "  Installing gcc for ${TARGET} (packages: ${DEB_PREFIX}-gcc)..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y "${DEB_PREFIX}-gcc" "${DEB_PREFIX}-g++"
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm "${DEB_PREFIX}-gcc"
    elif command -v brew &> /dev/null; then
        brew install FiloSottile/musl-cross/musl-cross
    else
        echo "  ERROR: Cannot detect package manager. Please install ${DEB_PREFIX}-gcc manually."
        exit 1
    fi
fi

# Install cross-compilation dependencies
echo "[2/4] Installing cross-compilation dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get install -y \
        pkg-config \
        libasound2-dev:${DEB_PREFIX} \
        libudev-dev:${DEB_PREFIX} 2>/dev/null || true
fi

# Set up cross-compilation environment
echo "[3/4] Setting up cross-compilation environment..."
export CC="${DEB_PREFIX}-gcc"
export CXX="${DEB_PREFIX}-g++"
export PKG_CONFIG_LIBDIR="/usr/${DEB_PREFIX}/lib/pkgconfig:/usr/${DEB_PREFIX}/share/pkgconfig:/usr/lib/${DEB_PREFIX}/pkgconfig"
export PKG_CONFIG_ALLOW_CROSS=1
echo "  CC=${CC}"
echo "  PKG_CONFIG_LIBDIR=${PKG_CONFIG_LIBDIR}"

# Install Rust target
echo "  Adding Rust target ${TARGET}..."
rustup target add "${TARGET}" 2>/dev/null || true

# Build
echo "[4/4] Building..."
cd "${PROJECT_DIR}"

# Use separate target directory to avoid mixing with native builds
export CARGO_TARGET_DIR="${PROJECT_DIR}/target/cross"

# Clean previous cross-build artifacts for this target
echo "  Cleaning previous artifacts for ${TARGET}..."
rm -rf "${CARGO_TARGET_DIR}/${TARGET}/release"

# Build the core library (cdylib)
echo "  Building rustsdlretro-core..."
cargo build --release --package rustsdlretro-core --target "${TARGET}"

# Build the frontend binary
echo "  Building rustsdlretro-frontend..."
cargo build --release --package rustsdlretro-frontend --target "${TARGET}"

echo ""
echo "=========================================="
echo " Build complete!"
echo "=========================================="
echo ""
echo "Artifacts:"
echo "  Core (shared lib): ${OUTPUT_DIR}/libsdlretro_core.so"
echo "  Frontend (binary): ${OUTPUT_DIR}/rustsdlretro-frontend"
echo ""
echo "Deploy to target device:"
echo "  scp ${OUTPUT_DIR}/libsdlretro_core.so ${OUTPUT_DIR}/rustsdlretro-frontend user@device:/"
echo ""
