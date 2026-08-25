#!/bin/bash
# Quick test script for ZIP ROM support
# Usage: ./test_zip.sh <core_path> <zip_file> [scale]
#
# Examples:
#   ./test_zip.sh ~/src/libretro-fceumm/fceumm_libretro.so /path/to/contrazip.zip
#   ./test_zip.sh ~/src/snes9x2010/snes9x2010_libretro.so ~/roms/snes/zelda.zip 640

set -e

CORE="${1:?Usage: $0 <core_path> <zip_file> [scale]}"
ZIP="${2:?Missing ZIP file path}"
SCALE="${3:-640}"

if [ ! -f "$CORE" ]; then
    echo "Error: Core not found: $CORE"
    exit 1
fi

if [ ! -f "$ZIP" ]; then
    echo "Error: ZIP file not found: $ZIP"
    exit 1
fi

echo "=== Testing ZIP ROM Support ==="
echo "Core: $CORE"
echo "ZIP:  $ZIP"
echo ""

./target/release/rustsdlretro-frontend \
    --features minifb,config \
    "$CORE" \
    "$ZIP"
