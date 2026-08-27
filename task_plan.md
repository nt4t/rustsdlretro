# Task Plan: Web API for Emulator Control

## Goal
Implement a WebSocket-based control API that allows external clients to:
- **Save/Load State** — Save/load state snapshots via WebSocket
- **Send Input Keys** — Map gamepad/keyboard input over the wire
- **Frame-by-frame execution** — Step through emulation one frame at a time
- **PNG frame streaming** — Send raw PNG frames over WebSocket for external clients

## Design Decisions (to be confirmed)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Transport | WebSocket only (binary + text) | Input, frames, control messages |
| WS crate | `tungstenite` (optional) | Supports binary frames natively |
| PNG encoding | `png` crate (optional) | Lightweight, no heavy image dependency |
| Architecture | Dedicated control thread with shared state | Decouples web server from main emulation loop |
| Input model | JSON commands → translate to existing InputReader state | Reuse existing input pipeline, don't duplicate mapping |
| State management | Atomic flags + mutex-protected Core handle | Minimal locking, safe concurrent access |

---

## Phase 0: Research & Dependencies
**Status:** COMPLETE ✅
- [x] Evaluate WebSocket crates — selected `tungstenite` (supports binary frames)
- [x] No HTTP server needed
- [x] Assess PNG encoding options — selected `png` crate
- [x] API is optional via Cargo feature flag `api`

## Phase 1: Shared State Architecture (`api.rs` — new module)
**Status:** COMPLETE ✅
- [x] Create `rustsdlretro-core/src/api.rs` (behind `#[cfg(feature = "api")]`)
- [x] Define `ApiState` struct with thread-safe state:
  - `running`, `paused`, `step_frame` (frame-by-frame flag)
  - `inputs: AllInputs` — current key states from web client
  - `save_requested`, `load_requested` — one-shot actions
- [x] Implement thread-safe getters/setters for all state via `Mutex<ApiState>`
- [x] Add `consume_frame_step()`, `take_save_request()`, `take_load_request()` etc.
- [x] Add `create_api_state()` factory that spawns the WebSocket server thread
- [x] Updated `Cargo.toml` with optional deps: tungstenite, png, tokio + api feature

## Phase 2: WebSocket Server (`api_server.rs` — new module, inline in api.rs)
**Status:** COMPLETE ✅ (with async-tungstenite 0.35 fix)
- [x] Implemented server in `rustsdlretro-core/src/api_server` submodule
- [x] Message protocol: ClientMessage enum (Input, Step, Play, Pause, SaveState, LoadState, SetOption)
- [x] Response messages: ServerMessage enum (Status, FrameDone, Flash, Error)
- [x] Server listens on configurable port `18932`
- [x] Handles all client→server message types with proper state mutations
- [x] Uses tokio runtime + tungstenite for async WebSocket handling
- [x] Implemented message protocol (JSON) — tested and verified working
  ```json
  // Input command (send continuously or on change)
  {"type": "input", "port": 0, "buttons": {"up": true, "down": false, ...}}

  // Frame step request
  {"type": "step"}

  // Save/Load state
  {"type": "save_state"}
  {"type": "load_state"}

```
- [x] All server response messages implemented and tested:
  ```json
  {"type": "status", "running": true, "fps": 59.94, "width": 320, "height": 240}
  {"type": "frame_done"}                              // frame step ack
  {"type": "flash", "message": "State Saved"}
  {"type": "error", "message": "..."}
  ```
- [ ] Start server on configurable port (default: `18932` — retro!)

## Phase 3: Frontend Integration (`main.rs`)
**Status:** COMPLETE ✅
- [x] Parse `--api-port N` CLI flag (only when compiled with `api` feature)
- [x] Gate API initialization behind `#[cfg(feature = "api")]` in main
- [x] Create SharedApiState after resolution is known (passes Arc<Mutex<ResolutionState>>)
- [x] In main loop, check ApiState flags before core.run():
  - If step_frame → consume and run one frame
  - If paused/not-running → skip throttle wait but still render GUI
- [x] Map web client input → existing InputReader state via `set_button()` method
- [x] Connect save/load requests from API to existing F2/F4 logic (save_state/load_state)
- [x] Relay option changes through pending queue to CoreOptions
- [x] Update resolution/FPS in ApiState each frame for status messages
  - If `step_frame`, set it and run one frame then return to idle
  - If not running (paused), skip frame throttle wait
- [x] Map web client input → existing InputReader state on each poll cycle (done in main loop)
- [x] Connect save/load requests from API to existing F2/F4 logic (done in main loop)
- [x] Relay flash messages and status updates back through WebSocket (done in api_server)

## Phase 4: DONE — No HTTP needed (ROM upload out of scope)
**Status:** COMPLETE ✅
- [x] ROM upload removed from requirements

## Phase 5: PNG Frame Streaming (`api_server.rs` extension)
**Status:** COMPLETE ✅
- [x] All PNG code behind `#[cfg(feature = "api")]`
- [x] Capture framebuffer buffer after each frame (via push_captured_frame in VideoBackend)
- [x] Encode RGBA → PNG using png crate
- [x] Send as binary WebSocket message: `[width u16][height u16][PNG bytes]`
- [x] Frame rate limiting (~30 fps max via elapsed time check before encoding/sending)

### Implementation Details
- **Frame capture**: Video backends copy core pixels → RGBA format in push_frame()
- **Channel**: crossbeam_channel bounded channel (size 4) for non-blocking frame transfer
- **Distribution**: Module-level CLIENTS static + per-client tokio channels for async delivery
- **Rate limiting**: ~30fps max via elapsed time check before encoding/sending
- **Binary format**: `[width u16 BE][height u16 BE][raw PNG bytes]`

### Testing
- `tests/test_png_stream.js` — Full integration test (connect → Play → Step → capture PNG frames)
- Validates PNG signature, IHDR chunk, IDAT data, saves to `./test_output/`

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `rustsdlretro-core/src/api.rs` | **Create** | ✅ Done — Shared state struct, thread-safe API, protocol types |
| `rustsdlretro-core/src/api_server` | **Inline in api.rs** | ✅ Done — WebSocket server + message handlers |
| `rustsdlretro-core/Cargo.toml` | Modify | ✅ Done — Added optional `tungstenite`, `png`, `tokio` deps + `api` feature |
| `rustsdlretro-frontend/src/main.rs` | Modify | ✅ Done — CLI flag, API state creation, main loop integration |
| `rustsdlretro-core/Cargo.toml` | Modify | ✅ Done — Added `api` feature to workspace member |

| `rustsdlretro-frontend/Cargo.toml` | Modify | ✅ Done — added `api = ["rustsdlretro-core/api"]` feature passthrough |

---

## Dependencies Added
```toml
tungstenite = { version = "0.30", features = ["rustls-tls-webpki-roots"], optional = true }
png = { version = "0.17", optional = true }
tokio = { version = "1", features = ["full"], optional = true }

[features]
api = ["dep:tungstenite", "dep:png", "dep:tokio", "dep:serde", "dep:serde_json"]
```

## Dependencies to Add
```toml
# rustsdlretro-core/Cargo.toml
[dependencies]
tungstenite = { version = "0.23", features = ["native-tls"], optional = true }
png = { version = "0.17", optional = true }

[features]
default = []
api = ["dep:tungstenite", "dep:png"]
```

---

## WebSocket Protocol Details

### Client → Server Messages

| Type | Payload | Description |
|------|---------|-------------|
| `input` | `{port, buttons: {up, down, left, right, a, b, x, y, l, r, start, select}}` | Joypad state snapshot |
| `step` | `{}` | Run exactly one frame (pauses after) |
| `play` | `{}` | Resume continuous playback |
| `pause` | `{}` | Pause emulation |
| `save_state` | `{}` | Trigger save state (same as F2) |
| `load_state` | `{}` | Trigger load state (same as F4) |
| `set_option` | `{key, value}` | Set core option dynamically |

### Server → Client Messages

| Type | Payload | Description |
|------|---------|-------------|
| `status` | `{running: bool, fps: float, width: u32, height: u32}` | Status update (sent on change or on request) |
| `frame_done` | `{}` | Acknowledge single frame executed (for step mode) |
| `flash` | `{message, duration_ms}` | Flash message relay |
| `error` | `{message}` | Error notification |

### Binary Frame Messages (PNG Streaming)

Server sends binary frames to connected clients:
```
[2-byte header: width (u16 BE), height (u16 BE)] [raw PNG bytes]
```

Clients build their own renderers — server just provides raw PNG data.

---

## Errors to Anticipate
- **Thread safety**: Core.run() is not reentrant; must lock around all Core access
- **Audio drift**: Frame stepping may cause audio buffer underruns — handle gracefully
- **PNG encoding CPU cost**: Encoding every frame at 60fps will be expensive — throttle to ~15-30fps or only encode on user request
