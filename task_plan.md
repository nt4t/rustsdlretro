# Task Plan: ZIP ROM File Support

## Goal
Enable rustsdlretro to load `.zip` archives containing game ROM files (e.g., `game.zip` with `game.sfc`, `mario.nes`, etc.) — a very common format for SNES, NES, GBA games.

## Current State
- `Core::load_game()` in `rustsdlretro-core/src/lib.rs` reads the ROM file directly via `std::fs::read(path)` and passes it as an in-memory buffer (`data`, `size`)
- Cores that set `need_fullpath=true` get a path string, but currently only works for real files on disk
- No ZIP support exists anywhere in the codebase

## Phases

### Phase 1: Design & Research (findings.md)
**Status:** COMPLETE

Research and document approach decisions.

#### Key Questions to Answer
1. How do libretro cores handle `need_fullpath` with virtual paths?
2. Which Rust ZIP library is best (zip crate vs miniz_oxide)?
3. Do we need to support nested ZIPs or just single-level?
4. Should we extract to a temp file or keep in memory?

#### Approach Options Considered

**Option A: Extract to temp file + pass path**
- Pros: Works for `need_fullpath=true` cores, simple
- Cons: File I/O overhead, cleanup complexity, security concerns with temp files

**Option B: In-memory extraction + virtual path (current approach)**
- Pros: No disk I/O, clean abstraction
- Cons: Won't work for `need_fullpath=true` cores without special handling

**Option C: Hybrid — detect need_fullpath, choose strategy**
- Extract to temp file if core needs fullpath, otherwise keep in memory
- Most compatible approach

#### Recommended Approach: Option C (Hybrid)
1. Detect if the ROM path ends with `.zip`
2. If `need_fullpath=false`: extract ROM data from ZIP into a `Vec<u8>`, pass as buffer (same as current behavior)
3. If `need_fullpath=true`: extract to a temp file, pass the temp file path
4. Use RAII guard to clean up temp files

### Phase 2: Add Dependency & Create Module
**Status:** COMPLETE

- [x] Added `zip` crate dependency (with `deflate` feature) to `rustsdlretro-core/Cargo.toml`
- [x] Created `rustsdlretro-core/src/zip_rom.rs` module
- [x] Implemented ZIP scanning and ROM selection logic
- [x] Implemented `extract_zip_to_memory()` for in-memory cores
- [x] Implemented `extract_zip_to_temp()` with RAII cleanup for fullpath cores

### Phase 3: Integrate into Core
**Status:** COMPLETE

- [x] Modified `Core::load_game()` to detect and handle `.zip` files
- [x] Added `load_game_from_zip()` helper method with hybrid strategy
- [x] Temp file cleanup via RAII `TempFileGuard` (Drop trait)
- [x] Synthetic path used for in-memory mode (`zip://archive.zip/rom_file`)

### Phase 4: Frontend Integration & UX
**Status:** COMPLETE

- [x] Updated frontend main.rs to use `zip_rom::get_zip_rom_name()` for display names
- [x] Updated README.md: moved "ZIP ROM loading" from Pending → Completed
- [x] ZIP usage documented with example command

### Phase 5: Testing
**Status:** PENDING

- [ ] Test with SNES zip files (need_fullpath varies by core)
- [ ] Test with NES zip files
- [ ] Test with GBA zip files
- [ ] Verify temp file cleanup on normal exit and crash
- [ ] Test with corrupted ZIP files (graceful error handling)

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `rustsdlretro-core/Cargo.toml` | Modify | Add `zip` dependency |
| `rustsdlretro-core/src/zip_rom.rs` | **Create** | ZIP parsing and extraction module |
| `rustsdlretro-core/src/lib.rs` | Modify | Integrate ZIP handling into `load_game()` |
| `rustsdlretro-frontend/src/main.rs` | Modify | Update ROM name display for ZIP paths |
| `README.md` | Modify | Document ZIP support in features list |

## Decisions Log

| Decision | Value | Reason |
|----------|-------|--------|
| ZIP library | `zip` crate v2.x with `deflate` feature | Most mature, actively maintained |
| Extraction strategy | Hybrid (temp file for need_fullpath, buffer otherwise) | Maximum compatibility with libretro cores |
| Temp file location | OS temp directory via `std::env::temp_dir()` | Standard practice, auto-cleaned by OS eventually |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| — | — | — |
