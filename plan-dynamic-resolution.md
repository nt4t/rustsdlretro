# Plan: Dynamic Resolution Handling

## Problem

Some libretro cores (Genesis-Plus-GX, snes9x, etc.) change resolution dynamically during gameplay.
Example: `MUSHA - Metallic Uniframe Super Hybrid Armor` switches between 384x224 and 512x448.

Current state: resolution is captured **once** after `retro_load_game()` and never updated.
Result: when the core switches to a different resolution, frames are drawn with wrong dimensions,
causing stretched/squeezed/offset rendering.

## Root Cause

1. `sdlretro-frontend/src/main.rs:107-121` calls `get_system_av_info()` once, then `set_core_format()` once.
2. `sdlretro-core/src/lib.rs:79-113` (`log_environment_cb`) does not handle keys 32 or 37.
3. `FbdevVideo::core_width`/`core_height` are never updated after initialization.

## Libretro APIs for Resolution Changes

| Key | Define | When | Effect |
|-----|--------|------|--------|
| 32 | `RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO` | Anytime (not in init) | Full reinit of video/audio with new geometry+timing |
| 37 | `RETRO_ENVIRONMENT_SET_GEOMETRY` | Only inside `retro_run()` | Resize viewport only, no driver reinit |

Both pass a pointer to `retro_game_geometry` (width, height, max_width, max_height, aspect_ratio).

## Constraints

- **Framebuffer is fixed**: `/dev/fb0` resolution cannot change at runtime on most devices.
  The approach is to keep the framebuffer fixed and adjust **letterboxing/viewport** instead.
- **No windowing system**: No resize events, no repainting. Just adjust where frames are drawn.
- **FPS may change**: `SET_SYSTEM_AV_INFO` can change timing.fps, which affects frame throttling.

## Design

### 1. Add environment command handlers

In `sdlretro-core/src/lib.rs`, extend the environment callback to handle:

- **Key 32** (`SET_SYSTEM_AV_INFO`): Update `CORE_FORMAT.width`, `CORE_FORMAT.height`, and expose new FPS.
- **Key 37** (`SET_GEOMETRY`): Update `CORE_FORMAT.width`, `CORE_FORMAT.height`.

The callback needs access to `FbdevVideo` to call `set_core_format()`.

### 2. Share resolution state between core and frontend

Approach: Use a shared `Arc<Mutex<ResolutionState>>` passed to the core at init time.

```rust
pub struct ResolutionState {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}
```

The environment callback reads/writes this state, and the frontend's main loop checks for FPS changes
to update the throttle.

### 3. Main loop: detect FPS changes

In the main loop, after `core.run()`, check if FPS changed. If so, recreate the `Throttle`.

```rust
let av_info = core.get_system_av_info();  // or read from shared state
if av_info.timing.fps != throttle_fps {
    throttle = Throttle::new(av_info.timing.fps);
    throttle_fps = av_info.timing.fps;
}
```

### 4. Handle max_width/max_height (optional, future)

Some cores (NES emulators) set `max_width > base_width` for multi-screen modes.
For now, ignore `max_width`/`max_height` and only use `base_width`/`base_height`.

## Implementation Steps

### Step 1: Add ResolutionState struct (`sdlretro-core/src/lib.rs`)

```rust
pub struct ResolutionState {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}
```

### Step 2: Pass ResolutionState to Core (`sdlretro-core/src/lib.rs`)

Add a field and setter to `Core`:

```rust
pub struct Core {
    handle: *mut c_void,
    need_fullpath: bool,
    resolution: Option<Arc<Mutex<ResolutionState>>>,
}

impl Core {
    pub fn set_resolution_state(&mut self, state: Arc<Mutex<ResolutionState>>) {
        self.resolution = Some(state);
    }
}
```

### Step 3: Handle keys 32 and 37 in environment callback (`sdlretro-core/src/lib.rs`)

Extend `log_environment_cb` (or create a new unified environment callback) to:
- Key 32: Read `retro_game_geometry` from data, update ResolutionState
- Key 37: Same as key 32 but only updates geometry (not timing)

The callback needs to also call `FbdevVideo::set_core_format()` to update letterboxing offsets.

### Step 4: Update frontend main loop (`sdlretro-frontend/src/main.rs`)

- Create `ResolutionState` with initial values from `get_system_av_info()`
- Pass it to `core.set_resolution_state()`
- In main loop, check FPS changes and recreate `Throttle` if needed

### Step 5: Test with Genesis-Plus-GX

Run MUSHA and verify resolution switches are handled correctly (no stretched/squeezed frames).

## Files to Modify

| File | Changes |
|------|---------|
| `sdlretro-core/src/lib.rs` | Add ResolutionState, pass to core, handle env keys 32/37 |
| `sdlretro-core/src/video.rs` | Ensure `set_core_format()` recomputes letterboxing |
| `sdlretro-frontend/src/main.rs` | Create ResolutionState, pass to core, detect FPS changes in loop |

## Risks

- **Thread safety**: Environment callback runs from core's `retro_run()` context. ResolutionState must be
  protected with Mutex. `FbdevVideo::set_core_format()` is called from the callback — must not conflict
  with `push_frame()` which runs from the video_refresh callback.
- **Frame tearing**: If resolution changes mid-frame, the next frame may have wrong dimensions.
  Acceptable tradeoff for embedded fbdev target.
- **SET_GEOMETRY restriction**: Per libretro spec, key 37 can only be called from `retro_run()`.
  The core calls it before `retro_video_refresh`, so letterboxing will be updated in time for the frame.
