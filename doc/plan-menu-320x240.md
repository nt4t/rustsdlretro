# Plan: Optimize Menu to 320x240 Resolution

## Problem

The current menu overlay is designed for larger framebuffers (400px wide, centered).
On 320x240 framebuffer, the menu is too large and elements overflow outside the visible area.

Current menu dimensions:
- Width: 400px (fixed)
- Position: centered `(fb_width - 400) / 2`
- Height: `fb_height - 100`

On 320x240:
- `bg_x1 = (320 - 400) / 2 = -40` (menu starts off-screen left!)
- `bg_x2 = -40 + 400 = 360` (menu extends past right edge)
- Menu elements like arrow indicators at `item_x - 8` are even further outside

## Goals

1. Menu fits completely within 320x240 framebuffer
2. Menu scales proportionally for larger framebuffers (480x272, 800x480, etc.)
3. All menu elements (text, arrows, borders) stay within bounds
4. No clipping or overflow of any UI elements

## Design

### 1. Define menu dimensions

Target resolution: 320x240

Menu dimensions for 320x240:
- Width: 300px (leaves 10px margin on each side)
- Height: 180px (leaves 30px margin top, 30px margin bottom)
- Position: centered horizontally, 30px from top

For larger framebuffers, scale proportionally:
- Menu width = `min(300, fb_width - 20)`
- Menu height = `min(180, fb_height - 60)`

### 2. Update menu layout calculations

Current layout:
```
bg_x1 = (w - 400) / 2
bg_y1 = 60
bg_x2 = bg_x1 + 400
bg_y2 = h - 40
```

New layout for 320x240:
```
menu_width = min(300, fb_width - 20)
menu_height = min(180, fb_height - 60)
bg_x1 = (w - menu_width) / 2
bg_y1 = (h - menu_height) / 2 - 20
bg_x2 = bg_x1 + menu_width
bg_y2 = bg_y1 + menu_height
```

### 3. Adjust element positions

Header text: `bg_x1 + 10`, `bg_y1 + 10`
Item start: `bg_y1 + 25`
Item x: `bg_x1 + 10`
Arrow indicator: `item_x - 6` (inside menu border)
Value text: right-aligned at `bg_x2 - 10`
Footer: `bg_y2 + 8`
Scroll indicators: `bg_x2 - 10`

### 4. Adjust visible item count

Current: `visible_count = (fb_height - 40) / 12`
New: `visible_count = (menu_height - 35) / 12` (reserve space for header/footer)

For 320x240: `visible_count = (180 - 35) / 12 = 12` items visible

### 5. Handle fallback overlay

When no core options are available, draw fallback menu:
- Same dimensions as main menu
- Centered on screen
- Smaller text for "Core options not available"

## Implementation Steps

### Step 1: Update `render()` in `gui.rs`

Replace hardcoded 400px width with calculated menu width:
```rust
let menu_width = (fb_width as i32 - 20).max(200); // min 200px
let menu_height = (fb_height as i32 - 60).max(120); // min 120px
let bg_x1 = (fb_width as i32 - menu_width) / 2;
let bg_y1 = (fb_height as i32 - menu_height) / 2 - 20;
let bg_x2 = bg_x1 + menu_width;
let bg_y2 = bg_y1 + menu_height;
```

### Step 2: Update `visible_count()` in `menu.rs`

```rust
pub fn visible_count(&self, fb_height: u32) -> usize {
    let menu_height = (fb_height as i32 - 60).max(120);
    let available_height = menu_height - 35;
    let item_height = 12;
    (available_height / item_height) as usize
}
```

### Step 3: Update arrow indicator position

Change from `item_x - 8` to `item_x - 6` to stay inside menu border.

### Step 4: Update value text positioning

Ensure value text doesn't overlap with arrow indicator:
```rust
let value_x = bg_x2 - 10 - value_text.len() as i32 * 6;
```

### Step 5: Update footer positioning

```rust
let footer_y = bg_y2 + 8;
```

### Step 6: Test on 320x240 and larger resolutions

Verify all elements are visible and no overflow occurs.

## Files to Modify

| File | Changes |
|------|---------|
| `sdlretro-core/src/gui.rs` | Update menu dimensions, positions, visible count |

## Risks

- **Text overlap**: Longer option labels may overlap with value text on narrow menus
- **Scroll indicators**: May overlap with value text if menu is too narrow
- **Smaller framebuffers**: If fb_width < 220, menu may be too small to be usable

## Mitigations

- Clamp minimum menu width to 200px
- Truncate long labels with ellipsis if they overlap with values
- Show "Press ESC to close" only on larger framebuffers
