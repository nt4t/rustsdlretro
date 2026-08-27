# rustsdlretro

Simple rust libretro frontend for Linux. Runs retro game emulators on:
- **Embedded Linux** (Raspberry Pi, RetroPie) via framebuffer with no X11 needed
- **Desktop Linux** via X11 windowed mode

## Features

- **Dual video backends** - Framebuffer (`/dev/fb0`) for embedded; X11 windowed (minifb) for desktop
- **Libretro core support** - Reuses existing `.so` cores (snes9x, Genesis-Plus-GX, mGBA, FCEUmm, etc.)
- **Menu overlay** - Browse and modify core options with ESC key, navigation arrows, value cycling
- **Dynamic resolution** - Handles cores that change resolution during gameplay (letterboxing)
- **Audio playback** - ALSA PCM output with ring buffer; null driver fallback for testing
- **Keyboard input** - evdev `/dev/input/event0` (embedded) or minifb keyboard polling (desktop)
- **Embedded bitmap fonts** - 8px and 16px tall fonts, no external font files needed
- **Core options v1/v2** - Full support for libretro core options API
- **JSON configuration** - Renderer selection, window settings, input device (requires `--features config`)
- **ZIP ROM loading** - Automatic extraction of ROMs from ZIP archives
- **Launcher scripts** - Quick launch scripts for NES, SNES, and PSX games
- **WebSocket API** - Remote control via WebSocket: play/pause/step, save/load state, gamepad input (requires `api` feature)

## Architecture

```
rustsdlretro/
├── rustsdlretro-core/          # Core library
│   ├── lib.rs              # Core lifecycle, FFI bindings, environment callback
│   ├── api.rs              # WebSocket control API (optional)
│   ├── video.rs            # VideoBackend trait + FbdevVideo: mmap framebuffer, pixel conversion
│   ├── video_minifb.rs     # MinifbVideo: X11 windowed backend, buffer rendering
│   ├── audio.rs            # ALSA audio driver with ring buffer
│   ├── audio_null.rs       # Null/silent audio driver fallback
│   ├── config.rs           # JSON config parsing, renderer selection
│   ├── input.rs            # InputReader: evdev polling + minifb keyboard polling
│   ├── font.rs             # Bitmap font renderer with embedded glyph data
│   ├── core_options.rs     # Core options v1/v2 API, value storage
│   ├── gui.rs              # Menu overlay, navigation, rendering
│   └── build.rs            # bindgen for libretro.h
├── rustsdlretro-frontend/      # Binary crate
│   └── main.rs             # CLI entry point, config loading, backend selection, main loop
├── start_nes.sh              # Launcher script for NES games
├── start_snes.sh             # Launcher script for SNES games
├── start_psx.sh              # Launcher script for PSX games
└── doc/                    # Design documents
```

## Building

### Prerequisites

- Rust toolchain (edition 2021)
- Linux with framebuffer support (`/dev/fb0`) **or** X11 (for minifb)
- ALSA development libraries
- X11 development libraries (for minifb: `libx11-dev`)
- Cross-compile toolchain for ARM (optional, for Raspberry Pi)

### Build

**Framebuffer-only (embedded)**:
```bash
cargo build --release
```

**Desktop with X11 window + JSON config**:
```bash
cargo build --release --features minifb,config
```

**Feature flags**:

| Crate | Flag | Description |
|-------|------|-------------|
| `rustsdlretro-core` | `default = ["fbdev"]` | Framebuffer backend only |
| `rustsdlretro-core` | `fbdev` | Framebuffer (`/dev/fb0`) video output |
| `rustsdlretro-core` | `minifb = ["dep:minifb"]` | X11 windowed backend via minifb |
| `rustsdlretro-core` | `config = ["dep:serde", "dep:serde_json"]` | JSON config file parsing |
| `rustsdlretro-core` | `null-audio` | Null/silent audio driver fallback |
| `rustsdlretro-core` | `api = ["dep:tungstenite", "dep:png", "dep:tokio"]` | WebSocket control API |
| `rustsdlretro-frontend` | `minifb` | Enables minifb keyboard polling |
| `rustsdlretro-frontend` | `config` | Enables config-based renderer selection |
| `rustsdlretro-frontend` | `null-audio` | Enables null audio driver fallback |

**Default build**: `cargo build --release` produces a framebuffer-only binary.

### Common Build Combinations

| Use Case | Command |
|----------|---------|
| **Desktop (X11 window only)** | `cargo build --release --features minifb` |
| **Desktop with config file** | `cargo build --release --features "minifb,config"` |
| **Full desktop + null audio fallback** | `cargo build --release --features "minifb,config,null-audio"` |
| **Desktop with WebSocket API** | `cargo build --release --features "minifb,api"` |
| **Embedded (Raspberry Pi)** | `cargo build --release` |
| **Cross-compile for RPi** | `cargo build --release --target armv7-unknown-linux-gnueabihf`

### Cross-compile for Raspberry Pi

```bash
rustup target add armv7-unknown-linux-gnueabihf
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## Usage

```bash
./target/release/rustsdlretro <core.so> <game.rom>
```

### WebSocket API (requires `api` feature)

Built-in WebSocket server on port **18932** for remote control:

```bash
# Start with API enabled
./target/release/rustsdlretro <core.so> <game.rom> --api-port 18932
```

**Client → Server messages (JSON):**

| Type | Payload | Description |
|------|---------|-------------|
| `input` | `{port, buttons: {up, down, left, right, a, b, x, y, l, r, start, select}}` | Joypad state snapshot |
| `step` | `{}` | Run one frame (pauses after) |
| `play` | `{}` | Resume continuous playback |
| `pause` | `{}` | Pause emulation |
| `save_state` | `{}` | Trigger save state (F2 equivalent) |
| `load_state` | `{}` | Trigger load state (F4 equivalent) |

**Server → Client messages:**

```json
{"type": "status", "running": true, "fps": 59.94, "width": 320, "height": 240}
{"type": "frame_done"}          // step ack
{"type": "flash", "message": "State Saved"}
```

**Test script:** `node test_api.js` — exercises the full API flow (Play → Save → Load → Step).

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

**Note**: Config file support requires the `config` feature flag at compile time.

## Supported Cores

Tested with:
- snes9x2010 (SNES)
- Genesis-Plus-GX (Sega Genesis)
- mGBA (Game Boy Advance)
- Beetle PSX-HW / Mednafen PSX (PlayStation 1)
- FCEUmm (NES/Famicom)

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
- ALSA audio playback with ring buffer
- Null/silent audio driver fallback
- Frame throttling with drift correction
- Dynamic resolution handling
- Bitmap font renderer
- Core options v1/v2 support
- GUI menu overlay with scrolling
- Optimized overlay rendering (bulk memory writes, no flicker)
- JSON configuration system with renderer selection
- ZIP ROM loading (extracts ROM from .zip archives automatically)
- Deferred audio rate change handling for stable emulation
- Save states / SRAM persistence (F2/F4 keys)
- Log format string expansion via C shim (vsnprintf)
- WebSocket control API with JSON message protocol
  - Remote play/pause/frame-step controls
  - Save/load state over the wire
  - Gamepad input mapping from web clients
  - Thread-safe shared state architecture

### Pending
- Language file loading (i18n)
- ROM browser / core selector

## Testing

### WebSocket API Test
```bash
npm install ws   # one-time setup
cargo build --release --features "minifb,api"
# Start emulator in one terminal, then:
node test_api.js
```

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
