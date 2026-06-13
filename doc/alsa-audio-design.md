# ALSA Audio Support Design

## Overview
Replace stubbed audio callbacks with real ALSA PCM output in `sdlretro-core`. Audio playback runs in a dedicated background thread that drains a ring buffer. The core writes samples into the ring buffer via `audio_sample_batch_cb`. No resampling — ALSA device sample rate matches the core's `sample_rate` from `retro_system_av_info.timing.sample_rate`.

## Design Decisions
- **No resampling**: ALSA configured to match core sample rate (32kHz/44.1kHz/48kHz). Simpler, no external dependency.
- **Separate ALSA thread**: Dedicated playback thread drains ring buffer into ALSA. Decouples audio from frame timing.
- **`alsa` crate**: Idiomatic Rust bindings to libasound. Type-safe, well-maintained.

## Architecture

### New Module
```
sdlretro-core/
├── src/
│   ├── lib.rs           # audio_sample_cb, audio_sample_batch_cb (FFI callbacks), MAIN_AUDIO static
│   ├── audio.rs         # AudioDriver, RingBuffer, playback thread
│   ├── video.rs         # FbdevVideo
│   └── input.rs         # InputReader
```

### Dependencies
Add to `sdlretro-core/Cargo.toml`:
```toml
[dependencies]
alsa = "0.9"
```

### Static Registry
Same pattern as `MAIN_VIDEO` and `MAIN_INPUT`:
```rust
static mut MAIN_AUDIO: Option<AudioDriver> = None;
```

## AudioDriver Struct

```rust
pub struct AudioDriver {
    sample_rate: u32,
    pcm: Option<pcm::Pcm>,
    pcm_params: Option<pcm::Params>,
    ring_buffer: Arc<Mutex<RingBuffer>>,
    thread_handle: Option<JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
}

pub struct RingBuffer {
    data: Vec<i16>,
    capacity: usize,
    read_pos: usize,
    write_pos: usize,
    count: usize,
}
```

## Lifecycle

### Initialization
1. `AudioDriver::new(sample_rate)` — create ring buffer (8192 samples), open ALSA PCM in `SND_PCM_STREAM_PLAYBACK` mode, configure hardware parameters (format: S16_LE, channels: 2, sample_rate from core), start playback thread.

### Push from Core
2. `AudioDriver::push_batch(data)` — called from `audio_sample_batch_cb`, locks ring buffer mutex, copies samples into buffer, unlocks. Drops excess if buffer full.

### Sample Rate Change
3. `AudioDriver::restart_with_rate(new_rate)` — called when core sends `SET_SYSTEM_AV_INFO` with new sample rate. Stops old thread, closes old PCM, re-opens with new rate, starts new thread. Ring buffer preserved.

### Shutdown
4. `AudioDriver::stop()` — set `stopped = true`, join thread, close PCM.

### Ring Buffer Parameters
- **Capacity**: 8192 samples (16384 bytes stereo)
- **Latency**: ~187ms at 44.1kHz
- **Read batch size**: 1024 frames (configurable)

## Data Flow

```
Core (retro_run)
    │
    ├── audio_sample_batch_cb(int16_t* data, frames)
    │       │
    │       ▼
    │   MAIN_AUDIO.push_batch(data, frames)
    │       │
    │       ▼
    │   RingBuffer::write(data) — lock mutex, copy samples, unlock
    │       │
    │       ▼
    │   (main loop continues, no blocking)
    │
    │   ┌─────────────────────────────────┐
    │   │  ALSA playback thread (loop):   │
    │   │                                 │
    │   │  loop while !stopped:           │
    │   │    samples = RingBuffer::read() │
    │   │    if empty: sleep(1ms)         │
    │   │    else: snd_pcm_writei()       │
    │   │    if underrun: snd_pcm_recover │
    │   └─────────────────────────────────┘
```

## audio_sample_cb Handling
`audio_sample_cb(left, right)` is a thin wrapper that forwards each stereo pair directly into the ring buffer via `push_batch`. Most cores use `audio_sample_batch_cb` which also writes to the ring buffer. Both paths converge on the same buffer.

## Sample Rate Change Flow
When core calls `SET_SYSTEM_AV_INFO` with new `sample_rate`:
1. Update `AudioDriver.sample_rate`
2. `stopped.store(true)`, join old playback thread
3. `snd_pcm_close(old_pcm)`
4. Re-open ALSA PCM with new `hw_params.sample_rate = new_rate`
5. `stopped.store(false)`, start new playback thread
6. Ring buffer preserved (no data loss)

## Error Handling

| Scenario | Behavior |
|----------|----------|
| ALSA open failure at startup | Log error, fall back to stub (no audio, no crash) |
| PCM write failure (EPIPE/XRUN) | `snd_pcm_recover(pcm, err, 1)`, log warning, continue |
| Ring buffer full | Drop oldest samples (core producing faster than playback) |
| Thread join timeout | Log error, abandon thread (process exit cleans up) |
| ALSA unavailable | Frontend continues silently with stub callbacks |

## Changes to Existing Code

### `sdlretro-core/src/lib.rs`
- Add `pub mod audio;`
- Replace `audio_sample_cb` and `audio_sample_batch_cb` stubs with real implementations that route through `MAIN_AUDIO`
- Add `MAIN_AUDIO: Option<AudioDriver>` static

### `sdlretro-core/src/audio.rs` (new)
- `AudioDriver` struct
- `RingBuffer` struct with lock-free-ish mutex-based access
- Playback thread loop
- `restart_with_rate()` for sample rate changes

### `sdlretro-frontend/src/main.rs`
- Initialize `AudioDriver` after getting `sample_rate` from `retro_get_system_av_info()`
- Store in `MAIN_AUDIO` static
- Call `audio_driver.stop()` during shutdown before `core.unload()`
- Handle sample rate changes in main loop (call `restart_with_rate()` when FPS changes and sample rate differs)

### `sdlretro-core/Cargo.toml`
- Add `alsa = "0.9"` dependency

## Testing
- **Unit**: Ring buffer read/write correctness, overflow handling
- **Integration**: Load a core with audio (e.g., Genesis-Plus-GX), verify sound output
- **Sample rate change**: Load a core that changes sample rate mid-game, verify seamless transition
- **Graceful degradation**: Run without ALSA device, verify frontend still works (silent)
