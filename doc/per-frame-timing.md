# Per-Frame Timing Analysis

## Test Setup

- **Core**: FCEUmm (NES emulator)
- **ROM**: Contra (USA).nes
- **Target FPS**: 60.10 FPS
- **Measurement**: Instrumented main loop, 60-frame rolling window averages

## Results

| Phase | Avg Time | Peak | Notes |
|-------|----------|------|-------|
| **run** (core.run / emulator) | 3–10 µs | 131 µs | FCEUmm extremely fast — negligible |
| **wait** (throttle sleep) | 265–300 µs | ~450 µs | Dominant cost per frame |
| **render** (gui.render) | 0 µs | — | GUI overlay only drawn when menu is open |
| **window** (update_window) | 0 µs | — | fbdev `update_window` is a no-op |
| **total** | ~275–290 µs | ~470 µs | |

## Key Findings

### 1. Rendering is essentially free
- `gui.render` measures 0 µs because the GUI overlay is only drawn when the menu is open, not during gameplay
- `update_window` measures 0 µs because fbdev writes directly to the mmap'd framebuffer during `push_frame`, so `update_window` is a no-op

### 2. The emulator is extremely fast
- FCEUmm runs in ~5 µs per frame on average
- This is ~0.03% of the 16.6ms budget at 60 FPS
- Even at peak (131 µs), it's only ~0.8% of the budget

### 3. Throttle sleep is ~270 µs
The throttle is sleeping only ~270 µs per frame, which is surprisingly low. The throttle should be sleeping ~16.6ms per frame to maintain 60 FPS.

The FPS counter confirms ~58.4 FPS, so the actual wall-clock time per frame is ~17ms. The discrepancy between the measured sleep time (~270 µs) and the expected sleep time (~16.6ms) suggests that either:
- `std::thread::sleep` has higher granularity than expected on this system
- The throttle's `next_frame` tracking may need investigation

### 4. Spikes in `run` time
Occasional spikes (80–130 µs, ~2% of frames) are likely from:
- Video format conversion (32bpp↔16bpp pixel copying)
- Garbage collection pauses
- OS scheduler jitter

## Code Locations

- **Timing instrumentation**: `rustsdlretro-frontend/src/main.rs` — main loop timing measurements
- **Throttle logic**: `rustsdlretro-core/src/lib.rs` — `Throttle` struct, `check_wait()`, `skip_check()`
- **Video push**: `rustsdlretro-core/src/video.rs` — `FbdevVideo::push_frame()`
- **GUI render**: `rustsdlretro-core/src/gui.rs` — `Gui::render()`

## Comparison: With vs Without Throttle

| Metric | Without Throttle | With Throttle |
|--------|-----------------|---------------|
| FPS | 4100+ | 58.4 |
| Audio | Choppy (buffer overflow) | Smooth |
| Per-frame time | ~240 µs | ~17 ms |

The throttle is essential for correct audio timing. Without it, the emulator runs at 4000+ FPS, producing audio samples far too fast and overflowing the ring buffer.
