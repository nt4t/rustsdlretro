# PNG Frame Capture via WebSocket API

## Overview

The API supports capturing individual frames as PNG images over WebSocket, triggered by the `Step` command. This is useful for debugging, frame-accurate analysis, or generating screenshots from emulation.

## Architecture

### Capture Modes

Each client has one of three capture modes:

| Mode | Description |
|------|-------------|
| `None` | Default state - no PNG frames sent to this client |
| `Continuous` | All received frames are forwarded as PNG (reserved for future use) |
| `SingleStep` | One frame is captured and sent per Step command |

### Flow

```
Client                          Server
  |                               |
  |-- {"type":"Play"} ----------> |
  |<-- Status{running:true} ------ |
  |                               |
  |-- {"type":"Step"} ----------> | Sets all clients to SingleStep
  |<-- FrameDone ----------------- | Emulation runs one frame
  |                               | Video backend calls push_captured_frame()
  |                               | Distributor checks capture mode → forwards frame
  |<-- Binary[width][height][PNG] | Client receives PNG frame
  |                               | Mode resets to None
```

### Data Flow

1. **Video Backend** (e.g., `video_minifb.rs`) calls `crate::api::push_captured_frame(frame)` each frame
2. Frame enters global crossbeam channel (`FRAME_TX`, capacity 500)
3. **Distributor task** reads from channel, checks per-client capture mode, forwards to matching clients
4. **Client handler** receives frame via per-client channel, encodes as PNG, sends over WebSocket
5. Capture mode resets to `None` after sending (single-step behavior)

### Binary Frame Format

PNG frames are sent as binary WebSocket messages with this structure:

```
[0-1]  Width (u16 big-endian)
[2-3]  Height (u16 big-endian)
[4+]   PNG-encoded image data
```

The client can decode the width/height from the first 4 bytes, then pass the rest to a PNG decoder.

## API Messages

### Client → Server: Step

Triggers single-frame capture for all connected clients.

```json
{"type": "Step"}
```

Server responds with `FrameDone`, runs one frame of emulation, then sends PNG frames to capturing clients.

### Server → Client: Binary Frame

After a Step command, each client that was in `SingleStep` mode receives a binary message containing the captured frame.

## Configuration

Channel capacity is set to 500 frames to prevent overflow during continuous playback when no clients are capturing. Frames are dropped silently if the channel is full (non-blocking send).

## Debug Logging

Enable with `--features "minifb,api"` and watch stderr for:

- `[API] push_captured_frame called, width=... height=...` - Video backend pushing frames
- `[API][CAP] Frame received in distributor, width=...` - Distributor processing
- `[API][CLIENT] Client N registered/unregistered` - Connection lifecycle
- `[API][CLIENT] Client received frame, encoding PNG...` - Capture active
