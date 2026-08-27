# PNG Frame Streaming Test

Tests WebSocket binary frame delivery for rustsdlretro's PNG streaming feature.

## Usage

```bash
node test_png_stream.js                          # defaults to ws://localhost:18932
node test_png_stream.js --url ws://host:port     # custom address
node test_png_stream.js --max-frames 5           # stop after N frames (default: 3)
```

## Test Flow

1. Connect to WebSocket server
2. Send `Play` command -> start emulation
3. Receive JSON Status response with FPS/resolution
4. Send `Step` command -> trigger one frame
5. Capture binary PNG frames sent as `[width u16 BE][height 16 BE][PNG bytes]`
6. Validate PNG structure (signature, IHDR, IDAT)
7. Save valid frames to `./test_output/frame_001.png`, etc.

## Requirements

- Node.js 18+ with `ws` package installed in project root
- rustsdlretro running with `--features api,minifb` and a loaded core/game

## Known Issues

### PNG Frame Delivery Not Working

The WebSocket server accepts connections and handles JSON commands (Play, Step, etc.) correctly,
but PNG frames are not delivered to clients after a Step command. Debugging shows:

- Client connects successfully [OK]
- Play/Step commands processed correctly [OK]  
- Distributor receives frames from video backend [OK]
- Distributor sends frames to client channel (`cap.tx.send()`) [OK]
- **Client handler never receives frames via `client_rx.try_recv()` [FAIL]**

The root cause is still under investigation. Suspected issues:
1. tokio mpsc channel not delivering between tasks in the multi-threaded runtime
2. Tungstenite TLS feature flags interfering with plain TCP WebSocket handling
3. Race condition between distributor send and client recv timing

### Server Runtime Fix Applied

The tokio runtime was changed from `new_current_thread()` to `new_multi_thread()` (2 workers)
because the single-threaded runtime failed to accept any TCP connections.

## Build Requirements

```bash
cargo build --release --features "minifb,api"
```

Note: The `api` feature enables tungstenite with rustls-tls features. If WebSocket handshakes
fail, try removing TLS feature flags from Cargo.toml.
