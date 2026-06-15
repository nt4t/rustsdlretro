# Plan: Fix Core Options Runtime Changes

## Problem
When user changes options in the GUI menu, snes9x2010 doesn't apply them because:
1. Frontend updates `CoreOptions.v2_values` HashMap ✓
2. Frontend's `GET_VARIABLE` handler returns new values ✓
3. **BUT** `GET_VARIABLE_UPDATE` (key 17) always returns `false` ✗
4. snes9x2010 polls `GET_VARIABLE_UPDATE` every frame, sees `false`, never re-reads options

## Root Cause
The environment callback never handles `key == 17` (GET_VARIABLE_UPDATE). It falls through to the default case which returns `false`.

## Solution
Add a `variable_update_pending` flag that:
1. Gets set to `true` when user changes an option in the GUI
2. Gets returned as `true` from `GET_VARIABLE_UPDATE` handler
3. Gets reset after the core reads it (standard pattern: reset on next `GET_VARIABLE_UPDATE` call after returning true)

## Implementation

### 1. Add static flag in `lib.rs`
```rust
static mut VARIABLE_UPDATE_PENDING: bool = false;
```

### 2. Add GET_VARIABLE_UPDATE handler (key 17) in `lib.rs`
```rust
if key == 17 {
    let update = data as *mut bool;
    if !update.is_null() {
        unsafe {
            *update = VARIABLE_UPDATE_PENDING;
            eprintln!("GET_VARIABLE_UPDATE: returning {}", *update);
        }
    }
    return true;
}
```

### 3. Reset logic in GET_VARIABLE_UPDATE handler
After returning `true`, reset the flag on the next call (to match libretro spec: "since the last call to GET_VARIABLE"):
```rust
if key == 17 {
    let update = data as *mut bool;
    if !update.is_null() {
        unsafe {
            *update = VARIABLE_UPDATE_PENDING;
            if VARIABLE_UPDATE_PENDING {
                // Reset flag - core will re-read via GET_VARIABLE
                VARIABLE_UPDATE_PENDING = false;
            }
        }
    }
    return true;
}
```

### 4. Set flag when GUI changes options in `gui.rs`
After `core_opts.set_v2_value(&key, &value)`, set the pending flag:
```rust
unsafe { VARIABLE_UPDATE_PENDING = true; }
```

## Files to Modify
1. `sdlretro-core/src/lib.rs` - Add flag and GET_VARIABLE_UPDATE handler
2. `sdlretro-core/src/gui.rs` - Set flag when user changes options

## Testing
1. Load snes9x2010 core with a ROM
2. Press ESC to open menu
3. Change an option (e.g., "Frame Skip")
4. Close menu and play
5. Core should detect change via GET_VARIABLE_UPDATE and apply new setting

## Notes
- snes9x2010 already polls `GET_VARIABLE_UPDATE` every frame in its main loop
- snes9x2010's `check_variables()` function handles re-reading options when update flag is true
- This is the standard libretro pattern used by all modern cores
