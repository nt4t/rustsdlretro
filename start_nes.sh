#!/bin/bash
# NES launcher - supports both .nes and .zip files
# For ZIP archives, just replace the ROM path with your .zip file:
#   ./target/release/rustsdlretro-frontend <core.so> game.zip

./target/release/rustsdlretro-frontend \
    ~/src/libretro-fceumm/fceumm_libretro.so \
    "/mnt/d/Games/Roms/nes/Contra\ \(USA\).zip"
