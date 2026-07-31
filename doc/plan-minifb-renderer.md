# Plan: Minifb Renderer & Config System

## Goal
Add a `minifb` windowed renderer as an alternative to the framebuffer backend, controlled via a config file. This enables development and testing on desktop Linux without requiring `/dev/fb0`.

## Background

### Current State
- `FbdevVideo` is hardcoded in `main.rs` — the frontend directly instantiates it
- `FbdevVideo` implements all rendering methods (`push_frame`, `draw_*_overlay`, etc.)
- The GUI (`Gui::render`) takes `&mut FbdevVideo` directly
- No config system exists; all settings are hardcoded or CLI args

### rust_minifb_test Reference
```rust
let mut buffer: Vec<u32> = vec![0; 640 * 480];
let mut window = Window::new("title", 640, 480, WindowOptions::default())?;
window.set_target_fps(60);
window.update_with_buffer(&buffer, 640, 480);
window.is_open();
window.is_key_down(Key::Escape);
```

Key API:
- `Window::new(name, w, h, opts)` — creates window
- `WindowOptions::scale` — `Scale::X1`, `X2`, etc. (integer scaling)
- `WindowOptions::borderless` — no window decorations
- `update_with_buffer(&[u32], w, h)` — display a 32bpp buffer
- `is_open()` — check if window is still open
- `is_key_down(Key)` — check key state

## Design Decisions

### 1. VideoBackend Trait (Not FbdevVideo Trait)
`FbdevVideo` stays as-is (it's the existing implementation). We introduce a `VideoBackend` trait that both `FbdevVideo` and `MinifbVideo` implement. This avoids breaking changes to existing code.

### 2. Feature Flag
```toml
[features]
default = ["fbdev"]
fbdev = []
minifb = ["minifb_dep"]

[dependencies]
minifb_dep = { package = "minifb", version = "0.28", optional = true, features = ["x11"] }
```

### 3. Config File Format
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

- `renderer`: `"fbdev"` or `"minifb"`
- `window.*`: only used when `renderer = "minifb"`
- Default config path: `~/.config/rustsdlretro/config.json`
- CLI flag `--config <path>` to override

### 4. Input Remapping for Minifb
minifb uses its own `Key` enum (A-Z, Up, Down, Left, Right, Escape, Space, etc.). We need to map minifb keys to the same internal keycodes that the libretro core expects.

Current input mapping (from `input.rs` evdev scan codes):
- ESC = key 1 (evdev `KEY_ESC` = 1)
- Up = key 14 (`KEY_UP`)
- Down = key 17 (`KEY_DOWN`)
- Left = key 12 (`KEY_LEFT`)
- Right = key 15 (`KEY_RIGHT`)
- Enter = key 28 (`KEY_ENTER`)
- Space = key 57 (`KEY_SPACE`)

minifb Key mapping:
- `Key::Escape` → 1
- `Key::Up` → 14
- `Key::Down` → 17
- `Key::Left` → 12
- `Key::Right` → 15
- `Key::Enter` → 28
- `Key::Space` → 57

### 5. MinifbVideo Implementation
- Internal buffer: `Vec<u32>` sized to `window_width * window_height`
- `push_frame`: scales core frame to window using minifb's built-in scale mode
- Overlay rendering: draws on top of the buffer (after `push_frame` but before `update_with_buffer`)
- Letterboxing: same logic as `FbdevVideo`, computed from window dimensions

## Architecture

```
rustsdlretro-core/
├── src/
│   ├── video.rs          # VideoBackend trait + FbdevVideo (unchanged)
│   ├── video_minifb.rs   # MinifbVideo implementation (new)
│   ├── config.rs         # Config loading/parsing (new)
│   ├── input.rs          # InputReader (unchanged)
│   ├── input_minifb.rs   # MinifbInputReader (new, optional)
│   └── ...
├── Cargo.toml             # feature flags
└── build.rs

rustsdlretro-frontend/
├── src/
│   ├── main.rs           # Updated: config loading, backend selection
│   └── ...
└── Cargo.toml
```

## Implementation Phases

### Phase 1: VideoBackend Trait ✅ DONE
1. ✅ Define `VideoBackend` trait in `video.rs` with all methods currently on `FbdevVideo`
2. ✅ Implement the trait for `FbdevVideo` (move methods into `impl VideoBackend for FbdevVideo`)
3. ✅ Update `Gui::render` to take `&mut dyn VideoBackend` instead of `&mut FbdevVideo`
4. ✅ Update all call sites in `main.rs` to use trait objects
5. ✅ `cargo check` to verify nothing breaks

### Phase 2: MinifbVideo Implementation ✅ DONE
1. ✅ Add `minifb` as optional dependency with `minifb` feature flag
2. ✅ Create `video_minifb.rs` with `MinifbVideo` struct
3. ✅ Implement `VideoBackend` for `MinifbVideo`
4. ✅ `MinifbVideo::new(width, height, opts)` — creates window + buffer
5. ✅ `MinifbVideo::push_frame` — copies core frame into buffer with letterboxing
6. ✅ `MinifbVideo::draw_*_overlay` — draw on buffer directly (32bpp only, simpler than fbdev)
7. ✅ `MinifbVideo::update` — call `window.update_with_buffer()` each frame
8. ✅ `MinifbVideo::is_open` — call `window.is_open()`

### Phase 3: Config System ✅ DONE
1. ✅ Create `config.rs` with `Config` struct
2. ✅ Parse JSON config (use `serde` + `serde_json`)
3. ✅ Default config at `~/.config/rustsdlretro/config.json`
4. ✅ CLI `--config <path>` override
5. ✅ `renderer` field determines which backend to instantiate
6. ✅ Window config fields for minifb settings

### Phase 4: Minifb Input (Optional)
1. Create `input_minifb.rs` with `MinifbInputReader`
2. Poll minifb key state each frame
3. Map minifb `Key` → evdev keycode
4. Shared `Arc<Mutex<dyn InputBackend>>` pattern

### Phase 5: Frontend Integration ✅ DONE
1. ✅ Update `main.rs` to load config
2. ✅ Select backend based on config
3. ✅ Handle both backends in the main loop
4. ✅ Graceful shutdown on window close

### Phase 6: Bug Fixes ✅ DONE
1. ✅ Fixed `Throttle::new`: `next_frame` initialized to `now_usec() + frame_time` (was `now_usec()`), preventing first-frame skip
2. ✅ Fixed `Throttle::check_wait`: when late, `next_frame = now + frame_time` (was `next_frame += frame_time`), preventing perpetual frame skip
3. ✅ Fixed minifb pixel format: all color writes use ARGB8888 (`0xAARRGGBB`) instead of incorrect `0x00BBGGRR`

## MinifbVideo Method Details

```rust
pub struct MinifbVideo {
    window: minifb::Window,
    buffer: Vec<u32>,
    width: u32,
    height: u32,
    core_width: u32,
    core_height: u32,
    offset_x: i32,
    offset_y: i32,
    skip_frame: bool,
    frame_drawn: bool,
}

impl MinifbVideo {
    pub fn new(window_width: u32, window_height: u32, opts: WindowOptions) -> Self {
        let mut window = minifb::Window::new("rustsdlretro", window_width as usize, window_height as usize, opts)
            .expect("Failed to create window");
        window.set_target_fps(60);
        
        let buffer = vec![0u32; (window_width * window_height) as usize];
        
        Self {
            window,
            buffer,
            width: window_width,
            height: window_height,
            core_width: 0,
            core_height: 0,
            offset_x: 0,
            offset_y: 0,
            skip_frame: false,
            frame_drawn: false,
        }
    }

    pub fn update(&mut self) {
        let _ = self.window.update_with_buffer(&self.buffer, self.width as usize, self.height as usize);
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn push_frame(&mut self, pixels: *const c_void, frame_w: u32, frame_h: u32, pitch: usize) {
        // Skip frame if throttle requested it
        if self.skip_frame { self.skip_frame = false; return; }
        // Handle core XRGB8888 (32bpp) and RGB565 (16bpp) → minifb ARGB8888
        // Color conversion: 0xAARRGGBB = 0xFF000000 | (r << 16) | (g << 8) | b
        // Letterboxing: centered with offset_x / offset_y
    }
}
```

## Key Considerations

1. **Letterboxing**: Both backends need the same letterboxing math. Extract to a shared function.
2. **Pixel format**: minifb expects ARGB8888 (`0xAARRGGBB` as a `u32`). On little-endian the in-memory byte order is B-G-R-A, so `0xAARRGGBB` maps to bytes `BB GG RR AA`. Core XRGB8888 frames are converted via `0xFF000000 | (r << 16) | (g << 8) | b`. Core RGB565 frames are expanded to 8-bit per channel then written the same way.
3. **Overlay rendering on minifb**: Since the buffer is always 32bpp, overlay drawing is simpler — no RGB565 conversion needed. All overlay colors must also use `0xFF000000 | (r << 16) | (g << 8) | b` format.
4. **Throttle timing**: The `Throttle` struct in `lib.rs` controls frame pacing. `next_frame` is initialized to `now_usec() + frame_time` (not `now_usec()`) so the first frame is never skipped. When the emulator runs faster than target FPS and `check_wait()` returns negative, `next_frame` is reset to `now + frame_time` (not `next_frame += frame_time`) to prevent `next_frame` from drifting indefinitely ahead, which would cause every subsequent frame to be skipped.
4. **Scaling**: Use `WindowOptions::scale = Scale::X2` (or X3/X4) so the core renders at native resolution and minifb scales up. This gives crisp pixel art on high-res displays.
5. **Borderless mode**: When `borderless: true`, the window fills the screen area — useful for kiosk mode on embedded devices.
6. **Config file permissions**: Use `0o600` when writing config for security.
7. **serde dependency**: Add `serde` and `serde_json` to `rustsdlretro-core` dependencies (both optional behind `config` feature).

## Testing Plan

1. Build with `--features minifb` on desktop Linux
2. Verify window opens at configured resolution
3. Verify game renders correctly with letterboxing
4. Verify GUI menu renders and is interactive
5. Verify key mapping works (ESC, arrows, space)
6. Verify scaling modes (X1, X2, X3)
7. Build with default features (fbdev only) — verify no minifb dependency is pulled in
