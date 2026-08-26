# Task Plan: Save State + SRAM Persistence

## Goal
Implement full save state support (F2/F4 keyboard shortcuts) and automatic SRAM persistence for rustsdlretro.

## Design Reference
Full design doc: `doc/save-state-design.md`

## Trigger Mechanism
| Key | Action | Context |
|-----|--------|---------|
| F2  | Save current state | Anytime (gameplay or menu) |
| F4  | Load last saved state | Anytime (gameplay or menu) |

## File Layout
```
~/.config/rustsdlretro/saves/{core_name}/{game_name}.{state|sav|rtc}
```

---

### Phase 1: FFI Bindings + Core Methods (`lib.rs`)
**Status:** COMPLETE ✅

Add libretro serialization FFI bindings and `Core` methods.

- [x] Add type aliases for `RetroSerializeSizeFn`, `RetroSerializeFn`, `RetroUnserializeFn`
- [x] Add helper methods in `Core`:
  - `save_state(&self) -> Result<Vec<u8>, CoreError>` — serialize to buffer
  - `load_state(&mut self, data: &[u8]) -> Result<(), CoreError>` — deserialize from buffer
- [x] Store core name from `retro_get_system_info()` during init

### Phase 2: SRAM Module (`sram.rs` — new file)
**Status:** COMPLETE ✅

Create dedicated module for SRAM/RTC persistence.

- [x] Create `rustsdlretro-core/src/sram.rs`
- [x] Add FFI types for `retro_get_memory_data`, `retro_get_memory_size` (manual dlsym)
- [x] Implement standalone functions:
  - `ensure_save_dir(system_dir, core_name) -> PathBuf`
  - `state_path(save_dir, game_name) -> PathBuf`
  - `sram_path()`, `rtc_path()`
  - `save_sram(handle, game_name, save_dir)` — SRAM → disk
  - `load_sram(handle, game_name, save_dir)` — disk → SRAM
- [x] Handle RETRO_MEMORY_SAVE_RAM and RETRO_MEMORY_RTC

### Phase 3: Keyboard Detection + Flash Message (`gui.rs`)
**Status:** COMPLETE ✅

Add F2/F4 key detection to existing GUI system.

- [x] Add `SaveLoadAction` enum (Save, Load)
- [x] Add `check_save_load_keys(&mut self, input: &InputReader) -> Option<SaveLoadAction>`
  - minifb mode: uses `was_f_key_just_pressed(2/4)` with dedicated tracking
  - fbdev mode: uses `was_key_just_pressed(60/62)` evdev keycodes
- [x] Track just-pressed state for edge-triggered behavior
- [x] Add `flash_message: Option<(String, u64)>` field to Gui struct (with timestamp)
- [x] Add `show_flash_message(&mut self, msg: &str)` method
- [x] Render flash message in `Gui::render()` — centered top, ~2 seconds with fade

### Phase 4: Frontend Integration (`main.rs`)
**Status:** COMPLETE ✅

Wire everything together in the main loop.

- [x] Create save directory at startup using actual core name from `retro_get_system_info()`
- [ ] Set core name on GUI for menu rendering
- [x] After `core.load_game()`: auto-load SRAM (silent on failure)
- [x] In main loop: check save/load keys, act on results with flash messages
- [x] Before program exit: auto-save SRAM (silent on failure)

### Phase 5: Testing
**Status:** IN PROGRESS

- [x] F2/F4 edge detection works correctly (fires once per press-release cycle)
- [x] Save path uses correct core name from libretro info
- [ ] Test F2 save + game reset + F4 load (verify exact state match) — pending user testing
- [ ] Test auto-SRAM persistence across program restarts — pending user testing
- [ ] Test with snes9x2010 core (SRAM + RTC support)
- [ ] Test with FCEUmm core (NES SRAM)
- [ ] Test with mGBA core (GBA SRAM/Flash)

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `rustsdlretro-core/src/lib.rs` | Modify | FFI bindings + save_state/load_state methods |
| `rustsdlretro-core/src/sram.rs` | **Create** | SRAM manager module |
| `rustsdlretro-core/src/gui.rs` | Modify | Save/load key detection + flash message |
| `rustsdlretro-frontend/src/main.rs` | Modify | Main loop integration, auto-save/load SRAM |

## Dependencies
No new crates needed. All std::fs and existing libc FFI.
