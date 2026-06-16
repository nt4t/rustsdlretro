# rustsdlretro

Simple rust libretro frontend for Linux framebuffer devices. Runs retro game emulators on embedded Linux hardware (Raspberry Pi, retroPie devices) without X11 or SDL dependencies.

## Features

- **Direct framebuffer rendering** - No X11, no SDL, direct `/dev/fb0` access
- **Libretro core support** - Reuses existing `.so` cores (snes9x, Genesis-Plus-GX, mGBA, etc.)
- **Menu overlay** - Browse and modify core options with ESC key, navigation arrows, value cycling
- **Dynamic resolution** - Handles cores that change resolution during gameplay (letterboxing)
- **Audio playback** - ALSA PCM output with ring buffer
- **Keyboard input** - evdev `/dev/input/event0` with SNES gamepad mapping
- **Embedded bitmap fonts** - 8px and 16px tall fonts, no external font files needed
- **Core options v1/v2** - Full support for libretro core options API

## Architecture

```
rustsdlretro/
├── rustsdlretro-core/          # Core library
│   ├── lib.rs              # Core lifecycle, FFI bindings, environment callback
│   ├── video.rs            # FbdevVideo: mmap framebuffer, pixel conversion, letterboxing
│   ├── input.rs            # InputReader: evdev polling, key mapping, menu helpers
│   ├── font.rs             # Bitmap font renderer with embedded glyph data
│   ├── core_options.rs     # Core options v1/v2 API, value storage
│   ├── gui.rs              # Menu overlay, navigation, rendering
│   └── build.rs            # bindgen for libretro.h
├── rustsdlretro-frontend/      # Binary crate
│   └── main.rs             # CLI entry point, main loop, GUI integration
└── doc/                    # Design documents
```

## Building

### Prerequisites

- Rust toolchain (edition 2021)
- Linux with framebuffer support (`/dev/fb0`)
- ALSA development libraries
- Cross-compile toolchain for ARM (optional, for Raspberry Pi)

### Build

```bash
cargo build --release
```

### Cross-compile for Raspberry Pi

```bash
rustup target add armv7-unknown-linux-gnueabihf
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## Usage

```bash
./target/release/rustsdlretro <core.so> <game.rom>
```

Example:

```bash
./target/release/rustsdlretro ~/snes9x2010_libretro.so ~/roms/snes/Super\ Mario\ World.smc
```

### Controls

| Key | Action |
|-----|--------|
| ESC | Toggle menu open/close |
| ↑/↓ | Navigate menu items |
| ←/→ | Cycle option values |
| Space | Cycle option values |

## Configuration

System directory: `~/.config/rustsdlretro/`

## Supported Cores

Tested with:
- snes9x2010 (SNES)
- Genesis-Plus-GX (Sega Genesis)
- mGBA (Game Boy Advance)

Any libretro core should work if it supports the standard libretro API.

## Target Devices

- Raspberry Pi (ARMv7)
- RetroPie devices
- Any Linux system with framebuffer and ALSA

## Project Status

### Completed
- Core loading and lifecycle management
- Framebuffer video output with letterboxing
- evdev keyboard input with gamepad mapping
- ALSA audio playback
- Frame throttling with drift correction
- Dynamic resolution handling
- Bitmap font renderer
- Core options v1/v2 support
- GUI menu overlay with scrolling

### Pending
- ZIP ROM loading
- Configuration file system
- Save states / SRAM persistence
- Language file loading (i18n)
- ROM browser with core selector

## Development

### Workspace Structure

This is a Cargo workspace with two crates:
- `rustsdlretro-core` - Core library (FFI, video, input, audio, GUI)
- `rustsdlretro-frontend` - Binary executable

### Running

```bash
cargo run --release -- <core.so> <game.rom>
```

## License

See [LICENSE](LICENSE) file.

## Acknowledgments

Based on the original sdlretro C++ project. Libretro cores © their respective authors.
