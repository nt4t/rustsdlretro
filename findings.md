# Findings: ZIP ROM Support Research

## Current ROM Loading Flow

```
main.rs: load_game(Path::new(rom_path))
  → lib.rs: Core::load_game(path)
    → std::fs::read(path)         // reads entire file into Vec<u8>
    → retro_game_info { path, data, size }  // passes to core
```

## Key Discovery: need_fullpath Behavior

From `lib.rs` line 529-530:
```rust
self.need_fullpath = info.need_fullpath;
eprintln!("Core: {} need_fullpath={}", ...);
```

The `retro_system_info::need_fullpath` field indicates whether the core requires a real file path.

### How retro_load_game works with zip files in other frontends

Looking at reference implementations (RetroArch, RetroPlayer):
- Cores that DON'T need fullpath: receive ROM data as buffer → ZIP extraction to memory works fine
- Cores that DO need fullpath: require a real file path on disk → must extract to temp file

### Common ZIP structures for game archives
```
game.zip/
  ├── game.sfc        (SNES)
  ├── game.smc        (SNES)
  ├── game.nes        (NES)
  ├── game.gb         (Game Boy)
  ├── game.gba        (GBA)
  └── README.txt      (optional metadata)
```

Some zips may contain:
- Single ROM file (most common) — straightforward extraction
- Multiple files (rare, ambiguous which is the game) — need heuristic to pick best match
- Nested directories (very rare) — unlikely needed for typical retro gaming

## ZIP Library Evaluation

### Option 1: `zip` crate (https://crates.io/crates/zip)
- Version: 2.x series
- Features: `deflate`, `bzip2`, `zstd` compression support
- Pros: Mature, well-tested, good API, supports reading from memory (`Cursor<Vec<u8>>`)
- Cons: Slightly heavier dependency

### Option 2: `miniz_oxide` + manual ZIP parsing
- Pros: Lightweight, no external deps beyond what's already used
- Cons: Manual ZIP format parsing is error-prone, reinventing the wheel

### Decision: Use `zip` crate v2.x with `deflate` feature
This covers 95%+ of retro game ZIP files which use deflate compression.
bzip2 support can be added later if needed.

## Temp File Strategy for need_fullpath Cores

For cores requiring full path, we extract to a temp file:
- Path: `{temp_dir}/rustsdlretro-{uuid}.ext` where ext matches the ROM extension
- Use `std::fs::File::create_temp()` or similar pattern
- RAII guard ensures cleanup on drop
- Fallback: if temp file extraction fails, log warning and try in-memory mode

## File Extension Detection

When extracting from ZIP, we need to determine the correct ROM extension:
1. Look for common extensions: `.sfc`, `.smc`, `.nes`, `.gb`, `.gba`, `.md`, `.sms`
2. Pick the first match found in the ZIP
3. If multiple matches exist (rare), pick the largest file or first one

## Error Handling Requirements

- Graceful error if ZIP is corrupted
- Clear error message: "Not a valid ZIP file" vs "No ROM found in archive"
- Don't crash frontend on bad ZIP — return Result::Err with descriptive message
