# Plan: Fix ALSA Broken Pipe (EPIPE) in Playback Thread

**Status: DONE** — implemented and verified (2026-07-31).

Verification results:
- `cargo test`: 8/8 lib tests pass
- 10 s live run: zero "Broken pipe" lines, steady 58 FPS
- Concurrent `aplay` sharing test: no EPIPE storm, both streams coexist
- SIGINT shutdown: `playback_thread: stopping (stopped=true)`, prompt exit

Extra fixes found along the way (pre-existing, previously hidden because the
test suite didn't compile in debug mode):
- `test_read_batch_limit` had an arithmetic overflow (`2048 - 8192`) — rewritten
- `RingBuffer::write` now implements the documented drop-oldest semantics
  (previously discarded incoming samples when full, which would freeze audio
  if the consumer lagged)
- `test_write_full_drops_oldest` read only one batch (8192) — now reads in a loop

## Symptom

10-second test run of `./start_snes.sh`:

```
playback_thread: 1 total ALSA writes, 798 frames (1596 samples) written this call
ALSA write error: ALSA function 'snd_pcm_writei' failed with error 'Broken pipe (32)', recovering...
ALSA write error: ... (repeats forever)
```

First write succeeds, then `snd_pcm_writei` fails with EPIPE on every subsequent
attempt. The "recovery" (drain + recover + start) never sticks. Game video runs
fine at ~57 FPS; audio is effectively broken/chopped.

## Root Cause

`AudioDriver::new()` opens the device with **non-blocking mode**:

```rust
alsa::pcm::PCM::new(dev_name, alsa::Direction::Playback, true)  // true = nonblock
```

In non-blocking mode, `snd_pcm_writei` returns **`-EPIPE` whenever the PCM
buffer cannot accept the full request** (i.e. it would block). The playback
thread writes up to 8192 samples (4096 frames) per call, while the ALSA buffer
is much smaller. Sequence:

1. Write 4096 frames → only 798 fit → EPIPE (partial write discarded!)
2. "Recovery" drains the buffer → next write fits a bit more → EPIPE again
3. Infinite loop: drain → partial write → drain → partial write
4. Every EPIPE also **drops audio samples** (the unwritten remainder of the
   local buffer is never retried) — hence choppy/no sound

The current error handler treats EPIPE as a fatal hardware error, but here it
is the normal "buffer full" signal of non-blocking I/O.

## Fix Design

All changes in `rustsdlretro-core/src/audio.rs`, function
`playback_thread_loop()` (plus small helpers). Keep `nonblock=true` (needed for
prompt shutdown) and do proper non-blocking I/O.

### 1. Write-all loop with offset tracking

Replace the single `writei` call with a loop that writes the whole local
buffer before the iteration ends:

```rust
let mut offset = 0usize; // frames written so far
while offset < frames_total {
    match io.writei(&buffer[offset*2..]) {
        Ok(n) => offset += n,
        Err(e) if e.errno() == EPIPE || e.errno() == EAGAIN => {
            // buffer full or xrun: wait for space, then retry
            wait_for_space(pcm);
        }
        Err(e) => { return ErrKind::Fatal(e); }  // real hardware error
    }
}
```

No samples are dropped: the local buffer is fully written (or the stream is
recovered) before the next ring-buffer read.

### 2. `wait_for_space(pcm)` — poll instead of spin

```rust
fn wait_for_space(pcm: &PCM) {
    let mut fds = pcm.get().unwrap_or_default();   // poll::Descriptors::get
    alsa::poll::poll(&mut fds, 100).unwrap_or(0);  // 100 ms timeout
    match pcm.state() {
        State::Xrun => { let _ = pcm.recover(libc::EPIPE, true); let _ = pcm.start(); }
        State::Stopped => { let _ = pcm.start(); }
        _ => {}
    }
}
```

- Poll with a **100 ms timeout** so the loop can re-check the `stopped` flag
  (keeps shutdown prompt).
- Only run drain/recover/start on an actual **XRUN** state, not on every EPIPE.

### 3. Distinguish transient vs fatal errors

| errno | meaning | action |
|---|---|---|
| `EPIPE` / `EAGAIN` | buffer full / xrun | poll + retry (no recovery storm) |
| `EIO` | real hardware/IO error | drain + `recover(EIO)` + `start()` (existing path) |
| other | unexpected | treat as fatal |

The existing `recover()` + `start()` path is kept **only for EIO**, and the
`start()` result is no longer ignored (`let _ =` → checked, logged).

### 4. Consecutive-failure fallback + log rate limiting

- Track `consecutive_failures`. After **5** fatal failures in a row, reopen the
  PCM device once (close + `PCM::new` + hw_params + start, reusing the logic
  from `restart_with_rate`). If that also fails, fall back to a **silent stub**:
  the thread keeps draining the ring buffer (so `push_batch` never blocks or
  grows unbounded) and logs a single "audio disabled" line.
- Rate-limit error logging: at most one line per second (the current code
  spams one line per failed write, ~thousands/sec).
- Keep the existing periodic "N total ALSA writes" heartbeat (every 1000).

### 5. (Optional, small) Configurable audio device

Add `audio.device` to `config.json` (default `"default"`), passed into
`AudioDriver::new(device, rate)`. Useful for Raspberry Pi where `default` may
not exist; keeps the existing `["default", "plughw:0,0", "hw:0,0"]` fallback
chain for the default case.

## Files Touched

- `rustsdlretro-core/src/audio.rs` — playback loop rewrite, helpers, fallback
- `rustsdlretro-core/src/config.rs` — optional `audio.device` field (step 5)
- `rustsdlretro-frontend/src/main.rs` — pass device name to `AudioDriver::new` (step 5)
- `doc/config-example.json`, `README.md` — document `audio.device` (step 5)

## Verification

1. `cargo build --release && cargo test` — existing ring-buffer tests pass.
2. `timeout 10 ./start_snes.sh`:
   - expect **zero** "Broken pipe" lines,
   - steady "N total ALSA writes" heartbeat (~60/s at 48 kHz with 8192-sample
     batches),
   - audible, uninterrupted game audio.
3. Sharing test: run with a second audio app playing simultaneously
   (`aplay /usr/share/sounds/freedesktop/stereo/beep.oga &`) — no EPIPE storm,
   both audible (PipeWire "default" mixes clients).
4. Shutdown test: exit with ESC/Ctrl+C — `stop()` returns promptly (poll
   timeout bounds the blocked write), no hangs.

## Out of Scope

- Resampling (core rate ≠ device rate) — `restart_with_rate` already handles
  rate changes by reopening.
- MMAP access — RWInterleaved is fine at these sizes.
