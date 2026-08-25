#!/bin/bash
# PSX launcher - supports .chd and .cue/.bin files
# For CD images, point to the cue sheet or chd file:
#   ./target/release/rustsdlretro-frontend <core.so> game.chd

./target/release/rustsdlretro-frontend \
    ~/src/beetle-psx-libretro/mednafen_psx_libretro.so \
    "/mnt/d/Games/Roms/ps1/Soukyuu Gurentai - Oubu Shutsugeki (Japan).chd"
