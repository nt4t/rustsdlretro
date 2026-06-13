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
| Phase 3: Audio | ⏳ STUBBED | Dummy callbacks set, ALSA output pending |
| Throttle timing | ✅ DONE | clock_gettime(CLOCK_MONOTONIC), drift-correct, frame skip |
| Cross-compile | ✅ DONE | thumbv7neon-unknown-linux-gnueabihf target configured |
| Phase 4: UI System | 🔲 TODO | Menu overlay, core selector, ROM loader |
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
- **Cross-compile**: .cargo/config.toml for armv7 target

### Pending Features
- **Audio**: ALSA PCM output with resampling (libsamplerate)
- **UI**: Menu system, core selector, ROM browser
- **Config**: sdlretro.json loader/writer
- **Save states**: SRAM/RTC persistence
- **ZIP ROM loading**: miniz_oxide extraction
- **Core variables**: Options and preferences
- **i18n**: Language file loading

## Architecture

### Crate Layout

```
sdlretro/
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock
│
├── sdlretro-core/                # Libretro core management (FFI)
│   ├── src/
│   │   ├── lib.rs                # Core struct, ResolutionState, Throttle, FFI bindings
│   │   ├── video.rs              # FbdevVideo: mmap fb0, pixel conversion, letterboxing
│   │   └── input.rs              # InputReader: evdev polling thread, key mapping
│   └── build.rs                  # bindgen invocation, copy libretro.h
│
├── sdlretro-frontend/            # Binary crate: main entry point
│   ├── src/
│   │   └── main.rs               # CLI args, driver init, main loop, callbacks
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
| 31 | `RETRO_ENVIRONMENT_GET_LOG_INTERFACE` | ✅ Implemented - registers log callback |
| 32 | `RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO` | ✅ Implemented - updates resolution + fps |
| 37 | `RETRO_ENVIRONMENT_SET_GEOMETRY` | ✅ Implemented - updates resolution, called from retro_run() |

Future commands to implement:
- `SET_VARIABLES`, `GET_VARIABLE`, `SET_CORE_OPTIONS`
- `SET_HW_RENDER`, `GET_PREFERRED_HW_RENDER`
- `SET_INPUT_DESCRIPTORS`
- `GET_VFS_INTERFACE`
- `SET_MESSAGE`, `SET_MESSAGE_EXT`
- `SET_SERIALIZATION_QUIRKS`

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

Currently stubbed with dummy callbacks that discard all audio samples.

```rust
extern "C" fn audio_sample_cb(_left: i16, _right: i16) { }
extern "C" fn audio_sample_batch_cb(_data: *const i16, _frames: usize) -> usize { 0 }
```

Future: ALSA PCM output with resampling via `alsa` crate and libsamplerate.

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
│    1. core.run()                // retro_run()              │
│       ├── retro_input_poll_cb()  → input.poll()            │
│       ├── retro_video_refresh_cb() → video.push_frame()    │
│       ├── retro_audio_sample_cb()  → audio.push_sample()   │
│       └── retro_input_state_cb()   → input.read_button()   │
│    2. throttle.check_wait()       // Frame timing           │
│       ├── usecs > 0 → sleep loop (re-check)                │
│       └── usecs <= 0 → set_skip_frame() (frame skip)       │
│    3. FPS counter (print every 5s)                          │
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
│  InputState       │    │                          │
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

### Phase 3: Audio + Input ✅ PARTIAL
- EVDEV input polling thread via `evdev` crate — DONE
- ALSA audio output with resampling — STUBBED (dummy callbacks)
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

### Phase 4: UI System 🔲 TODO
- `sdlretro-gui` with Painter trait, bitmap font, menu state machine
- In-game menu overlay, core selector, ROM loader

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
