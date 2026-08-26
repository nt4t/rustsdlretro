#!/bin/bash
# PSX launcher with WebSocket API support
# For CD images, point to the cue sheet or chd file:
#   ./start_psx.sh
#
# WebSocket control on port 18932 — use --api-port to change
# Clients can send: step, play, pause, save_state, load_state, input, set_option

./target/release/rustsdlretro-frontend \
    ~/src/beetle-psx-libretro/mednafen_psx_libretro.so \
    "/mnt/d/Games/Roms/ps1/Soukyuu Gurentai - Oubu Shutsugeki (Japan).chd" \
    --api-port 18932
