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
2. Send `Play` command → start emulation
3. Receive JSON Status response with FPS/resolution
4. Send `Step` command → trigger one frame
5. Capture binary PNG frames sent as `[width u16 BE][height u16 BE][PNG bytes]`
6. Validate PNG structure (signature, IHDR, IDAT)
7. Save valid frames to `./test_output/frame_001.png`, etc.

## Requirements

- Node.js 18+ with `ws` package installed in project root
- rustsdlretro running with `--features api,minifb` and a loaded core/game
