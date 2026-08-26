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
**Status:** NOT STARTED
- [ ] Create `rustsdlretro-core/src/api.rs` (behind `#[cfg(feature = "api")]`)
- [ ] Define `ApiState` struct with atomic flags:
  - `run_requested`, `pause_requested`, `step_frame` (frame-by-frame flag)
  - `input_snapshot` — current key states from web client
  - `save_requested`, `load_requested` — one-shot actions

- [ ] Implement thread-safe getters/setters for all state
- [ ] Add `ApiState::consume_frame_step()` — returns true and clears flag

## Phase 2: WebSocket Server (`api_server.rs` — new module)
**Status:** NOT STARTED
- [ ] Create `rustsdlretro-core/src/api_server.rs` (behind `#[cfg(feature = "api")]`)
- [ ] Implement message protocol (JSON):
  ```json
  // Input command (send continuously or on change)
  {"type": "input", "port": 0, "buttons": {"up": true, "down": false, ...}}

  // Frame step request
  {"type": "step"}

  // Save/Load state
  {"type": "save_state"}
  {"type": "load_state"}

```
- [ ] Implement server response messages:
  ```json
  {"type": "status", "running": true, "fps": 59.94, "width": 320, "height": 240}
  {"type": "frame_done"}                              // frame step ack
  {"type": "flash", "message": "State Saved"}
  {"type": "error", "message": "..."}
  ```
- [ ] Start server on configurable port (default: `18932` — retro!)

## Phase 3: Frontend Integration (`main.rs`)
**Status:** NOT STARTED
- [ ] Parse `--api-port N` CLI flag (only when compiled with `api` feature)
- [ ] Gate API initialization behind `#[cfg(feature = "api")]` in main
- [ ] In main loop, check `ApiState` flags BEFORE `core.run()`:
  - If `step_frame`, set it and run one frame then return to idle
  - If not running (paused), skip frame throttle wait
- [ ] Map web client input → existing InputReader state on each poll cycle
- [ ] Connect save/load requests from API to existing F2/F4 logic
- [ ] Relay flash messages and status updates back through WebSocket

## Phase 4: DONE — No HTTP needed (ROM upload out of scope)
**Status:** COMPLETE ✅
- [x] ROM upload removed from requirements

## Phase 5: PNG Frame Streaming (`api_server.rs` extension)
**Status:** NOT STARTED
- [ ] All PNG code behind `#[cfg(feature = "api")]`
- [ ] Capture framebuffer buffer after each frame (from VideoBackend)
- [ ] Encode RGBA → PNG (native core resolution, e.g. 320×240)
- [ ] Send as binary WebSocket message: `[width u16][height u16][PNG bytes]`
- [ ] Frame rate limiting (throttle to ~15-30 fps over WS)

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `rustsdlretro-core/src/api.rs` | **Create** | Shared state struct, atomic flags, thread-safe API |
| `rustsdlretro-core/src/api_server.rs` | **Create** | WebSocket server + PNG frame streaming |
| `rustsdlretro-core/Cargo.toml` | Modify | Add optional `tungstenite`, `png` deps + `api` feature |
| `rustsdlretro-frontend/src/main.rs` | Modify | Conditional compilation: `#[cfg(feature = "api")]` for API init

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
