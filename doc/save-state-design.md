# Save State + SRAM Persistence Design

## Goal

Implement full save state support (checkpoint saving/loading) and SRAM persistence for rustsdlretro, using libretro's serialization API.

## Trigger Mechanism

| Key | Action | Context |
|-----|--------|---------|
| F2  | Save current state | Anytime (gameplay or menu) |
| F4  | Load last saved state | Anytime (gameplay or menu) |

Edge-triggered: only fires on first frame after key press, like the existing ESC toggle.

## File Layout

```
~/.config/rustsdlretro/saves/
├── {core_name}/
│   ├── {game_name}.state    ← full save state snapshot (binary)
│   ├── {game_name}.sav      ← SRAM / battery RAM data (binary)
│   └── {game_name}.rtc      ← RTC real-time clock data (binary)
```

- `{core_name}` = core library name from `retro_get_system_info()->library_name`
- `{game_name}` = ROM filename stem (no extension), e.g., "mario_sfc" for "Mario.sfc"
- All files are raw binary blobs — no headers, compression, or encryption

## Components

### 1. FFI Bindings (`lib.rs`)

Add bindings for three libretro functions:

```rust
// Size needed to serialize the state
type RetroSerializeSizeFn = unsafe extern "C" fn() -> usize;

// Serialize internal state into buffer (returns true on success)
type RetroSerializeFn = unsafe extern "C" fn(data: *mut c_void, len: usize) -> bool;

// Unserialize state from buffer (returns true on success)  
type RetroUnserializeFn = unsafe extern "C" fn(data: *const c_void, len: usize) -> bool;
```

### 2. Core Methods (`lib.rs`)

Add to `Core` struct:

```rust
impl Core {
    /// Get the serialized state as a Vec<u8>. Returns error if core doesn't support serialization.
    pub fn save_state(&self) -> Result<Vec<u8>, CoreError>;
    
    /// Load a saved state from bytes. Returns error if unserialize fails.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), CoreError>;
}
```

Implementation:
- `save_state()`: call `retro_serialize_size()` to get buffer size → allocate Vec → call `retro_serialize(buf, size)`
- `load_state()`: call `retro_unserialize(data.as_ptr(), data.len())`

### 3. SRAM Module (`sram.rs` — new file)

```rust
pub struct SramManager;

impl SramManager {
    /// Get the save state/SRAM directory path for a game
    pub fn get_save_dir(system_dir: &Path, core_name: &str) -> PathBuf
    
    /// Build full path to a state file
    pub fn state_path(save_dir: &Path, game_name: &str) -> PathBuf
    
    /// Save SRAM data from the core into {game_name}.sav
    pub fn save(game_name: &str, save_dir: &Path) -> Result<(), SramError>;
    
    /// Load SRAM data from {game_name}.sav into the core
    pub fn load(game_name: &str, save_dir: &Path) -> Result<(), SramError>;
}
```

Memory access via libretro FFI:
- `retro_get_memory_data(RETRO_MEMORY_SAVE_RAM)` → pointer to SRAM
- `retro_get_memory_data(RETRO_MEMORY_RTC)` → pointer to RTC data
- `retro_get_memory_size(id)` → size in bytes

### 4. Keyboard Detection (`gui.rs`)

Add F2/F4 key detection to the existing input handling:

```rust
impl Gui {
    /// Check for save/load state keys. Returns SaveLoadAction if triggered.
    pub fn check_save_load_keys(&mut self, input: &InputReader) -> Option<SaveLoadAction>;
}

enum SaveLoadAction {
    Save,
    Load,
}
```

- Edge-triggered using `was_key_just_pressed()` pattern (same as ESC toggle)
- Map F2/F4 to evdev keycodes: `KEY_F2` = 60, `KEY_F4` = 62
- On trigger → set a flag that main.rs acts on

### 5. Flash Message (`gui.rs`)

Add brief visual feedback when save/load occurs:

```rust
impl Gui {
    pub fn show_flash_message(&mut self, message: &str);
}
```

Display centered at top of screen for ~2 seconds (120 frames at 60fps), then fade out. Same font rendering as existing menu overlay.

## Integration Points

### In `Core::init()` — detect serialization support

After core init, optionally call `retro_serialize_size()`. If it returns 0 or the function pointer is null, the core doesn't support save states. Log this at init time.

### After `Core::load_game()` — auto-load SRAM

```rust
// In main.rs after load_game():
SramManager::load(&rom_name, &save_dir).ok(); // ignore errors silently
```

### Before `Core::unload()` / program exit — auto-save SRAM + state

```rust
// In cleanup before core.unload():
SramManager::save(&rom_name, &save_dir).ok();  // save SRAM on exit
```

Note: We do NOT automatically save full states on exit (would overwrite user's manual save). Only SRAM auto-saves. Full state is only saved/loaded via F2/F4.

### In main loop — handle save/load keys

```rust
while RUNNING.load() {
    // ... existing GUI input handling ...
    
    // Check for save/load keys (works regardless of menu open)
    if let Some(action) = gui.check_save_load_keys(&input) {
        match action {
            SaveLoadAction::Save => {
                if let Ok(state_data) = core.save_state() {
                    std::fs::write(state_path, &state_data).ok();
                    gui.show_flash_message("State Saved");
                } else {
                    gui.show_flash_message("Save Failed");
                }
            },
            SaveLoadAction::Load => {
                if let Ok(state_data) = std::fs::read(&state_path) {
                    if core.load_state(&state_data).is_ok() {
                        gui.show_flash_message("State Loaded");
                    } else {
                        gui.show_flash_message("Load Failed");
                    }
                }
                // If no state file exists: silently ignore, no flash message
            },
        }
    }
    
    // ... rest of main loop ...
}
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Core doesn't support serialization | Log warning at init; F2/F4 produce "Save/Load Failed" message |
| No state file exists + F4 pressed | Silently ignore, no flash message |
| SRAM size is 0 (no save RAM) | Skip SRAM save/load silently |
| File write fails (disk full, permissions) | Show "Save Failed" flash message |
| State file corrupted / wrong core | `retro_unserialize` returns false → show "Load Failed" message |

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `rustsdlretro-core/src/lib.rs` | Modify | Add FFI bindings for serialize/unserialize, add `save_state()` and `load_state()` methods to Core |
| `rustsdlretro-core/src/sram.rs` | **Create** | SRAM manager: get_memory_data/size FFI, save/load to disk |
| `rustsdlretro-core/src/gui.rs` | Modify | Add `check_save_load_keys()`, `SaveLoadAction` enum, `show_flash_message()` |
| `rustsdlretro-frontend/src/main.rs` | Modify | Integrate save/load key handling into main loop, auto-load SRAM after load_game, auto-save SRAM before exit |

## Dependencies

No new crates needed. All operations use:
- Existing libc FFI for libretro calls
- `std::fs::{read, write}` for file I/O
- `std::path::PathBuf` for path construction
- `std::os::unix::ffi::OsStrExt` (already used)

## Implementation Status

**Completed** — All phases implemented and committed.

| Phase | Component | Status |
|-------|-----------|--------|
| 1 | FFI bindings + `save_state()`/`load_state()` in `lib.rs` | ✅ Done |
| 2 | SRAM module (`sram.rs`) | ✅ Done |
| 3 | Key detection + flash messages in `gui.rs` | ✅ Done |
| 4 | Main loop integration in `main.rs` | ✅ Done |

### Deviations from Design

- **SRAM Module**: Uses standalone functions instead of a struct (`SramManager`). Functions: `ensure_save_dir()`, `state_path()`, `sram_path()`, `rtc_path()`, `save_sram()`, `load_sram()`.
- **Core Name**: Stored in `Core` struct from `retro_get_system_info()->library_name`, exposed via `get_core_name()`. Also set on GUI for menu rendering.
- **Minifb F2/F4**: Dedicated tracking fields (`f2_just_pressed`, `f4_just_pressed`, `f2_held`, `f4_held`) in `InputReader` since minifb doesn't use evdev keycodes. Edge detection compares current frame against previous-frame held state.
- **SRAM auto-save**: Saves on program exit before `core.unload()` (not just before unload), ensuring data is written while the core is still valid.

## Testing Plan

1. Load a ROM → F2 to save → reset game → F4 to restore (verify exact state match)
2. Play for a while → exit program → reload → verify SRAM auto-loaded (in-game saves still present)
3. Test with multiple cores: snes9x2010, FCEUmm, mGBA (all support serialization + SRAM)
4. Verify state files are not cross-compatible between different cores
