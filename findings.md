# Findings: Web API Implementation Research

## Codebase Analysis

### Architecture Summary
- **Two-crate workspace**: `rustsdlretro-core` (library) + `rustsdlretro-frontend` (binary)
- **Single-threaded main loop**: All emulation runs in one thread with async callbacks
- **Global state via unsafe statics**: `MAIN_VIDEO`, `MAIN_AUDIO`, `MAIN_INPUT` are raw pointers to boxed values
- **Core lifecycle**: load → init → run loop → unload

### Key Existing Patterns to Reuse

#### 1. Input System (`input.rs`)
```rust
pub struct InputReader {
    state: Arc<Mutex<[i32; TOTAL_SLOTS]>>,   // Per-port button states (4 ports × 16 buttons)
    just_pressed: Arc<Mutex<[bool; TOTAL_SLOTS]>>, // Edge detection flags
}
```
- **TOTAL_SLOTS = 64** (MAX_PORTS=4 × 16 buttons)
- Buttons mapped via `player1_keycodes_to_joypad()` and `player2_keycodes_to_joypad()`
- Joypad IDs: up(4), down(5), left(6), right(7), b(0), a(8), y(1), x(9), l(10), r(11), start(3), select(2)

#### 2. Core Methods (`lib.rs`)
```rust
impl Core {
    pub fn run(&mut self) -> Result<(), CoreError>;   // One frame
    pub fn save_state(&self) -> Result<Vec<u8>, CoreError>;
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), CoreError>;
    pub fn load_game(&mut self, path: &Path) -> Result<(), CoreError>;
    pub fn unload_game(&mut self);
}
```

#### 3. Throttle System (`lib.rs`)
```rust
pub struct Throttle {
    frame_time: u64,      // microseconds per frame (e.g., 16667 for 60fps)
    next_frame: u64,       // target timestamp for next frame
}
// check_wait() returns µs to sleep, or ≤0 if frame was late
```

#### 4. Save/Load State (`gui.rs` + `main.rs`)
- F2 = save state (calls `core.save_state()` → writes to disk)
- F4 = load state (reads from disk → calls `core.load_state()`)
- Paths: `~/.config/rustsdlretro/saves/{core_name}/{game_name}.{state}`

### Thread Safety Observations
- **MAIN_INPUT**: accessed via raw pointer, protected by caller discipline
- **InputReader internal state**: uses Arc<Mutex<>> internally — thread-safe!
- **Core methods**: NOT thread-safe; `run()` must be called from single thread
- **Gui**: not thread-safe; only modified in main loop

## WebSocket Crate Evaluation

### Option 1: `tungstenite` (recommended)
- Pros: Lightweight, async-ready, native binary frame support, used by many projects
- Cons: Requires manual message handling loop
- License: MIT/Apache-2.0
- Size: ~50KB crate
- **Binary frames**: Built-in — `Message::Binary(Vec<u8>)`

### Option 2: `ws` (websockets crate)
- Pros: Higher-level API with connection handlers
- Cons: Older, less maintained, heavier dependencies
- License: MIT

### Option 3: `actix-web` + `actix-ws`
- Pros: Full HTTP+WS server, mature ecosystem
- Cons: Heavy dependency (10MB+), overkill for single endpoint
- Not recommended unless we need full web UI hosting

**Recommendation**: Use `tungstenite` — supports binary frames natively for PNG streaming.

## Input Mapping Strategy

### Approach: Direct State Injection
Instead of translating JSON to keycodes, inject directly into the input state array:

```rust
// Web client sends joypad button states
// We write directly to InputReader's internal state
let mut s = input.state.lock().unwrap();
s[JOYPAD_UP as usize] = if msg.buttons.up { 1 } else { 0 };
// ... etc for all buttons
```

This avoids duplicating the keycode→joypad mapping logic.

## Frame-by-Frame Implementation Strategy

### Current Main Loop (simplified):
```rust
while RUNNING.load() {
    // Poll input
    // Check GUI save/load keys
    if !menu_open && core.run().is_err() { break; }
    
    // Throttle: sleep until next frame time
    let usecs = throttle.check_wait();
    if usecs > 0 { sleep(usecs); } else { skip_frame(); }
    
    // Render GUI overlay
    video.render(...);
}
```

### Frame Step Modification:
```rust
let api_state = ApiState::get();

// Check for frame step request
if api_state.is_step_requested() {
    if !menu_open && core.run().is_ok() {
        // Send frame_done to WebSocket clients
    }
    api_state.clear_frame_step();
    continue; // Skip throttle, go back to waiting
}

// Normal run mode — apply throttle as usual
```

### Pause Mode:
- Set `pause_requested` flag
- In main loop, if paused: skip `core.run()` entirely, just render GUI overlay
- Audio buffer will gradually drain (acceptable behavior)

## ROM Loading Considerations

### Current ZIP Support (`zip_rom.rs`)
- Detects `.zip` files and extracts ROM to memory or temp file
- Handles both `need_fullpath=true` and `false` cores

### Web Upload Strategy:
1. **HTTP endpoint** `POST /rom` receives multipart form data
2. Store uploaded bytes in `ApiState::pending_rom` (Vec<u8>)
3. Main loop checks for pending ROM → calls `core.unload_game()` then `core.load_game_from_memory()`
4. Need to handle core unload/reinit properly

### Memory Concerns:
- Large ROMs (PSX games ~600MB) should be streamed or saved to disk first
- For WebSocket upload, base64 encoding adds 33% overhead — not suitable for large files
- Recommend HTTP multipart for uploads > 1MB

## PNG Encoding for Frame Streaming

### Crate: `png` (v0.17)
- Pros: Lightweight (~20KB), pure Rust, no FFI dependencies, standard crate
- Cons: Requires manual encoder setup per frame
- License: MIT/Apache-2.0

### Usage Pattern:
```rust
use png::Encoder;
use std::io::Cursor;

fn encode_frame_rgba8(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut encoder = Encoder::new(Cursor::new(Vec::new()), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data).unwrap();
    writer.into_inner().into_inner()
}
```

### Framebuffer Access:
The video backend (`VideoBackend` trait) holds the frame buffer.
After `core.run()`, the buffer contains the latest rendered frame.
We need to copy it before PNG encoding (to avoid holding lock during encode).

```rust
// In main loop, after core.run():
let frame_data = unsafe {
    // MAIN_VIDEO.borrow().get_framebuffer_copy()
};
if api_state.has_ws_clients_streaming() {
    let png_bytes = encode_frame_rgba8(&frame_data, width, height);
    // Send via WebSocket: [2B header] + PNG bytes
}
```

### Performance Considerations:
- **PNG encoding at 60fps**: ~5-10ms per frame on ARM (RPi) — may cause CPU overload
- **Mitigation**: Throttle streaming to 15-30fps, or only encode when client requests it
- **Bandwidth**: 320×240 PNG ≈ 8-15KB; at 15fps = ~120-225 KB/s — fine for WiFi
- Server sends native resolution; clients scale as they wish

### Alternative: JPEG (if needed later)
- `jpeg` crate or `image` crate with JPEG encoding
- Smaller file size (~3-8KB at 70% quality) but higher CPU cost
- PNG preferred for simplicity and lossless quality

---

## Audio Considerations During Frame Step
- ALSA ring buffer will continue draining during frame steps
- If stepping too fast, audio underflow → silence (current behavior)
- If pausing, audio stops naturally when buffer empties
- **No special handling needed** — current architecture handles this gracefully
