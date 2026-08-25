#!/bin/bash
# Genesis launcher - supports both .gen/.md and .zip files
# For ZIP archives, just replace the ROM path with your .zip file:
#   ./target/release/rustsdlretro-frontend <core.so> game.zip

./target/release/rustsdlretro-frontend \
    ~/src/Genesis-Plus-GX/genesis_plus_gx_libretro.so \
    "/mnt/d/Games/Roms/gen/MUSHA - Metallic Uniframe Super Hybrid Armor (USA).zip"
