# Progress: ZIP ROM Support

## Session Log

### Session 1 — Planning & Research (Current)
- [x] Read README.md for project overview
- [x] Analyzed current `Core::load_game()` implementation in `lib.rs`
- [x] Identified that ROM is read via `std::fs::read(path)` into memory buffer
- [x] Understood `need_fullpath` flag from libretro core info
- [x] Evaluated ZIP library options (zip crate vs miniz_oxide)
- [x] Determined hybrid approach: temp file for fullpath cores, in-memory for others
- [x] Created task_plan.md with 5 phases
- [x] Created findings.md with research results

### Session 2 — Implementation
- [x] Phase 2: Added `zip` crate v2 dependency to Cargo.toml (with deflate feature)
- [x] Phase 2: Created `rustsdlretro-core/src/zip_rom.rs` module with:
  - `is_zip()` — detect ZIP files by extension or magic bytes
  - `find_rom_entry()` — scan ZIP for best ROM file (by extension, largest if multiple)
  - `extract_zip_to_memory()` — extract to Vec<u8> for in-memory cores
  - `extract_zip_to_temp()` — extract to temp file with RAII cleanup guard
  - `get_zip_rom_name()` — display-friendly name from ZIP path
- [x] Phase 3: Modified `Core::load_game()` to detect and handle ZIP files
- [x] Phase 3: Added `load_game_from_zip()` helper method in Core impl
- [x] Phase 4: Updated frontend main.rs to use zip_rom for ROM name display
- [x] Phase 4: Updated README.md documentation (moved ZIP from Pending → Completed)
- [x] Build passes, unit tests pass

## What's Done
- All implementation phases complete (2, 3, 4)
- Unit test `test_is_zip_by_extension` passes
- Clean release build
- Documentation updated

## Testing Results
- [x] **Genesis Plus GX** + MUSHA ZIP — SUCCESS
  - need_fullpath=true handled via temp file extraction
  - Game loads at 256x192 @ 59.92 FPS, audio at 44100 Hz
  - Temp file: `/tmp/rustsdlretro_MUSHA_....gen`
- [ ] SNES core ZIP test (pending)
- [ ] NES core ZIP test (pending)
- [ ] GBA core ZIP test (pending)  
- [ ] Corrupted ZIP error handling
- [ ] Temp file cleanup verification
