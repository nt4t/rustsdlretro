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
- **Frame timing**: Throttle class with clock_gettime(CLOCK_MONOTONIC), drift-correct
- **FPS output**: Console print every 5 seconds with frame count and actual FPS
- **Continuous loop**: Main loop runs indefinitely, Ctrl+C exit via SIGINT handler
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
│   │   ├── lib.rs                # Workspace re-exports
│   │   ├── core.rs               # Core struct: init, load_game, run, unload
│   │   ├── bindings.rs           # bindgen-generated from libretro.h
│   │   ├── environment.rs        # Environment command handlers (50+ commands)
│   │   ├── callbacks.rs          # Video/audio/input callback registration
│   │   ├── variables.rs          # Core variables/options
│   │   └── vfs.rs                # VFS interface (RETRO_ENVIRONMENT_GET_VFS_INTERFACE)
│   └── build.rs                  # bindgen invocation, copy libretro.h
│
├── sdlretro-driver/              # Driver abstraction + fbdev impl
│   ├── src/
│   │   ├── lib.rs
│   │   ├── driver_base.rs        # Trait: VideoDriver, AudioDriver, InputDriver
│   │   └── fbdev/
│   │       ├── mod.rs            # FbdevDriver: creates video/audio/input
│   │       ├── video.rs          # FbdevVideo: mmap fb0, pixel conversion, scaling
│   │       ├── audio.rs          # FbdevAudio: ALSA PCM output
│   │       └── input.rs          # FbdevInput: evdev polling thread
│   └── Cargo.toml
│
├── sdlretro-gui/                 # Menu/UI system
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ui_host.rs            # Top-level UI controller, state machine
│   │   ├── menu.rs               # Menu navigation, button interaction
│   │   ├── elements.rs           # UI element traits: text, rect, button, list
│   │   ├── painter.rs            # Drawing primitives to framebuffer
│   │   └── font.rs               # Bitmap font rendering (bmfont.inl data)
│   └── Cargo.toml
│
├── sdlretro-frontend/            # Binary crate: main entry point
│   ├── src/
│   │   ├── main.rs               # CLI args, driver init, main loop
│   │   └── rom_loader.rs         # ZIP ROM extraction (miniz binding)
│   └── Cargo.toml
│
├── sdlretro-config/              # Configuration management
│   ├── src/
│   │   ├── lib.rs
│   │   ├── loader.rs             # Read/write sdlretro.json
│   │   └── schema.rs             # Config struct definitions
│   └── Cargo.toml
│
└── sdlretro-i18n/                # Internationalization
    ├── src/
    │   ├── lib.rs
    │   └── loader.rs             # Language file loading (en-US.json, zh-CN.json)
    └── Cargo.toml
```

### Workspace Dependencies

```toml
# Workspace Cargo.toml (shared across crates)
[workspace.dependencies]
# FFI / OS
libc = "0.2"
nix = { version = "0.29", features = ["mmap", "fs", "term", "input"] }

# Error handling
thiserror = "2"
anyhow = "1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Audio
alsa = "0.9"

# ZIP support
miniz_oxide = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Build-time
bindgen = "0.71"
```

### Core Libretro Integration

#### FFI Bindings
- **Source**: Vendored `src/libretro/include/libretro.h` (same header used by C++ build)
- **Generation**: `build.rs` runs `bindgen` to produce `bindings.rs`
- **Safety**: Bindings wrapped in safe Rust structs; all `unsafe` blocks are localized to the `sdlretro-core` crate

#### Core Lifecycle
```rust
pub struct Core {
    handle: Library,                    // dlopen handle
    api: retro_api,                     // All function pointers
    env_cb: Option<EnvironmentHandler>, // Set via retro_set_environment
    video_cb: Option<VideoRefresh>,
    audio_cb: Option<AudioSample>,
    audio_batch_cb: Option<AudioSampleBatch>,
    input_poll_cb: Option<InputPoll>,
    input_state_cb: Option<InputState>,
    // ... other callbacks
}

impl Core {
    pub fn new(path: &Path) -> Result<Self, CoreError>;
    pub fn init(&mut self) -> Result<(), CoreError>;
    pub fn load_game(&mut self, game: &RomData) -> Result<(), CoreError>;
    pub fn run(&mut self) -> Result<(), CoreError>;
    pub fn unload_game(&mut self);
    pub fn unload(&mut self);
}
```

#### Callback Registration
The C++ `driver_base.cpp` sets up callbacks via `retro_set_environment`, `retro_set_video_refresh`, etc. The Rust equivalent:

```rust
impl Core {
    pub fn set_environment(&mut self, handler: EnvironmentHandler) {
        unsafe {
            (self.api.set_environment)(Some(environment_callback_trampoline));
            self.env_cb = Some(handler);
        }
    }
    // Similar for video, audio, input callbacks
}
```

Each trampoline extracts the Rust handler from thread-local or static storage and invokes it.

#### Environment Commands
All 50+ environment commands from the C++ implementation are supported:
- **Display**: `SET_PIXEL_FORMAT`, `SET_SYSTEM_AV_INFO`, `SET_GEOMETRY`, `SET_SUPPORT_NO_GAME`
- **Variables**: `SET_VARIABLES`, `GET_VARIABLE`, `GET_VARIABLE_UPDATE`, `SET_CORE_OPTIONS`, `SET_CORE_OPTIONS_DISPLAY`
- **Rendering**: `SET_HW_RENDER`, `GET_PREFERRED_HW_RENDER`
- **Input**: `SET_INPUT_DESCRIPTORS`, `GET_INPUT_DEVICE_CAPABILITIES`, `GET_INPUT_BITMASKS`
- **Filesystem**: `GET_SYSTEM_DIRECTORY`, `GET_SAVE_DIRECTORY`, `GET_VFS_INTERFACE`
- **Features**: `SET_KEYBOARD_CALLBACK`, `GET_RUMBLE_INTERFACE`, `SET_MESSAGE`, `SET_MESSAGE_EXT`
- **Save States**: `SET_SERIALIZATION_QUIRKS`, `GET_SAVE_DIRECTORY`
- **Info**: `GET_LOG_INTERFACE`, `GET_PERFORMANCE_INTERFACE`

### Graphics Driver (FBDEV)

#### Framebuffer Access
```rust
pub struct FbdevVideo {
    fd: RawFd,                        // /dev/fb0 file descriptor
    map: *mut u8,                     // mmap'd framebuffer memory
    len: usize,                       // mmap length
    info: fb_var_screeninfo,          // ioctl query result
    mode: VideoMode,                  // Resolution, bpp, pixel format
    h_line: Vec<u16>,                 // Scaling line buffer (RGB565)
    dirty: Rect,                      // Dirty rectangle for partial updates
}
```

- **mmap**: `nix::sys::mmap::mmap` with `Protection::READWRITE | Protection::READ`, `Flags::SHARED`
- **ioctl**: `nix::sys::ioctl` for `FBIOGET_VSCREENINFO`, `FBIOPUT_VSCREENINFO`
- **Pixel conversion**: XRGB8888 → RGB565 with SIMD acceleration (SSE2 on x86, NEON on ARM)
- **Scaling**: h_line buffer algorithm identical to C++ `fbdev_video.cpp`

#### Video Refresh Callback
```rust
fn video_refresh(data: *const c_void, width: c_uint, height: c_uint, pitch: c_uint) {
    // Called by libretro core during retro_run()
    let driver = get_video_driver();
    driver.push_frame(data, width, height, pitch);
}
```

### Audio Driver (ALSA)

```rust
pub struct FbdevAudio {
    pcm: AlsaPcmHandle,               // ALSA PCM device handle
    sample_rate: u32,
    channels: u16,
    buffer: Vec<f32>,                 // Resampled audio buffer
}
```

- **Output**: ALSA `pcm::Pcm::new` with `pcm::PcmType::PCM`, `pcm::AccessType::Interleaved`
- **Resampling**: libsamplerate via FFI (reuse existing `libsamplerate` vendored `.c` files, bind with `cbindgen`)
- **Mono mixdown**: Stereo-to-mono conversion in the audio callback
- **Buffer management**: Ring buffer with ALSA `pcm::avail_wait` for precise timing

### Input Driver (EVDEV)

```rust
pub struct FbdevInput {
    state: Arc<Mutex<InputState>>,    // Shared mutable input state
    reader: JoinHandle<()>,           // Background evdev polling thread
}

pub struct InputState {
    pub buttons: [u32; RETRO_DEVICE_ID_JOYPAD_MAX],  // Last frame button states
    pub axis: [i16; RETRO_DEVICE_ANALOG_MAX],        // Analog axis positions
}
```

- **Device enumeration**: Scan `/dev/input/event*` for devices with `EV_KEY` and `EV_ABS`
- **Polling thread**: `evdev::Device::open()` → `device.grab()` → blocking read loop with `select`/`poll`
- **State sharing**: `Arc<Mutex<InputState>>` shared between polling thread and main loop
- **Keymap**: Linux `EV_KEY` codes → `RETRO_DEVICE_ID_JOYPAD_*` mapping (matching C++ `sdl1_input.cpp` / `fbdev_input.cpp`)
- **Analog axes**: `EV_ABS` range normalization to `[-32768, 32767]`

### UI System

#### Painter Trait
```rust
pub trait Painter {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color);
    fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color);
    fn draw_button(&mut self, x: i32, y: i32, w: i32, h: i32, label: &str, selected: bool);
    fn flush(&mut self);                    // Apply pending draws to framebuffer
    fn dimensions(&self) -> (u32, u32);    // Current framebuffer resolution
}
```

#### State Machine
```rust
pub enum UiState {
    GameRunning,                            // Core running, no menu
    InGameMenu(SubMenu),                    // Menu overlay on top of game frame
    CoreSelector(Vec<CoreEntry>),           // Multiple cores matched, user selects
    GameLoader(rom_path: Option<PathBuf>),  // ROM selection / ZIP browser
    Settings(SettingsPage),                 // Configuration screen
}
```

#### Font Rendering
- Reuse `bmfont.inl` bitmap font data (embedded as a const byte array)
- Character atlas: precomputed glyph metrics, blit to framebuffer per character
- No FreeType / stb_truetype dependency (keeping footprint minimal for embedded)

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
    "sdlretro-driver",
    "sdlretro-gui",
    "sdlretro-frontend",
    "sdlretro-config",
    "sdlretro-i18n",
]
resolver = "2"
```

### Cross-Compilation
- **ARM (Raspberry Pi)**: `--target armv7-unknown-linux-gnueabihf`
- **x86_64**: Native build on Linux desktop
- Toolchain files in `cargo/` directory
- Conditional compilation: `#[cfg(target_arch = "arm")]`, `#[cfg(target_arch = "x86_64")]`

### Vendored Dependencies
Reuse existing `external/` libraries where possible:
- **libsamplerate**: Compile `.c` files via `cc` crate, expose via FFI
- **miniz**: Use `miniz_oxide` crate instead of vendored `external/miniz`
- **fmt**: Use `std::fmt` (Rust standard library formatting is sufficient)
- **nlohmann/json**: Replace with `serde_json`
- **xxhash**: Use `xxhash-rust` crate
- **cpuid**: Inline x86 CPUID detection (30 lines of inline assembly)

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
- x86_64 Linux VM: Full regression suite with QEMU framebuffer emulation
