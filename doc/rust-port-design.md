# Rust Port Design - sdlretro

## Overview
Port the `sdlretro` frontend to Rust, targeting the Linux framebuffer and `/dev/input/event0` directly, removing the SDL dependency. The Rust port reuses the libretro cores (`.so` files) unchanged via FFI, while replacing all SDL1/SDL2/FBDEV C++ backends with idiomatic Rust code.

## Goals
- **Zero SDL dependency**: Direct Linux kernel interfaces only (`/dev/fb0`, `/dev/input/event*`, `/dev/snd/`)
- **Reuse existing libretro cores**: FFI to `.so` cores, no core rewriting
- **Preserve feature parity**: Menu system, ZIP ROM loading, configuration, i18n, core variables, save states, performance monitoring
- **Target devices**: Raspberry Pi (ARM), x86_64 Linux desktop
- **Memory safety**: No unsafe code except explicit FFI boundary wrappers

## Current Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Foundation | ✅ DONE | Workspace, sdlretro-core, bindgen, core loading/lifecycle |
| Phase 2: Video (FBDEV) | ✅ DONE | FbdevVideo with mmap, 1:1 output, letterboxing, pixel format from core |
| Phase 3: Input | ✅ DONE | evdev crate, background thread, shared Arc<Mutex<InputState>> |
| Dynamic Resolution | ✅ DONE | Handles RETRO_ENVIRONMENT_SET_GEOMETRY (key 37) and SET_SYSTEM_AV_INFO (key 32) |
| Phase 3: Audio | ✅ DONE | ALSA PCM output, ring buffer, playback thread, sample rate changes |
| Throttle timing | ✅ DONE | clock_gettime(CLOCK_MONOTONIC), drift-correct, frame skip |
| Cross-compile | ✅ DONE | thumbv7neon-unknown-linux-gnueabihf target configured |
| Font renderer | ✅ DONE | Bitmap font with embedded glyph data, shadow rendering, text measurement |
| Core options | ✅ DONE | v1/v2 API support, runtime option changes via GET_VARIABLE_UPDATE |
| Phase 4: UI System | ✅ DONE | Menu overlay with ESC toggle, option browsing, value cycling, debounce |
| Phase 5: Full Integration | 🔲 TODO | Config, i18n, save states, ZIP ROM loading |
| Phase 6: Hardening | 🔲 TODO | Performance, device testing, packaging |

### Implemented Features
- **Core loading**: dlopen + bindgen FFI, retro_get_system_info, retro_set_environment, retro_init
- **ROM loading**: File read + pass data to core when need_fullpath is false
- **Video**: FbdevVideo struct, fbdev ioctl, mmap, push_frame with letterboxing
- **Input**: InputReader using evdev crate, background thread, SNES keyboard mapping
- **Frame timing**: Throttle with clock_gettime(CLOCK_MONOTONIC), drift-correct
- **FPS output**: Console print every 5 seconds with frame count and actual FPS
- **Continuous loop**: Main loop runs indefinitely, Ctrl+C exit via SIGINT handler
- **Dynamic resolution**: Handles SET_GEOMETRY and SET_SYSTEM_AV_INFO from core, updates letterboxing and throttle in real-time
- **Audio**: ALSA PCM playback, ring buffer (8192 samples), background thread, sample rate changes via SET_SYSTEM_AV_INFO
- **Cross-compile**: .cargo/config.toml for armv7 target
- **Font renderer**: Embedded bitmap fonts (8px/16px tall), shadow rendering, text measurement, XRGB8888/RGB565 support
- **Core options**: v1/v2 API support, SET_CORE_OPTIONS_V2_INTL, runtime option changes via GET_VARIABLE_UPDATE
- **GUI overlay**: ESC toggle menu, option browsing, value cycling (left/right arrows/space), navigation debounce, fallback rendering

### Pending Features
- **Audio**: ALSA PCM output with resampling (libsamplerate)
- **Config**: sdlretro.json loader/writer
- **Save states**: SRAM/RTC persistence
- **ZIP ROM loading**: miniz_oxide extraction
- **i18n**: Language file loading
- **Core selector**: ROM browser with core matching

## Architecture

### Crate Layout

```
sdlretro/
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock
│
├── sdlretro-core/                # Libretro core management (FFI)
│   ├── src/
│   │   ├── lib.rs                # Core struct, ResolutionState, Throttle, FFI bindings, env callback
│   │   ├── video.rs              # FbdevVideo: mmap fb0, pixel conversion, letterboxing, overlay helpers
│   │   ├── input.rs              # InputReader: evdev polling thread, key mapping, menu helpers
│   │   ├── font.rs               # Bitmap font renderer: glyph data, text rendering, shadow, measurement
│   │   ├── core_options.rs       # Core options: v1/v2 FFI bindings, variable parsing, value storage
│   │   └── gui.rs                # GUI overlay: menu state machine, navigation, rendering, input handling
│   └── build.rs                  # bindgen invocation, copy libretro.h
│
├── sdlretro-frontend/            # Binary crate: main entry point
│   ├── src/
│   │   └── main.rs               # CLI args, driver init, main loop, GUI integration
│   └── Cargo.toml
│
└── doc/
    └── libretro.h                # Vendored libretro header for bindgen
```

### Workspace Dependencies

```toml
# Workspace Cargo.toml (shared across crates)
[workspace.dependencies]
# FFI / OS
libc = "0.2"

# Input
evdev = "0.12"

# Build-time
bindgen = "0.71"
```

### Core Libretro Integration

#### FFI Bindings
- **Source**: Vendored `doc/libretro.h`
- **Generation**: `build.rs` runs `bindgen` to produce `bindings.rs`
- **Safety**: Bindings wrapped in safe Rust structs; all `unsafe` blocks are localized to the `sdlretro-core` crate

#### Core Lifecycle
```rust
pub struct Core {
    handle: *mut c_void,            // dlopen handle
    need_fullpath: bool,            // From retro_get_system_info
    resolution: Arc<Mutex<ResolutionState>>,
}

impl Core {
    pub fn new(path: &Path) -> Result<Self, CoreError>;
    pub fn init(&mut self) -> Result<(), CoreError>;
    pub fn load_game(&mut self, game: &Path) -> Result<(), CoreError>;
    pub fn run(&mut self) -> Result<(), CoreError>;
    pub fn unload_game(&mut self);
    pub fn get_system_av_info(&self) -> retro_system_av_info;
    pub fn unload(&mut self);
}
```

#### Callback Registration
```rust
// In Core::init()
let set_env: RetroSetEnvironmentFn = ...;
unsafe { set_env(Some(log_environment_cb)) };

let set_vr: RetroSetVideoRefreshFn = ...;
unsafe { set_vr(None) };

let set_audio: RetroSetAudioSampleFn = ...;
unsafe { set_audio(Some(audio_sample_cb)) };

let set_audio_batch: RetroSetAudioSampleBatchFn = ...;
unsafe { set_audio_batch(Some(audio_sample_batch_cb)) };
```

#### Environment Commands
Currently handled in `log_environment_cb`:

| Key | Define | Status |
|-----|--------|--------|
| 10 | `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT` | ✅ Implemented - updates CORE_FORMAT.bpp |
| 13 | `RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY` | ✅ Implemented - returns static path |
| 15 | `RETRO_ENVIRONMENT_GET_VARIABLE` | ✅ Implemented - returns current option value from v2_values/old_values |
| 16 | `RETRO_ENVIRONMENT_SET_VARIABLES` | ✅ Implemented - old-style variable parsing |
| 17 | `RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE` | ✅ Implemented - returns VARIABLE_UPDATE_PENDING flag, reset after read |
| 31 | `RETRO_ENVIRONMENT_GET_LOG_INTERFACE` | ✅ Implemented - registers log callback |
| 32 | `RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO` | ✅ Implemented - updates resolution + fps |
| 37 | `RETRO_ENVIRONMENT_SET_GEOMETRY` | ✅ Implemented - updates resolution, called from retro_run() |
| 52 | `RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION` | ✅ Implemented - returns v2 API version |
| 53 | `RETRO_ENVIRONMENT_SET_CORE_OPTIONS` | ✅ Implemented - v1 option definitions |
| 55 | `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY` | ✅ Implemented - visibility toggles |
| 67 | `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2` | ✅ Implemented - v2 option definitions |
| 68 | `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL` | ✅ Implemented - v2 with intl support |

Future commands to implement:
- `SET_HW_RENDER`, `GET_PREFERRED_HW_RENDER`
- `SET_INPUT_DESCRIPTORS`
- `GET_VFS_INTERFACE`
- `SET_MESSAGE`, `SET_MESSAGE_EXT`
- `SET_SERIALIZATION_QUIRKS`

### Core Options Architecture

Three-layer design matching the original C++ sdlretro:

1. **libretro.h FFI** — `retro_core_option_value`, `retro_core_option_definition`, `retro_core_option_v2_definition`, `retro_variable`
2. **core_options.rs** — `CoreOptions` struct with `v2_values` HashMap for runtime values, `get_current_value()` with fallback chain
3. **Environment callback** — Handles keys 52/53/55/67/68 for option definitions, key 15 for GET_VARIABLE, key 17 for GET_VARIABLE_UPDATE

**Runtime option changes flow**:
```
User changes option in GUI
  → gui.rs sets core_opts.set_v2_value(key, value)
  → gui.rs sets VARIABLE_UPDATE_PENDING = true
  → Core polls GET_VARIABLE_UPDATE (key 17) → returns true
  → Core calls GET_VARIABLE (key 15) → returns new value from v2_values
  → Core's check_variables() applies the new setting
```

**Storage**: `CoreOptions.v2_values` HashMap (key → value string), populated during `retro_set_environment()` via SET_CORE_OPTIONS_V2_INTL, updated by GUI when user changes options.

### Graphics Driver (FBDEV)

#### Framebuffer Access
```rust
pub struct FbdevVideo {
    fb_fd: c_int,                   // /dev/fb0 file descriptor
    fb_ptr: *mut u8,                // mmap'd framebuffer memory
    fb_len: usize,                  // mmap length
    fb_width: u32,                  // Framebuffer resolution (from ioctl)
    fb_height: u32,
    fb_pitch: u32,                  // Bytes per row
    fb_bpp: u32,
    core_width: u32,                // Cached core resolution (for logging)
    core_height: u32,
    skip_frame: bool,
    frame_drawn: bool,
}
```

- **mmap**: `libc::mmap` with `PROT_READ | PROT_WRITE`, `MAP_SHARED`
- **ioctl**: `FBIOGET_VSCREENINFO`, `FBIOGET_FSCREENINFO`
- **Pixel conversion**: XRGB8888 ↔ RGB565 (bit manipulation, no SIMD yet)
- **Letterboxing**: Computed per-frame from actual core resolution

#### Video Refresh Callback
```rust
extern "C" fn video_refresh_cb(pixels: *const c_void, w: u32, h: u32, pitch: usize) {
    // w/h are the ACTUAL dimensions from the core for this frame
    MAIN_VIDEO.push_frame(pixels, w, h, pitch);
}
```

The core resolution is used directly from the callback arguments, not cached values. This ensures correct rendering when cores change resolution dynamically.

### Audio Driver (ALSA)

Real ALSA PCM output with ring buffer and background playback thread.

```rust
extern "C" fn audio_sample_cb(left: i16, right: i16) {
    MAIN_AUDIO.push_stereo_pair(left, right);
}

extern "C" fn audio_sample_batch_cb(data: *const i16, frames: usize) -> usize {
    let slice = std::slice::from_raw_parts(data, frames * 2);
    MAIN_AUDIO.push_batch(slice);
    frames
}
```

- **Ring buffer**: 8192 samples (16384 bytes stereo), mutex-protected
- **Playback thread**: Drains ring buffer into ALSA PCM at 1024-frame batches
- **Sample rate**: Configured from core's `sample_rate` via SET_SYSTEM_AV_INFO
- **Underrun recovery**: `snd_pcm_recover` on EPIPE/XRUN

### Input Driver (EVDEV)

```rust
pub struct InputReader {
    device: Device,                   // evdev Device (keyboard)
    state: Arc<Mutex<InputState>>,   // Shared mutable input state
    reader: JoinHandle<()>,          // Background polling thread
}

pub struct InputState {
    pub buttons: [i16; 128],         // Last frame button states
}
```

- **Device**: Opens `/dev/input/event0` (keyboard)
- **Polling thread**: Reads evdev events, maps to `RETRO_DEVICE_ID_JOYPAD_*`
- **State sharing**: `Arc<Mutex<InputState>>` shared between polling thread and main loop
- **Keymap**: Linux `EV_KEY` codes → `RETRO_DEVICE_ID_JOYPAD_*` mapping

### Dynamic Resolution Handling

Some libretro cores (Genesis-Plus-GX, snes9x, etc.) change resolution during gameplay. Example: MUSHA switches between 256x192 and 320x224.

**Implementation:**

1. **`ResolutionState`**: Shared `Arc<Mutex<ResolutionState>>` with width, height, fps fields
2. **Environment callback**: Handles keys 32 (`SET_SYSTEM_AV_INFO`) and 37 (`SET_GEOMETRY`)
   - Updates `ResolutionState` with new dimensions
   - Calls `FbdevVideo::set_core_format()` to recompute letterboxing offsets
   - Logs "Resolution changed: WxH @ FPS"
3. **Frame rendering**: `push_frame()` uses actual `w`/`h` from `video_refresh_cb` arguments, not cached values
4. **Throttle update**: Main loop detects FPS changes and recreates `Throttle`

```rust
// In log_environment_cb for keys 32/37:
if let Some(state) = RESOLUTION_STATE.get() {
    let mut s = state.lock().unwrap();
    let changed = s.width != w || s.height != h || s.fps != fps;
    s.width = w; s.height = h; s.fps = fps;
    drop(s);
    if changed {
        eprintln!("Resolution changed: {}x{} @ {:.2} FPS", w, h, fps);
        if let Some(ref mut video) = MAIN_VIDEO {
            video.set_core_format(w, h, CORE_FORMAT.bpp);
        }
    }
}
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        Main Loop (main.rs)                  │
│                                                             │
│  while RUNNING.load() {                                     │
│    1. gui.handle_input()            // Menu input handling  │
│       ├── ESC: toggle menu open/close                      │
│       ├── arrows: navigate options                         │
│       ├── left/right/space: cycle values                   │
│       └── sets VARIABLE_UPDATE_PENDING when options change │
│    2. if !menu_open: core.run()     // retro_run()          │
│       ├── retro_input_poll_cb()  → input.poll()            │
│       ├── retro_video_refresh_cb() → video.push_frame()    │
│       ├── retro_audio_sample_cb()  → audio.push_stereo()   │
│       ├── retro_audio_sample_batch_cb() → audio.push_batch │
│       └── retro_input_state_cb()   → input.read_button()   │
│    3. gui.render()                  // Overlay rendering    │
│       └── draws menu on framebuffer if menu_open           │
│    4. throttle.check_wait()           // Frame timing       │
│       ├── usecs > 0 → sleep loop (re-check)                │
│       └── usecs <= 0 → set_skip_frame() (frame skip)       │
│    5. FPS counter (print every 5s)                          │
│  }                                                          │
│  Exit: SIGINT → RUNNING.store(false)                        │
└─────────────────────────────────────────────────────────────┘
         ▲                          │
         │                          ▼
┌───────────────────┐    ┌──────────────────────────┐
│  Input Thread     │    │  Video Driver (fbdev)    │
│  (input.rs)       │    │  (video.rs)              │
│                   │    │                          │
│  poll /dev/input  │    │  mmap /dev/fb0           │
│  event0           │    │  push_frame()            │
│                   │    │  pixel_convert()           │
│  update Mutex     │    │  letterboxing            │
│  InputState       │    │  overlay drawing         │
└───────────────────┘    └──────────────────────────┘
```

## Configuration & Persistence

### Config File
Path: `{store_dir}/cfg/sdlretro.json` (same as C++ implementation)

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    resolution: Resolution,
    scale: f32,
    fullscreen: bool,
    frame_limit: bool,
    frame_delay: u32,
    audio_volume: u32,
    audio_device: Option<String>,
    language: String,
    save_check_interval: u32,
    core_dirs: Vec<PathBuf>,
    rom_dirs: Vec<PathBuf>,
    store_dir: PathBuf,
}
```

### Save States
- **SRAM**: `{store_dir}/saves/{core_name}/{game_name}.sav`
- **RTC**: `{store_dir}/saves/{core_name}/{game_name}.rtc`
- Periodic save checked against `save_check_interval` config

### Core Discovery
- Scan `core_dirs` for `*.so` files
- Load each core, call `retro_get_system_info()` to get library name, version, extensions
- Match ROM file extension to core extensions (same logic as C++ `core_manager.cpp`)

## Error Handling

- **Root error type**: `thiserror::Error` hierarchy per crate
- **High-level reporting**: `anyhow::Result` at binary boundaries
- **Device errors**: Descriptive `io::Error` context for missing `/dev/fb0`, `/dev/input/event*`, ALSA devices
- **Core errors**: `CoreError` enum (LoadFailed, InitFailed, GameLoadFailed, RunFailed)
- **Graceful degradation**: Fail open where possible (e.g., no audio device → silent fallback)

## Build System

### Cargo.toml Structure
```toml
# Root workspace
[workspace]
members = [
    "sdlretro-core",
    "sdlretro-frontend",
]
resolver = "2"
```

### Cross-Compilation
- **ARM (Raspberry Pi)**: `--target thumbv7neon-unknown-linux-gnueabihf`
- **x86_64**: Native build on Linux desktop
- `.cargo/config.toml` with arm-linux-gnueabihf-gcc linker and SSH runner

### Vendored Dependencies
- **libretro.h**: Vendored in `doc/libretro.h`, used by bindgen
- **miniz**: Use `miniz_oxide` crate instead of vendored `external/miniz`
- **nlohmann/json**: Replace with `serde_json`
- **fmt**: Use `std::fmt` (Rust standard library formatting is sufficient)

## Migration Strategy

### Phase 1: Foundation ✅ DONE
- Workspace setup, `sdlretro-core` with bindgen, basic core loading/lifecycle
- Verified: core init → load_game → run → unload works

### Phase 2: Video (FBDEV) ✅ DONE
- `sdlretro-core` with fbdev video: mmap, pixel conversion, letterboxing
- Verified: renders frames from snes9x2010 and fceumm cores to /dev/fb0

### Phase 3: Audio + Input ✅ DONE
- EVDEV input polling thread via `evdev` crate — DONE
- ALSA audio output with ring buffer and playback thread — DONE
- Verified: keyboard input works (SNES mapping: arrows, K/J/L/U/S/D, Enter, Shift)

### Dynamic Resolution ✅ DONE
- Handles `RETRO_ENVIRONMENT_SET_GEOMETRY` (key 37) and `SET_SYSTEM_AV_INFO` (key 32)
- Updates letterboxing and throttle in real-time
- Verified: Genesis-Plus-GX (MUSHA) switches 256x192 ↔ 320x224 correctly

### Throttle Timing ✅ DONE
- Drift-correct frame timing using `clock_gettime(CLOCK_MONOTONIC)`
- Frame skip when core is late, tight sleep loop with re-check
- Verified: 60.10 FPS locked for NES/SNES NTSC cores

### Cross-Compilation ✅ DONE
- ARM7 (thumbv7neon-unknown-linux-gnueabihf) target configured
- `.cargo/config.toml` with arm-linux-gnueabihf-gcc linker

### Phase 4: UI System ✅ DONE
- **Font renderer** (`font.rs`): Embedded bitmap fonts (8px/16px tall, ASCII 0x20-0x7E), shadow rendering at (x+1, y-1), text measurement, XRGB8888/RGB565 format conversion
- **Core options** (`core_options.rs`): v1/v2 API support, `retro_core_option_definition` parsing, `retro_core_option_v2_definition` with categories, old-style `retro_variable` parsing, `v2_values` HashMap for runtime values, `get_current_value()` with fallback chain (v2_values → old_values → defaults)
- **GUI overlay** (`gui.rs`): `Gui` struct with `GuiState` enum (Playing/MenuOpen/Settings), `Menu` struct with navigation/scrolling/value cycling, ESC toggle (keycode 1), arrow keys for navigation, left/right arrows or space for value cycling, 15-frame debounce on value changes, fallback overlay when no core options available
- **Runtime option changes**: `VARIABLE_UPDATE_PENDING` flag set when user changes options, returned by `GET_VARIABLE_UPDATE` handler (key 17), reset after core reads it — snes9x2010 detects changes via `check_variables()` and applies new settings

### GUI Design
```
┌─────────────────────────────────────┐
│  snes9x2010                         │  ← Header (core name)
├─────────────────────────────────────┤
│  ► Frame Skip: [1]                  │  ← Selected option (yellow highlight)
│    Block Invalid VRAM: [disabled]   │
│    Synchronous DSP: [false]         │
│    ...                              │
│  < v                                │  ← Scroll indicators
├─────────────────────────────────────┤
│  No ROM loaded | Press ESC to close │  ← Footer
└─────────────────────────────────────┘
```

**Key interactions**:
- ESC: Toggle menu open/close (edge detection via `was_key_just_pressed()`)
- Up/Down arrows: Navigate options (debounced, requires key release)
- Left/Right arrows or Space: Cycle option values (15-frame debounce ≈ 0.25s)
- Enter: Confirm selection (for future action items)

### Phase 5: Full Integration 🔲 TODO
- `sdlretro-frontend` binary tying everything together
- Configuration, i18n, save states, ZIP ROM loading
- Feature parity with existing C++ build

### Phase 6: Hardening 🔲 TODO
- Performance profiling and optimization (SIMD, threading)
- Edge case testing across target devices
- Documentation and packaging (`.opk` for GCW-Zero)

## Testing

### Unit Tests
- **sdlretro-core**: Environment command dispatch, callback registration, variable parsing
- **sdlretro-driver**: Pixel conversion functions (XRGB8888→RGB565), scaling math, keymap lookup
- **sdlretro-gui**: Menu state transitions, text measurement, button hit detection
- **sdlretro-config**: Config serialization/deserialization round-trip

### Integration Tests
- **Core loading**: Load a known `.so` core, call `retro_get_system_info()`, verify library name
- **Frame rendering**: Load a ROM, run 1 frame, verify framebuffer memory contains expected pixels
- **Audio output**: Generate silence, verify no ALSA errors
- **Input polling**: Inject synthetic evdev events, verify InputState updates

### Device-Specific Testing
- Raspberry Pi (ARM): Real hardware smoke test
- x86_64 Linux: Full regression suite with framebuffer (tested: Genesis-Plus-GX/MUSHA dynamic resolution)

### GUI Testing
- ESC toggle: Menu opens/closes with edge detection (verified)
- Navigation: Up/down arrows scroll options (verified)
- Value cycling: Left/right arrows and space cycle option values (verified)
- Runtime changes: snes9x2010 detects and applies option changes via GET_VARIABLE_UPDATE (verified)
- Debounce: 15-frame delay prevents rapid value changes (verified)
- Fallback overlay: Shows when core doesn't support core options (verified)
