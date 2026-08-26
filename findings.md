# Findings — Web API Implementation

## tungstenite + async-tungstenite Integration

### Version Compatibility Issue
- `async-tungstenite` 0.28 depends on `tungstenite` 0.24 (version mismatch)
- We need `tungstenite` 0.30 for rustls support without system OpenSSL
- Solution: use `async-tungstenite` 0.35 which depends on `tungstenite` 0.30

### Async vs Blocking I/O
- Initial attempts used `spawn_blocking` + blocking tungstenite API — failed due to non-blocking socket issues from tokio
- Final solution: fully async using `async_tungstenite::tokio::accept_async()`
- This wraps the Tokio stream with `TokioAdapter<T>` which implements futures_io traits

### Key API patterns:
```rust
// Server side - accept connection on tokio TcpStream
let mut ws = async_tungstenite::tokio::accept_async(stream).await?;

// Message loop using futures StreamExt
use futures_util::StreamExt;
match ws.next().await {
    Some(Ok(msg)) => { /* handle */ }
    Some(Err(e)) => { /* error */ }
    None => { break; } // disconnected
}

// Send response
ws.send(Message::text(json_string)).await?;
```

## Dependency Decisions

### tungstenite version
- Started with `0.23` (native-tls) → required OpenSSL dev libraries
- Upgraded to `0.30` with `rustls-tls-webpki-roots` → zero system dependencies needed
- Feature names changed across versions: `native-tls` → `rustls-tls-webpki-roots`

### tokio dependency
- Required for tungstenite async server integration
- Using `features = ["full"]` since we need TcpListener + runtime
- Alternative would be to use a blocking server, but async is cleaner for WebSocket handling

## Architecture Decisions

### Shared state pattern
- Chose `Arc<Mutex<ApiState>>` over atomics for simplicity
- Main loop holds the Arc clone; server thread locks/unlocks as needed
- All mutations go through mutex — no data races possible

### Server placement
- Initially planned separate `api_server.rs` file
- Consolidated into `api_server` submodule inside `api.rs` to avoid extra lib.rs module declarations
- Uses `#[cfg(all(feature = "tokio", feature = "tungstenite"))]` for conditional compilation

## Integration Notes (Phase 3)

### Frontend integration points:
1. Parse `--api-port N` CLI arg in main.rs — only creates API state if flag present
2. Create ApiState with `create_api_state(Arc<Mutex<ResolutionState>>)` when api feature is enabled
3. Main loop checks: `is_running()` before core.run() — paused/stepped frames skip emulation but render GUI
4. Input mapping: API snapshot → `InputReader::set_button()` for each port's 12 buttons
5. Save/Load: check flags, call existing save/load logic (same as F2/F4)
6. Options: queue pending changes → apply via CoreOptions::set_v2_value()

### Key design decisions:
- `InputReader::set_button()` added to allow direct state manipulation from API
- SharedApiState is created AFTER ROM load so resolution is known
- API input overrides keyboard input for web-connected ports (no merging)
- `type SharedApiState = ()` fallback when api feature disabled

### Existing code patterns to follow:
- Static globals use `unsafe` blocks (MAIN_VIDEO, MAIN_INPUT) — new API state should be cleaner with Arc
- Feature gates use `#[cfg(feature = "...")]` consistently
- Error handling uses Result types with descriptive messages
