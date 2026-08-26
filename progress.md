# Progress: Save State + SRAM Persistence

## Session Log

### Session — Save States Implementation
- [x] Read design doc `doc/save-state-design.md`
- [x] Phase 1: Added FFI function types for serialization in `lib.rs`:
  - `RetroSerializeSizeFn`, `RetroSerializeFn`, `RetroUnserializeFn`
  - `Core::save_state()` — allocates buffer, calls retro_serialize, returns Vec<u8>
  - `Core::load_state()` — calls retro_unserialize from bytes
- [x] Phase 2: Created `rustsdlretro-core/src/sram.rs`:
  - Constants: `RETRO_MEMORY_SAVE_RAM`, `RETRO_MEMORY_RTC`
  - Path helpers: `get_save_dir()`, `state_path()`, `sram_path()`, `rtc_path()`
  - `ensure_save_dir()` — creates `{system_dir}/saves/{core_name}/`
  - `save_sram()` — reads SRAM/RTC via retro_get_memory_data, writes to disk
  - `load_sram()` — reads from disk, copies into core memory via retro_get_memory_data
  - Uses dlsym for memory functions (not in bindgen allowlist)
- [x] Phase 3: Updated `gui.rs`:
  - Added `SaveLoadAction` enum (Save, Load)
  - Added `flash_message: Option<(String, u64)>` field to Gui struct
  - Added `check_save_load_keys()` — detects F2 (save)/F4 (load) via evdev keycodes 60/62
  - Added `show_flash_message()` and `flash_is_visible()` methods
  - Flash message renders centered at top of screen, ~2 second fade-out
  - Fixed render method variable ordering for flash message rendering
- [x] Phase 4: Updated `rustsdlretro-frontend/src/main.rs`:
  - Auto-create save directory after ROM load
  - Auto-load SRAM after ROM load (silent on failure)
  - Main loop checks F2/F4 keys before each frame, saves/loads state files
  - Flash messages shown for Save/Load success/failure
  - Auto-save SRAM on program exit (before core.unload())
- [x] Build passes with no errors

## Files Created/Modified

| File | Action | Description |
|------|--------|-------------|
| `rustsdlretro-core/src/lib.rs` | Modified | Added serialization FFI types + save_state/load_state methods |
| `rustsdlretro-core/src/sram.rs` | **Created** | SRAM/RTC persistence module (150 lines) |
| `rustsdlretro-core/src/gui.rs` | Modified | Save/load key detection + flash message system (+60 lines) |
| `rustsdlretro-frontend/src/main.rs` | Modified | Main loop integration, auto-save/load SRAM (+80 lines) |

## Build Status
✅ Compiles clean (cargo build succeeds)

### Fix: Log format strings not expanded (%s, %d, etc.)
- Bug: `retro_log_printf_t` is a variadic C function (`level, fmt, ...`) but our Rust callback only received 2 params → format args were dropped
- Fix: Added `log_helper.c` — C shim that receives va_list, uses vsnprintf to expand format string, then calls non-variadic Rust handler with formatted message
- Uses `dlsym(RTLD_NOW)` at env-callback registration time to get the C wrapper's address
- Added `cc = "1"` build dependency for compiling log_helper.c

### Fix: F2/F4 not working in minifb mode
- Added `f2_just_pressed`/`f4_just_pressed` tracking fields to InputReader (minifb only)
- Updated `poll_with_video()` to detect F2/F4 edge transitions via `Key::F2` / `Key::F4`
- Changed `check_save_load_keys()` in gui.rs to use cfg-gated approach:
  - minifb: uses `was_f_key_just_pressed(2/4)`
  - fbdev: uses `was_key_just_pressed(60/62)`

### Fix: F2/F4 hang (deadlock)
- The first fix had a bug: `poll_with_video` locked `f2_just_pressed` twice without releasing, causing deadlock
- Fixed by using single lock scope per key with `let mut flag = self.f2_just_pressed.lock().unwrap()`

### Fix: F2/F4 firing every frame (not edge-triggered)
- Bug 1: flag was cleared immediately in `poll_with_video`, so next frame saw "was not pressed + is down" again → fire again
- Bug 2: even with persistent flag, while key physically held, `was_f_key_just_pressed()` clears it, then next poll sees "flag=false + f2_down=true"
- Fixed: track `f2_held`/`f4_held` (previous-frame state) like evdev does with `prev_state`; only set just_pressed on rising edge

### Fix: Save directory used wrong core name
- Bug: `gui.core_name` defaulted to "RetroCore" and was never set from the libretro core
- Fix: Store core name in `Core` struct during init() (from `retro_get_system_info`)
- Now uses actual core name (e.g., "Beetle PSX", "snes9x2010") for save directory path

## Pending Testing
- [ ] Test F2 save + game reset + F4 load (verify exact state match)
- [ ] Test auto-SRAM persistence across program restarts
- [ ] Test with snes9x2010 core
- [ ] Test with FCEUmm core
- [ ] Test with mGBA core
- [ ] Test F4 with no existing state file (silent ignore)
- [ ] Test corrupted state file handling
