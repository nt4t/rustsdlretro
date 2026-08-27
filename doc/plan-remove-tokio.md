# Plan: Remove Tokio Dependency from API Module

## Goal

Remove `tokio`, `async-tungstenite`, and `futures-util` dependencies. Replace async WebSocket server with a simple blocking thread-per-client model using only `tungstenite` (handshake feature) + std library.

## Why

- Tokio adds significant build time and binary size
- The API is simple enough to not need an async runtime
- Removes complexity from the codebase (async/await, select!, channel issues)
- Makes debugging easier (synchronous code path)
- Eliminates the tokio mpsc channel delivery bug we encountered

## Current State

```
Dependencies: tokio, tungstenite, async-tungstenite, futures-util, crossbeam-channel
Architecture: Tokio single/multi-threaded runtime → TcpListener.accept() → spawn client tasks
Frame delivery: crossbeam_channel (distributor) → tokio mpsc (per-client) → PNG encoder
```

## Target State

```
Dependencies: tungstenite (handshake only), crossbeam-channel
Architecture: std::net::TcpListener → accept() → thread-per-client
Frame delivery: crossbeam_channel (distributor) → crossbeam_channel (per-client) → PNG encoder
```

## Changes Required

### 1. `rustsdlretro-core/Cargo.toml`

**Remove:**
- `tokio = { version = "1", features = ["full"], optional = true }`
- `async-tungstenite = { version = "0.35", ... }`
- `futures-util = { version = "0.3", optional = true }`

**Modify:**
```toml
# Before:
tungstenite = { version = "0.30", default-features = false, features = ["handshake"], optional = true }

# After:
tungstenite = { version = "0.30", default-features = false, features = ["handshake"], optional = true }
```

**Keep:**
- `crossbeam-channel` (already used for frame distribution)
- `png`, `serde`, `serde_json`

### 2. Feature Flag Changes

**Before:**
```toml
api = ["tungstenite", "tokio", "async-tungstenite", "futures-util", "png", ...]
```

**After:**
```toml
api = ["tungstenite", "png", "dep:serde", "dep:serde_json", "dep:crossbeam-channel"]
```

Remove `tokio`, `async-tungstenite`, `futures-util` from the api feature.

### 3. `rustsdlretro-core/src/api.rs` — Major Rewrite

#### A. Remove `create_api_state()` tokio runtime spawn

**Before:**
```rust
let rt = tokio::runtime::Builder::new_multi_thread()...
std::thread::spawn(move || {
    rt.block_on(api_server::run(addr, state))
});
```

**After:**
```rust
use std::net::{TcpListener, TcpStream};
// Spawn simple blocking server thread
std::thread::spawn(|| {
    if let Err(e) = api_server::run(addr, Arc::clone(&inner)) {
        eprintln!("[API] Server error: {}", e);
    }
});
```

#### B. Replace `api_server` module completely

**Remove:** All async functions (`async fn run`, `async fn handle_client`, etc.)

**Add blocking implementation:**
```rust
pub mod api_server {
    use super::*;
    
    pub fn run(
        addr: SocketAddr,
        state: Arc<Mutex<ApiState>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(addr)?;
        eprintln!("[API] WebSocket server listening on ws://{}", addr);
        
        // Start frame distributor thread
        #[cfg(feature = "png")]
        if let Some(ref rx) = *FRAME_RX.lock().unwrap() {
            let rx_clone = (*rx).clone();
            std::thread::spawn(move || frame_distributor_task(rx_clone));
        }
        
        loop {
            match listener.accept() {
                Ok((stream, peer)) => {
                    eprintln!("[API] Client connected: {}", peer);
                    let state = Arc::clone(&state);
                    std::thread::spawn(move || {
                        if let Err(e) = handle_client(stream, state) {
                            eprintln!("[API] Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[API] Accept error: {}", e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
    
    fn frame_distributor_task(rx: crossbeam_channel::Receiver<CapturedFrame>) { ... }
    fn handle_client(stream: TcpStream, state: Arc<Mutex<ApiState>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ... }
}
```

#### C. Replace `async_tungstenite` with sync `tungstenite`

**Before:**
```rust
let mut ws = async_tungstenite::tokio::accept_async(stream).await?;
// ... use futures_util::StreamExt for ws.next().await
ws.send(Message::binary(...)).await?;
```

**After:**
```rust
use tungstenite::server::{accept, Incoming};
use tungstenite::Message;

let mut ws = accept(stream)?;  // Blocking handshake

// Message loop (blocking)
loop {
    match ws.read_message() {
        Ok(Message::Text(text)) => handle_message(&text, &state, &mut ws)?,
        Ok(_) => continue,
        Err(tungstenite::Error::ConnectionClosed) | 
        Err(tungstenite::Error::AlreadyClosed) => break,
        Err(e) => { eprintln!("Read error: {}", e); break; }
    }
    
    // Send responses synchronously
    ws.write_message(Message::text(serde_json::to_string(&resp)?))?;
}
```

#### D. Replace per-client tokio mpsc with crossbeam channel

**ClientCapture struct:**
```rust
struct ClientCapture {
    tx: crossbeam_channel::Sender<CapturedFrame>,  // Changed from tokio mpsc
    rx: crossbeam_channel::Receiver<CapturedFrame>,
    id: usize,
    mode: CaptureMode,
}
```

**Channel creation in handle_client:**
```rust
let (client_tx, client_rx) = crossbeam_channel::unbounded::<CapturedFrame>();
captures.push(ClientCapture { tx: client_tx, rx: client_rx, id: client_id, mode: CaptureMode::None });
```

**Message loop with frame delivery:**
```rust
loop {
    // Use select!-like behavior with crossbeam's select macro
    use crossbeam::channel::{select, RecvError};
    
    select! {
        recv(client_rx) -> result => {
            if let Ok(frame) = result {
                send_png_frame(&mut ws, &frame)?;
            } else { break; }
        },
        recv(ws_incoming) -> msg => {  // Need to wrap ws in a channel or use try_recv
            match msg { ... }
        },
    }
}
```

**Simpler approach without crossbeam select:** Just poll frames first, then WS:
```rust
loop {
    // Try to get a frame (non-blocking)
    if let Ok(frame) = client_rx.try_recv() {
        send_png_frame(&mut ws, &frame)?;
        continue;  // Go back and check for more frames/messages
    }
    
    // No frame available, wait for WS message with timeout
    match ws.read_message() { ... }
}
```

#### E. Update `send_png_frame` to be synchronous

**Before:**
```rust
async fn send_png_frame(ws: &mut ..., frame: &CapturedFrame) -> Result<...> { ... ws.send(...).await?; }
```

**After:**
```rust
fn send_png_frame(ws: &mut WebSocketStream<TcpStream>, frame: &CapturedFrame) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // PNG encoding (synchronous, already is)
    let mut buf = Vec::new();
    let mut encoder = png::Encoder::new(&mut buf, frame.width as u32, frame.height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&frame.pixels).unwrap();
    writer.finish().unwrap();
    
    // Build binary message
    let header = [(frame.width >> 8) as u8, (frame.width & 0xFF) as u8,
                   (frame.height >> 8) as u8, (frame.height & 0xFF) as u8];
    let mut binary = Vec::with_capacity(4 + buf.len());
    binary.extend_from_slice(&header);
    binary.extend_from_slice(&buf);
    
    // Send synchronously
    ws.write_message(Message::binary(binary))?;
    Ok(())
}
```

#### F. Update `handle_message` to be synchronous

**Before:**
```rust
async fn handle_message(text: &str, state: &Arc<Mutex<ApiState>>, ws: &mut ...) -> Result<bool, ...> { ... }
```

**After:**
```rust
fn handle_message(
    text: &str, 
    state: &Arc<Mutex<ApiState>>, 
    ws: &mut WebSocketStream<TcpStream>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let msg = serde_json::from_str::<ClientMessage>(text)?;
    
    match msg {
        ClientMessage::Step => {
            state.lock().unwrap().request_frame_step();
            // Set capture mode...
            ws.write_message(Message::text(serde_json::to_string(&ServerMessage::FrameDone)?))?;
            return Ok(true);
        }
        // ... other cases similar
    }
    
    Ok(false)
}
```

### 4. `rustsdlretro-frontend/src/main.rs` — Minor Changes

Update `#[cfg(feature = "api")]` blocks to remove tokio-specific cfg checks:

**Before:**
```rust
#[cfg(all(feature = "tokio", feature = "tungstenite"))]
pub static FRAME_TX: ...
```

**After:**
```rust
#[cfg(feature = "png")]  // or just #[cfg(feature = "api")]
pub static FRAME_TX: ...
```

### 5. `tests/README.md` — Update

Remove references to tokio issues and document the new synchronous architecture.

## Implementation Order

1. **Update Cargo.toml** — Remove dependencies, update features
2. **Rewrite api.rs server module** — Replace async with sync implementation
3. **Update frontend main.rs** — Fix cfg attributes  
4. **Build & test** — Verify WebSocket connections work
5. **Test PNG streaming** — Verify frame delivery works
6. **Update documentation**

## Testing Strategy

1. Build: `cargo build --release --features "minifb,api"`
2. Start emulator: `./start_psx.sh` (or any game)
3. Test API: `node test_api.js` — Play/SaveState/LoadState/Step commands
4. Test PNG streaming: `node tests/test_png_stream.js --max-frames 3`

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Thread-per-client doesn't scale to many connections | API is for development/debugging, not production; ~5 clients max expected |
| Blocking I/O on WS read blocks frame delivery | Use `try_recv()` for frames, non-blocking check before blocking WS read |
| PNG encoding blocks the client thread briefly | Encoding 320x240 RGBA → PNG takes <1ms, negligible |
| crossbeam channel select complexity | Simple loop: try_recv frames first, then ws.read_message() |

## Notes

- The `tungstenite` crate provides synchronous server APIs via `server::accept()` 
- No TLS needed for local development WebSocket (ws:// not wss://)
- Crossbeam channels work perfectly for sync multi-threaded scenarios
- Thread-per-client is fine for this use case (<10 concurrent connections typical)
