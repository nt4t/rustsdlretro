# Per-Frame Timing Analysis

## Test Setup

- **Core**: FCEUmm (NES emulator)
- **ROM**: Contra (USA).nes
- **Target FPS**: 60.10 FPS (NES PPU at 60.0988 Hz)
- **Measurement**: Instrumented main loop, 60-frame rolling window averages

## Results

| Phase | Avg Time | Peak | Notes |
|-------|----------|------|-------|
| **run** (core.run / emulator) | 3–10 µs | 131 µs | FCEUmm extremely fast — negligible |
| **wait** (throttle sleep) | ~16,200 µs | ~16,640 µs | Throttle sleeps to maintain target FPS |
| **render** (gui.render) | 0 µs | — | GUI overlay only drawn when menu is open |
| **window** (update_window) | 0 µs | — | fbdev `update_window` is a no-op |
| **total** | ~16,640 µs | — | Matches 60.1 FPS target |

## Key Findings

### 1. Throttle achieves stable 60 FPS
The throttle sleeps ~16,200 µs per frame, resulting in a total of ~16,640 µs/frame — exactly matching the 60.1 FPS target. The emulator runs at a stable **60.1 FPS**.

### 2. Rendering is essentially free
- `gui.render` measures 0 µs because the GUI overlay is only drawn when the menu is open, not during gameplay
- `update_window` measures 0 µs because fbdev writes directly to the mmap'd framebuffer during `push_frame`, so `update_window` is a no-op

### 3. The emulator is extremely fast
- FCEUmm runs in ~5 µs per frame on average
- This is ~0.03% of the 16.6ms budget at 60 FPS
- Even at peak (131 µs), it's only ~0.8% of the budget

### 4. Spikes in `run` time
Occasional spikes (80–130 µs, ~2% of frames) are likely from:
- Video format conversion (32bpp↔16bpp pixel copying)
- Garbage collection pauses
- OS scheduler jitter

## Throttle Design

The `Throttle` struct tracks `next_frame` (absolute timestamp of the next scheduled frame) and `frame_time` (1,000,000 / fps in microseconds).

### `check_wait()` logic

```rust
pub fn check_wait(&mut self) -> i64 {
    let now = now_usec();
    let result = self.next_frame as i64 - now as i64;
    if result > 0 {
        result  // Sleep for `result` microseconds
    } else {
        self.next_frame += self.frame_time;  // Advance schedule by one frame
        result  // Negative value — throttle is behind schedule
    }
}
```

**Key insight:** When behind schedule, `next_frame` is advanced by `frame_time` (not reset to `now + frame_time`). This prevents frames from accumulating and keeps timing stable.

## Bug Fixes

### 1. Timing instrumentation bug (fixed)
`timing_count` was never reset after printing, causing subsequent averages to divide by 60 instead of the actual frame count. This made wait times appear ~60× smaller than they were.

### 2. Throttle schedule reset bug (fixed)
When behind schedule, `next_frame = now + frame_time` reset the schedule to the current time, causing the next `check_wait()` to return 0 and skip sleeping. This resulted in ~57 FPS instead of 60.

**Fix:** Changed to `next_frame += frame_time` to advance the schedule by one frame instead of resetting.

### 3. Audio choppiness (fixed)
Ring buffer drained to 0 between reads because ALSA writes consumed samples as fast as the core produced them.

**Fixes applied:**
- Ring buffer capacity: 65k → 262k samples
- Added accumulation buffer (8k samples) before ALSA writes
- ALSA buffer: default (~10ms) → 256ms, period → 64ms

## Known: Ring Buffer Always Empty

The ring buffer consistently shows 0/262144 samples, yet audio plays smoothly. This is expected because:

1. **ALSA buffer absorbs the gap**: The 256ms ALSA hardware buffer holds ~12,288 samples, providing ~256ms of audio buffer independent of the ring buffer.

2. **Accumulation buffer batches writes**: The playback thread accumulates 8k samples before writing to ALSA, reducing write frequency from 60Hz → ~15Hz.

3. **Ring buffer is a pass-through**: Samples flow Core → Ring Buffer → Accumulation → ALSA. The ring buffer never accumulates because ALSA drains it as fast as the core fills it, but the ALSA buffer provides the actual audio smoothing.

```log
audio: rb=0/262144 (0.0s), accum=4948/8192  # Ring empty, accum fluctuating
audio: rb=0/262144 (0.0s), accum=3304/8192  # Still empty, smooth playback
audio: rb=172/262144 (0.0s), accum=0/8192   # Rarely fills slightly
```

**Conclusion:** Ring buffer size is not the bottleneck — ALSA hardware buffer (256ms) is what makes audio smooth. The ring buffer could potentially be reduced, but keeping it large provides a safety margin.

## Code Locations

- **Timing instrumentation**: `rustsdlretro-frontend/src/main.rs` — main loop timing measurements
- **Throttle logic**: `rustsdlretro-core/src/lib.rs` — `Throttle` struct, `check_wait()`, `skip_check()`
- **Video push**: `rustsdlretro-core/src/video.rs` — `FbdevVideo::push_frame()`
- **GUI render**: `rustsdlretro-core/src/gui.rs` — `Gui::render()`

## Comparison: With vs Without Throttle

| Metric | Without Throttle | With Throttle |
|--------|-----------------|---------------|
| FPS | 4100+ | 60.1 |
| Audio | Choppy (buffer overflow) | Smooth |
| Per-frame time | ~240 µs | ~16,640 µs |

The throttle is essential for correct audio timing. Without it, the emulator runs at 4000+ FPS, producing audio samples far too fast and overflowing the ring buffer.
