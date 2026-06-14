# Core Options GUI Plan

## Overview
Add a simple overlay GUI for browsing and modifying libretro core options, rendered on the framebuffer using the embedded bitmap font.

## Architecture

```
sdlretro-core/src/
├── gui.rs              # New: GUI framework + menu rendering
├── core_options.rs     # New: libretro core options FFI bindings + loading
├── font.rs             # New: bitmap font renderer
├── lib.rs              # Existing: Core, environment callback updates
├── video.rs            # Existing: FbdevVideo (unchanged)
├── input.rs            # Existing: InputReader (+ menu key mapping)
└── audio.rs            # Existing: AudioDriver (unchanged)
```

## Phase 1: Font Rendering

### 1.1 `font.rs` - Bitmap Font Renderer
- Include `bmfont.inl` as a module (C→Rust conversion in build step, or inline as Rust)
- `Font` struct with `small` and `big` font data
- `Font::draw_char(fb: &mut FbdevVideo, x: y: u32, ch: char, color: u32)` - draws a single glyph
- `Font::measure_text(text: &str) -> (width, height)` - returns dimensions for text layout
- `Font::draw_text(fb: &mut FbdevVideo, x: y: u32, text: &str, color: u32)` - draws a string

### 1.2 C→Rust Font Data Conversion
- Convert `bmfont.inl` to Rust at build time (`build.rs`)
- Output: `gen/fonts.rs` with `FONT_SMALL_*` and `FONT_BIG_*` constants
- Glyph data stays as `&[u8]` slices, glyph metadata as arrays

## Phase 2: Core Options FFI

### 2.1 `core_options.rs` - libretro Core Options Bindings
- FFI types from `libretro.h`:
  - `retro_core_option_version`
  - `retro_core_option_value`
  - `retro_core_option_definition` (v1) / `retro_core_option_v2_definition` (v2)
  - `retro_core_option_display`
- FFI function pointers:
  - `RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION` (key 52)
  - `RETRO_ENVIRONMENT_SET_CORE_OPTIONS` (key 53)
  - `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2` (key 67)
  - `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY` (key 55)
- `CoreOptions` struct:
  - `load(core_handle) -> Result<Vec<CoreOptionDefinition>>` - retrieves options from core
  - `get_value(key) -> Option<String>` - gets current option value
  - `set_value(key, value) -> bool` - sets option value
  - `supports_v2() -> bool` - checks if core supports v2 API

### 2.2 Environment Callback Updates
- Extend `log_environment_cb` in `lib.rs` to handle:
  - `GET_CORE_OPTIONS_VERSION` → returns version number (1 or 2)
  - `SET_CORE_OPTIONS` / `SET_CORE_OPTIONS_V2` → stores option definitions
  - `SET_CORE_OPTIONS_DISPLAY` → shows/hides option in menu

## Phase 3: GUI Framework

### 3.1 `gui.rs` - Menu System
- `GuiState` enum: `Playing`, `MenuOpen`, `Settings`
- `Menu` struct:
  - `title: String` - menu header (e.g., "Core Options")
  - `items: Vec<MenuItem>` - list of options
  - `selected: usize` - cursor position
  - `scroll_offset: usize` - vertical scroll for long lists
- `MenuItem` enum:
  - `Text(label: String)` - static label
  - `OptionItem { key: String, label: String, values: [(String, String), ..], current: usize }`
  - `Separator` - visual divider
  - `Action { label: String, callback: Box<dyn Fn()> }` - button
- `Gui::new(fb_width, fb_height, font) -> Self`
- `Gui::render(&self, fb: &mut FbdevVideo)` - draws the menu overlay
- `Gui::handle_input(&mut self, input: &InputReader) -> GuiState`

### 3.2 Rendering
- Semi-transparent dark overlay behind menu (drawn via framebuffer blit)
- Menu box with border (drawn using font characters: `+`, `-`, `|`)
- Highlighted item with different color (e.g., yellow on dark bg)
- Arrow indicators for scrollable lists (`^`, `v`)
- "Press ESC to exit" footer text

### 3.3 Color Palette
- Background: `0x000000` (black, alpha via overlay opacity)
- Text: `0xFFFFFF` (white)
- Highlight: `0xFFFF00` (yellow)
- Border: `0x888888` (gray)
- Footer: `0x888888` (gray)

## Phase 4: Integration

### 4.1 Input Handling
- Add menu-specific key mapping to `InputReader`:
  - `ESC` (keycode 1) → toggle menu open/close
  - Arrow keys → navigate menu items
  - `ENTER` (keycode 28) → select/confirm
  - `SPACE` (keycode 57) → cycle option value
  - `TAB` (keycode 15) → change option value (+1)
  - `LSHIFT` (keycode 42) / `RSHIFT` (keycode 54) → change option value (-1/+1)

### 4.2 Main Loop Changes
```rust
// In main.rs main loop:
let gui_state = gui.handle_input(&input);

match gui_state {
    GuiState::Playing => {
        // Normal game loop
        core.run();
    }
    GuiState::MenuOpen | GuiState::Settings => {
        // Pause core, render menu overlay on top of last frame
        gui.render(&mut video);
    }
}
```

### 4.3 Core Pause/Resume
- When menu opens: set a `paused` flag, skip `core.run()` calls
- When menu closes: resume normal game loop
- Audio ring buffer handles pause naturally (just stops receiving new data)

## Phase 5: Polish

### 5.1 Core Detection
- Show core name in menu header (from `retro_get_system_info().library_name`)
- Show ROM name in footer
- Show FPS overlay when menu is closed (existing FPS counter)

### 5.2 Persistence
- Save option changes to a per-core config file:
  - Path: `~/.config/rustsdlretro/<core_name>.cfg`
  - Format: simple `key=value` lines
- Load config on core init, apply options before `retro_load_game`

### 5.3 Visual Polish
- Smooth scroll animation (optional)
- Transition effect when opening/closing menu (fade)
- "Saving..." indicator when writing config

## Implementation Order

1. **Font renderer** (`font.rs` + build.rs conversion)
2. **Core options FFI** (`core_options.rs` + env callback updates)
3. **Basic GUI framework** (`gui.rs` - menu rendering + navigation)
4. **Integration** (main loop changes, input mapping, pause/resume)
5. **Persistence** (config file save/load)

## Key Constraints
- Framebuffer only (no SDL/OpenGL) - all rendering via pixel manipulation
- Limited RAM on Pi - keep font data and menu state small
- No dynamic allocation in hot path (menu rendering runs every frame)
- Must not block the audio playback thread
- Menu key input uses same `/dev/input/event0` as game input
