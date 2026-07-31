# rustsdlretro

Simple rust libretro frontend for Linux framebuffer devices. Runs retro game emulators on embedded Linux hardware (Raspberry Pi, retroPie devices) without X11 or SDL dependencies.

## Features

- **Dual video backends** - Framebuffer (`/dev/fb0`) or X11 windowed (minifb) via config
- **Libretro core support** - Reuses existing `.so` cores (snes9x, Genesis-Plus-GX, mGBA, etc.)
- **Menu overlay** - Browse and modify core options with ESC key, navigation arrows, value cycling
- **Dynamic resolution** - Handles cores that change resolution during gameplay (letterboxing)
- **Audio playback** - ALSA PCM output with ring buffer
- **Keyboard input** - evdev `/dev/input/event0` with SNES gamepad mapping
- **Embedded bitmap fonts** - 8px and 16px tall fonts, no external font files needed
- **Core options v1/v2** - Full support for libretro core options API
- **JSON configuration** - Renderer selection, window settings, CLI `--config` override

## Architecture

```
rustsdlretro/
├── rustsdlretro-core/          # Core library
│   ├── lib.rs              # Core lifecycle, FFI bindings, environment callback
│   ├── video.rs            # VideoBackend trait + FbdevVideo: mmap framebuffer, pixel conversion
│   ├── video_minifb.rs     # MinifbVideo: X11 windowed backend, buffer rendering
│   ├── config.rs           # JSON config parsing, renderer selection
│   ├── input.rs            # InputReader: evdev polling, key mapping, menu helpers
│   ├── font.rs             # Bitmap font renderer with embedded glyph data
│   ├── core_options.rs     # Core options v1/v2 API, value storage
│   ├── gui.rs              # Menu overlay, navigation, rendering
│   └── build.rs            # bindgen for libretro.h
├── rustsdlretro-frontend/      # Binary crate
│   └── main.rs             # CLI entry point, config loading, backend selection, main loop
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

### Build with X11 windowed backend

```bash
cargo build --release --features minifb,config
```

**Feature flags**:
- `default = ["fbdev"]` — framebuffer backend only
- `minifb = ["dep:minifb"]` — adds X11 windowed backend
- `config = ["dep:serde", "dep:serde_json"]` — adds JSON config parsing

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

### Config File (`config.json`)

```json
{
    "renderer": "fbdev",
    "window": {
        "width": 640,
        "height": 480,
        "scale": 2,
        "borderless": false,
        "title": "rustsdlretro"
    },
    "input": {
        "device": "/dev/input/event0"
    }
}
```

**Fields**:
- `renderer`: `"fbdev"` (framebuffer) or `"minifb"` (X11 windowed)
- `window.*`: Only used when `renderer = "minifb"` (width, height, scale, borderless, title)
- `input.device`: Input device path (default: `/dev/input/event0`)

**CLI override**: `--config <path>` to specify a custom config file location.

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
- X11 windowed video output (minifb) with scale modes
- Abstracted VideoBackend trait for backend-agnostic rendering
- evdev keyboard input with gamepad mapping
- ALSA audio playback
- Frame throttling with drift correction
- Dynamic resolution handling
- Bitmap font renderer
- Core options v1/v2 support
- GUI menu overlay with scrolling
- Optimized overlay rendering (bulk memory writes, no flicker)
- JSON configuration system with renderer selection

### Pending
- ZIP ROM loading
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
