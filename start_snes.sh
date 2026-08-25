#!/bin/bash
# SNES launcher - supports both .sfc/.smc and .zip files
# For ZIP archives, just replace the ROM path with your .zip file:
#   ./target/release/rustsdlretro-frontend <core.so> game.zip

./target/release/rustsdlretro-frontend \
    ~/src/snes9x2010/snes9x2010_libretro.so \
    "$HOME/rom/snes/Zelda\ -\_Alink_to_the_Past.sfc.zip"
