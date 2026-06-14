// Bitmap font renderer for framebuffer overlay
// Font data from bmfont.inl, rendering based on fbdev_video.cpp draw_text_impl()
//
// Bit ordering: bit 0 = leftmost pixel, bit 1 = next pixel, etc.
// Shadow offset: fd.x + 1 (right), fd.y + 1 (up, since y is negative)

use std::os::raw::c_int;

// Glyph metadata (matches font_data_t C struct layout)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FontGlyph {
    pub step_width: c_int,   // total width including spacing
    pub width: c_int,        // glyph bitmap width
    pub height: c_int,       // glyph bitmap height
    pub x: c_int,            // x offset
    pub y: c_int,            // y offset (negative = above baseline)
    pub data_offset: usize,  // offset into glyph_data array
}

// Small font: 8px tall, ASCII 0x20-0x7E (96 chars)
pub const SMALL_FONT_H: c_int = 8;
pub const SMALL_FONT_DATA: &[u8] = &[
    0x03, 0x03, 0x03, 0x03, 0x03, 0x00, 0x03, 0x05, 0x05, 0x0A, 0x1F, 0x1F, 0x0A, 0x1F, 0x1F, 0x0A,
    0x0A, 0x1E, 0x0B, 0x0F, 0x1E, 0x1A, 0x0F, 0x0A, 0x13, 0x1B, 0x18, 0x0C, 0x04, 0x36, 0x32, 0x0E,
    0x1B, 0x0E, 0x2F, 0x3B, 0x3B, 0x2E, 0x01, 0x01, 0x04, 0x02, 0x03, 0x03, 0x03, 0x03, 0x02, 0x04,
    0x01, 0x02, 0x06, 0x06, 0x06, 0x06, 0x02, 0x01, 0x04, 0x15, 0x1F, 0x0E, 0x1F, 0x15, 0x04, 0x04,
    0x04, 0x1F, 0x04, 0x04, 0x02, 0x03, 0x01, 0x0F, 0x03, 0x03, 0x04, 0x04, 0x04, 0x06, 0x02, 0x03,
    0x03, 0x01, 0x0E, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x0E, 0x06, 0x07, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x0F, 0x18, 0x18, 0x0E, 0x03, 0x03, 0x1F, 0x0F, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x0F, 0x18, 0x1C,
    0x1A, 0x19, 0x1F, 0x18, 0x18, 0x0F, 0x01, 0x0F, 0x18, 0x18, 0x18, 0x0F, 0x0E, 0x03, 0x0F, 0x1B,
    0x1B, 0x1B, 0x0E, 0x1F, 0x18, 0x0C, 0x0C, 0x06, 0x06, 0x06, 0x0E, 0x1B, 0x1B, 0x0E, 0x1B, 0x1B,
    0x0E, 0x0E, 0x1B, 0x1B, 0x1B, 0x1E, 0x18, 0x0E, 0x03, 0x03, 0x00, 0x03, 0x03, 0x03, 0x03, 0x00,
    0x02, 0x03, 0x01, 0x08, 0x0C, 0x06, 0x03, 0x06, 0x0C, 0x08, 0x0F, 0x00, 0x0F, 0x01, 0x03, 0x06,
    0x0C, 0x06, 0x03, 0x01, 0x0F, 0x18, 0x0C, 0x06, 0x06, 0x00, 0x06, 0x1E, 0x33, 0x3F, 0x3B, 0x3F,
    0x03, 0x1E, 0x0E, 0x1B, 0x1B, 0x1B, 0x1F, 0x1B, 0x1B, 0x0F, 0x1B, 0x0F, 0x1B, 0x1B, 0x1B, 0x0F,
    0x1E, 0x03, 0x03, 0x03, 0x03, 0x03, 0x1E, 0x0F, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x0F, 0x1F, 0x03,
    0x0F, 0x03, 0x03, 0x03, 0x1F, 0x1F, 0x03, 0x0F, 0x03, 0x03, 0x03, 0x03, 0x0E, 0x03, 0x03, 0x1B,
    0x1B, 0x1B, 0x1E, 0x1B, 0x1B, 0x1F, 0x1B, 0x1B, 0x1B, 0x1B, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
    0x03, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x07, 0x33, 0x1B, 0x0F, 0x07, 0x0F, 0x1B, 0x33, 0x03,
    0x03, 0x03, 0x03, 0x03, 0x03, 0x0F, 0x41, 0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x31, 0x33, 0x37,
    0x3F, 0x3B, 0x33, 0x23, 0x1E, 0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0F, 0x1B, 0x1B, 0x1B, 0x0F,
    0x03, 0x03, 0x1E, 0x33, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x30, 0x0F, 0x1B, 0x1B, 0x1B, 0x0F, 0x0B,
    0x1B, 0x0E, 0x03, 0x03, 0x06, 0x0C, 0x0C, 0x07, 0x3F, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1B,
    0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x0E, 0x33, 0x33, 0x33, 0x1E, 0x1E, 0x0C, 0x0C, 0x63, 0x63, 0x6B,
    0x7F, 0x3E, 0x3E, 0x36, 0x33, 0x33, 0x1E, 0x0C, 0x1E, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C,
    0x0C, 0x0C, 0x1F, 0x18, 0x1C, 0x0E, 0x07, 0x03, 0x1F, 0x07, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
    0x07, 0x01, 0x01, 0x01, 0x03, 0x02, 0x06, 0x06, 0x04, 0x07, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x07, 0x02, 0x07, 0x05, 0x3F, 0x01, 0x02, 0x0E, 0x18, 0x1E, 0x1B, 0x1E, 0x03, 0x03, 0x0F, 0x1B,
    0x1B, 0x1B, 0x0F, 0x0E, 0x03, 0x03, 0x03, 0x0E, 0x18, 0x18, 0x1E, 0x1B, 0x1B, 0x1B, 0x1E, 0x0E,
    0x1B, 0x1F, 0x03, 0x1E, 0x06, 0x03, 0x07, 0x03, 0x03, 0x03, 0x03, 0x1E, 0x1B, 0x1B, 0x1E, 0x18,
    0x0E, 0x03, 0x03, 0x0F, 0x1B, 0x1B, 0x1B, 0x1B, 0x03, 0x00, 0x03, 0x03, 0x03, 0x03, 0x03, 0x06,
    0x00, 0x06, 0x06, 0x06, 0x06, 0x06, 0x03, 0x03, 0x03, 0x1B, 0x0F, 0x07, 0x0F, 0x1B, 0x03, 0x03,
    0x03, 0x03, 0x03, 0x03, 0x03, 0x7F, 0xDB, 0xDB, 0xDB, 0xDB, 0x0F, 0x1B, 0x1B, 0x1B, 0x1B, 0x0E,
    0x1B, 0x1B, 0x1B, 0x0E, 0x0F, 0x1B, 0x1B, 0x0F, 0x03, 0x03, 0x1E, 0x1B, 0x1B, 0x1E, 0x18, 0x18,
    0x0B, 0x0F, 0x03, 0x03, 0x03, 0x0E, 0x03, 0x0F, 0x0C, 0x07, 0x03, 0x03, 0x07, 0x03, 0x03, 0x03,
    0x06, 0x1B, 0x1B, 0x1B, 0x1B, 0x1E, 0x1B, 0x1B, 0x0E, 0x0E, 0x04, 0x63, 0x6B, 0x6B, 0x3E, 0x36,
    0x1B, 0x1B, 0x0E, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1E, 0x18, 0x0E, 0x1F, 0x0C, 0x06, 0x03, 0x1F,
    0x07, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x07, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
    0x07, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x07, 0x16, 0x0D,
];

pub const SMALL_FONT_GLYPHS: &[FontGlyph] = &[
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x00
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x01
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x02
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x03
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x04
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x05
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x06
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x07
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x08
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x09
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x0A
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x0B
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x0C
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x0D
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x0E
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x0F
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x10
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x11
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x12
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x13
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x14
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x15
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x16
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x17
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x18
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x19
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x1A
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x1B
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x1C
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x1D
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x1E
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x1F
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x20 (space)
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 0 },     // 0x21 (!)
    FontGlyph { step_width: 4, width: 3, height: 2, x: 0, y: -7, data_offset: 7 },     // 0x22 (")
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 9 },     // 0x23 (#)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 8, y: -7, data_offset: 16 },    // 0x24 ($)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 23 },    // 0x25 (%)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 30 },    // 0x26 (&)
    FontGlyph { step_width: 2, width: 1, height: 2, x: 0, y: -7, data_offset: 37 },    // 0x27 (')
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 39 },    // 0x28 (()
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 47 },    // 0x29 ())
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 55 },    // 0x2A (*)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -6, data_offset: 62 },    // 0x2B (+)
    FontGlyph { step_width: 3, width: 2, height: 3, x: 0, y: -2, data_offset: 67 },    // 0x2C (,)
    FontGlyph { step_width: 5, width: 4, height: 1, x: 0, y: -4, data_offset: 70 },    // 0x2D (-)
    FontGlyph { step_width: 3, width: 2, height: 2, x: 0, y: -2, data_offset: 74 },    // 0x2E (.)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 76 },    // 0x2F (/)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 84 },    // 0x30 (0)
    FontGlyph { step_width: 4, width: 3, height: 7, x: 0, y: -7, data_offset: 90 },    // 0x31 (1)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 97 },    // 0x32 (2)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 104 },   // 0x33 (3)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 111 },   // 0x34 (4)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 118 },   // 0x35 (5)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 125 },   // 0x36 (6)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 132 },   // 0x37 (7)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 139 },   // 0x38 (8)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 146 },   // 0x39 (9)
    FontGlyph { step_width: 3, width: 2, height: 5, x: 0, y: -5, data_offset: 153 },   // 0x3A (:)
    FontGlyph { step_width: 3, width: 2, height: 6, x: 0, y: -5, data_offset: 158 },   // 0x3B (;)
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 164 },   // 0x3C (<)
    FontGlyph { step_width: 5, width: 4, height: 3, x: 0, y: -4, data_offset: 171 },   // 0x3D (=)
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 176 },   // 0x3E (>)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 183 },   // 0x3F (?)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -6, data_offset: 190 },   // 0x40 (@)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 197 },   // 0x41 (A)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 204 },   // 0x42 (B)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 211 },   // 0x43 (C)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 218 },   // 0x44 (D)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 225 },   // 0x45 (E)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 232 },   // 0x46 (F)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 239 },   // 0x47 (G)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 246 },   // 0x48 (H)
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 253 },   // 0x49 (I)
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 260 },   // 0x4A (J)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 267 },   // 0x4B (K)
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 275 },   // 0x4C (L)
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 282 },   // 0x4D (M)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 291 },   // 0x4E (N)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 300 },   // 0x4F (O)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 309 },   // 0x50 (P)
    FontGlyph { step_width: 7, width: 6, height: 8, x: 0, y: -8, data_offset: 318 },   // 0x51 (Q)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 328 },   // 0x52 (R)
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 337 },   // 0x53 (S)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 345 },   // 0x54 (T)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 353 },   // 0x55 (U)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 360 },   // 0x56 (V)
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 369 },   // 0x57 (W)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 380 },   // 0x58 (X)
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 390 },   // 0x59 (Y)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 400 },   // 0x5A (Z)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 410 },   // 0x5B ([)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 419 },   // 0x5C (\)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 428 },   // 0x5D (])
    FontGlyph { step_width: 4, width: 3, height: 3, x: 0, y: -7, data_offset: 437 },   // 0x5E (^)
    FontGlyph { step_width: 6, width: 6, height: 1, x: 0, y: -1, data_offset: 443 },   // 0x5F (_)
    FontGlyph { step_width: 3, width: 2, height: 2, x: 0, y: -7, data_offset: 452 },   // 0x60 (`)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 458 },   // 0x61 (a)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 467 },   // 0x62 (b)
    FontGlyph { step_width: 5, width: 4, height: 5, x: 0, y: -5, data_offset: 476 },   // 0x63 (c)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 484 },   // 0x64 (d)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 493 },   // 0x65 (e)
    FontGlyph { step_width: 4, width: 3, height: 7, x: 0, y: -7, data_offset: 502 },   // 0x66 (f)
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 509 },   // 0x67 (g)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 519 },   // 0x68 (h)
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 528 },   // 0x69 (i)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 538 },   // 0x6A (j)
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 549 },   // 0x6B (k)
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 558 },   // 0x6C (l)
    FontGlyph { step_width: 9, width: 8, height: 5, x: 0, y: -5, data_offset: 568 },   // 0x6D (m)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 580 },   // 0x6E (n)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 589 },   // 0x6F (o)
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 598 },   // 0x70 (p)
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 607 },   // 0x71 (q)
    FontGlyph { step_width: 5, width: 4, height: 5, x: 0, y: -5, data_offset: 616 },   // 0x72 (r)
    FontGlyph { step_width: 5, width: 4, height: 5, x: 0, y: -5, data_offset: 624 },   // 0x73 (s)
    FontGlyph { step_width: 4, width: 3, height: 7, x: 0, y: -7, data_offset: 632 },   // 0x74 (t)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 641 },   // 0x75 (u)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 650 },   // 0x76 (v)
    FontGlyph { step_width: 8, width: 7, height: 5, x: 0, y: -5, data_offset: 659 },   // 0x77 (w)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 670 },   // 0x78 (x)
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 681 },   // 0x79 (y)
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 691 },   // 0x7A (z)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 702 },   // 0x7B ({)
    FontGlyph { step_width: 3, width: 2, height: 8, x: 0, y: -8, data_offset: 712 },   // 0x7C (|)
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 720 },   // 0x7D (})
    FontGlyph { step_width: 6, width: 5, height: 2, x: 0, y: -7, data_offset: 730 },   // 0x7E (~)
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },      // 0x7F
];

// Big font: 16px tall, ASCII 0x20-0x7E (96 chars)
pub const BIG_FONT_H: c_int = 16;
pub const BIG_FONT_DATA: &[u8] = &[
    0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F,
    0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x01, 0x05, 0x05, 0x05, 0x12, 0x12, 0x3F, 0x12,
    0x12, 0x12, 0x3F, 0x12, 0x12, 0x04, 0x04, 0x0E, 0x15, 0x05, 0x05, 0x0E, 0x14, 0x14, 0x15, 0x0E,
    0x04, 0x04, 0x02, 0x05, 0x25, 0x12, 0x08, 0x24, 0x52, 0x50, 0x20, 0x0E, 0x11, 0x01, 0x01, 0x0E,
    0x11, 0x11, 0x11, 0x1E, 0x01, 0x01, 0x01, 0x04, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x04,
    0x01, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x02, 0x01, 0x04, 0x15, 0x0E, 0x15, 0x04, 0x04, 0x04,
    0x1F, 0x04, 0x04, 0x02, 0x02, 0x01, 0x0F, 0x01, 0x04, 0x04, 0x04, 0x02, 0x02, 0x02, 0x01, 0x01,
    0x01, 0x0E, 0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x0E, 0x04, 0x06, 0x05, 0x04, 0x04, 0x04,
    0x04, 0x04, 0x1F, 0x0E, 0x11, 0x10, 0x08, 0x04, 0x02, 0x01, 0x01, 0x1F, 0x0E, 0x11, 0x10, 0x10,
    0x0C, 0x10, 0x10, 0x11, 0x0E, 0x10, 0x18, 0x14, 0x12, 0x11, 0x1F, 0x10, 0x10, 0x10, 0x1F, 0x01,
    0x01, 0x0F, 0x10, 0x10, 0x10, 0x11, 0x0E, 0x0E, 0x11, 0x01, 0x01, 0x0F, 0x11, 0x11, 0x11, 0x0E,
    0x1F, 0x10, 0x10, 0x08, 0x04, 0x02, 0x01, 0x01, 0x01, 0x0E, 0x11, 0x11, 0x11, 0x0E, 0x11, 0x11,
    0x11, 0x0E, 0x0E, 0x11, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x11, 0x0E, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02, 0x01, 0x04, 0x02, 0x01, 0x02,
    0x04, 0x0F, 0x00, 0x0F, 0x01, 0x02, 0x04, 0x02, 0x01, 0x0E, 0x11, 0x10, 0x08, 0x04, 0x04, 0x04,
    0x00, 0x04, 0x3E, 0x41, 0x59, 0x55, 0x55, 0x55, 0x39, 0x01, 0x3E, 0x0E, 0x11, 0x11, 0x11, 0x11,
    0x1F, 0x11, 0x11, 0x11, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x0E, 0x11, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x11, 0x0E, 0x0F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0F,
    0x1F, 0x01, 0x01, 0x01, 0x07, 0x01, 0x01, 0x01, 0x1F, 0x1F, 0x01, 0x01, 0x01, 0x07, 0x01, 0x01,
    0x01, 0x01, 0x0E, 0x11, 0x01, 0x01, 0x19, 0x11, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x11, 0x11, 0x1F,
    0x11, 0x11, 0x11, 0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F, 0x3C, 0x30, 0x10, 0x10,
    0x10, 0x10, 0x10, 0x11, 0x0E, 0x11, 0x11, 0x09, 0x05, 0x03, 0x05, 0x09, 0x11, 0x11, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x1F, 0x41, 0x41, 0x63, 0x55, 0x49, 0x41, 0x41, 0x41, 0x41, 0x11,
    0x11, 0x11, 0x13, 0x15, 0x19, 0x11, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x0E, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x01, 0x01, 0x0E, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x15, 0x09, 0x16, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x05, 0x09, 0x11, 0x11, 0x0E, 0x11, 0x01, 0x01,
    0x0E, 0x10, 0x10, 0x11, 0x0E, 0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04,
    0x41, 0x41, 0x49, 0x49, 0x49, 0x49, 0x49, 0x49, 0x36, 0x11, 0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F, 0x10, 0x10, 0x08, 0x04,
    0x02, 0x01, 0x01, 0x1F, 0x07, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x07, 0x01, 0x01, 0x01,
    0x02, 0x02, 0x02, 0x04, 0x04, 0x04, 0x07, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x07, 0x04,
    0x0A, 0x11, 0xFF, 0x01, 0x02, 0x0E, 0x11, 0x10, 0x1E, 0x11, 0x11, 0x1E, 0x01, 0x01, 0x0F, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x0F, 0x0E, 0x11, 0x01, 0x01, 0x01, 0x11, 0x0E, 0x10, 0x10, 0x1E, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x1E, 0x0E, 0x11, 0x11, 0x1F, 0x01, 0x11, 0x0E, 0x0C, 0x02, 0x02, 0x0F,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x1E, 0x11, 0x11, 0x11, 0x11, 0x1E, 0x10, 0x11, 0x0E, 0x01, 0x01,
    0x0F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x04, 0x00, 0x07, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F,
    0x10, 0x00, 0x1C, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x11, 0x0E, 0x01, 0x01, 0x11, 0x09, 0x05,
    0x03, 0x05, 0x09, 0x11, 0x07, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F, 0x37, 0x49, 0x49,
    0x49, 0x49, 0x41, 0x41, 0x0F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x0E, 0x0F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x1E, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x1E, 0x10, 0x10, 0x19, 0x05, 0x03, 0x01, 0x01, 0x01, 0x01, 0x0E, 0x11, 0x01, 0x0E, 0x10,
    0x11, 0x0E, 0x02, 0x0F, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0C, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04, 0x41, 0x41, 0x49, 0x49, 0x49, 0x49, 0x36, 0x11,
    0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E, 0x10, 0x11, 0x0E, 0x1F,
    0x10, 0x08, 0x04, 0x02, 0x01, 0x1F, 0x0C, 0x02, 0x02, 0x02, 0x01, 0x02, 0x02, 0x02, 0x0C, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03, 0x04, 0x04, 0x04, 0x08, 0x04, 0x04, 0x04,
    0x03, 0x26, 0x19, 0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F,
];

pub const BIG_FONT_GLYPHS: &[FontGlyph] = &[
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 0 },       // 0x00
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 9 },       // 0x01
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 18 },      // 0x02
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 27 },      // 0x03
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 36 },      // 0x04
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 45 },      // 0x05
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 54 },      // 0x06
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 63 },      // 0x07
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 72 },      // 0x08
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 81 },      // 0x09
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 90 },      // 0x0A
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 99 },      // 0x0B
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 108 },     // 0x0C
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 117 },     // 0x0D
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 126 },     // 0x0E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 135 },     // 0x0F
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 144 },     // 0x10
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 153 },     // 0x11
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 162 },     // 0x12
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 171 },     // 0x13
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 180 },     // 0x14
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 189 },     // 0x15
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 198 },     // 0x16
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 207 },     // 0x17
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 216 },     // 0x18
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 225 },     // 0x19
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 234 },     // 0x1A
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 243 },     // 0x1B
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 252 },     // 0x1C
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 261 },     // 0x1D
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 270 },     // 0x1E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 279 },     // 0x1F
    FontGlyph { step_width: 8, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },        // 0x20 (space)
    FontGlyph { step_width: 8, width: 1, height: 9, x: 3, y: -9, data_offset: 288 },    // 0x21 (!)
    FontGlyph { step_width: 8, width: 3, height: 3, x: 2, y: -9, data_offset: 297 },    // 0x22 (")
    FontGlyph { step_width: 8, width: 6, height: 9, x: 1, y: -9, data_offset: 306 },    // 0x23 (#)
    FontGlyph { step_width: 8, width: 5, height: 13, x: 1, y: -5, data_offset: 321 },   // 0x24 ($)
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 340 },    // 0x25 (%)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 357 },    // 0x26 (&)
    FontGlyph { step_width: 8, width: 1, height: 3, x: 3, y: -9, data_offset: 372 },    // 0x27 (')
    FontGlyph { step_width: 8, width: 3, height: 9, x: 3, y: -9, data_offset: 381 },    // 0x28 (()
    FontGlyph { step_width: 8, width: 3, height: 9, x: 1, y: -9, data_offset: 393 },    // 0x29 ())
    FontGlyph { step_width: 8, width: 5, height: 5, x: 1, y: -9, data_offset: 405 },    // 0x2A (*)
    FontGlyph { step_width: 8, width: 5, height: 5, x: 1, y: -7, data_offset: 415 },    // 0x2B (+)
    FontGlyph { step_width: 8, width: 2, height: 3, x: 2, y: -1, data_offset: 425 },    // 0x2C (,)
    FontGlyph { step_width: 8, width: 4, height: 1, x: 2, y: -5, data_offset: 431 },    // 0x2D (-)
    FontGlyph { step_width: 8, width: 1, height: 1, x: 3, y: -1, data_offset: 435 },    // 0x2E (.)
    FontGlyph { step_width: 8, width: 3, height: 9, x: 2, y: -9, data_offset: 438 },    // 0x2F (/)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 450 },    // 0x30 (0)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 462 },    // 0x31 (1)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 474 },    // 0x32 (2)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 486 },    // 0x33 (3)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 498 },    // 0x34 (4)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 510 },    // 0x35 (5)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 522 },    // 0x36 (6)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 534 },    // 0x37 (7)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 546 },    // 0x38 (8)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 558 },    // 0x39 (9)
    FontGlyph { step_width: 8, width: 1, height: 7, x: 3, y: -7, data_offset: 570 },    // 0x3A (:)
    FontGlyph { step_width: 8, width: 2, height: 9, x: 2, y: -7, data_offset: 581 },    // 0x3B (;)
    FontGlyph { step_width: 8, width: 3, height: 5, x: 2, y: -7, data_offset: 594 },    // 0x3C (<)
    FontGlyph { step_width: 8, width: 4, height: 3, x: 2, y: -6, data_offset: 604 },    // 0x3D (=)
    FontGlyph { step_width: 8, width: 3, height: 5, x: 2, y: -7, data_offset: 612 },    // 0x3E (>)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 622 },    // 0x3F (?)
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 636 },    // 0x40 (@)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 657 },    // 0x41 (A)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 672 },    // 0x42 (B)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 687 },    // 0x43 (C)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 702 },    // 0x44 (D)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 717 },    // 0x45 (E)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 732 },    // 0x46 (F)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 747 },    // 0x47 (G)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 762 },    // 0x48 (H)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 777 },    // 0x49 (I)
    FontGlyph { step_width: 8, width: 6, height: 9, x: 1, y: -9, data_offset: 792 },    // 0x4A (J)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 807 },    // 0x4B (K)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 825 },    // 0x4C (L)
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 843 },    // 0x4D (M)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 864 },    // 0x4E (N)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 885 },    // 0x4F (O)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 906 },    // 0x50 (P)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 927 },    // 0x51 (Q)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 948 },    // 0x52 (R)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 969 },    // 0x53 (S)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 990 },    // 0x54 (T)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1011 },   // 0x55 (U)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1032 },   // 0x56 (V)
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 1053 },   // 0x57 (W)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1076 },   // 0x58 (X)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1099 },   // 0x59 (Y)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1122 },   // 0x5A (Z)
    FontGlyph { step_width: 8, width: 3, height: 9, x: 3, y: -9, data_offset: 1145 },   // 0x5B ([)
    FontGlyph { step_width: 8, width: 3, height: 9, x: 2, y: -9, data_offset: 1160 },   // 0x5C (\)
    FontGlyph { step_width: 8, width: 3, height: 9, x: 1, y: -9, data_offset: 1175 },   // 0x5D (])
    FontGlyph { step_width: 8, width: 5, height: 3, x: 1, y: -9, data_offset: 1192 },   // 0x5E (^)
    FontGlyph { step_width: 8, width: 8, height: 1, x: 0, y: -15, data_offset: 1202 },  // 0x5F (_)
    FontGlyph { step_width: 8, width: 2, height: 2, x: 2, y: -9, data_offset: 1213 },   // 0x60 (`)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1220 },   // 0x61 (a)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1233 },   // 0x62 (b)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1251 },   // 0x63 (c)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1267 },   // 0x64 (d)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1285 },   // 0x65 (e)
    FontGlyph { step_width: 8, width: 4, height: 9, x: 2, y: -9, data_offset: 1303 },   // 0x66 (f)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 1317 },   // 0x67 (g)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1335 },   // 0x68 (h)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1353 },   // 0x69 (i)
    FontGlyph { step_width: 8, width: 5, height: 11, x: 1, y: -9, data_offset: 1371 },  // 0x6A (j)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1391 },   // 0x6B (k)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1411 },   // 0x6C (l)
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 1431 },   // 0x6D (m)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1453 },   // 0x6E (n)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1471 },   // 0x6F (o)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 1489 },   // 0x70 (p)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 1509 },   // 0x71 (q)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1529 },   // 0x72 (r)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1547 },   // 0x73 (s)
    FontGlyph { step_width: 8, width: 4, height: 8, x: 2, y: -8, data_offset: 1565 },   // 0x74 (t)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1578 },   // 0x75 (u)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1596 },   // 0x76 (v)
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 1614 },   // 0x77 (w)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1636 },   // 0x78 (x)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 1656 },   // 0x79 (y)
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 1675 },   // 0x7A (z)
    FontGlyph { step_width: 8, width: 4, height: 9, x: 2, y: -9, data_offset: 1693 },   // 0x7B ({)
    FontGlyph { step_width: 8, width: 1, height: 9, x: 3, y: -9, data_offset: 1707 },   // 0x7C (|)
    FontGlyph { step_width: 8, width: 4, height: 9, x: 1, y: -9, data_offset: 1719 },   // 0x7D (})
    FontGlyph { step_width: 8, width: 6, height: 2, x: 1, y: -9, data_offset: 1733 },   // 0x7E (~)
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1744 },   // 0x7F
];

// Bit reversal lookup table for fast pixel extraction
const BIT_REV_TABLE: [u8; 256] = [
    0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0,
    0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70, 0xF0,
    0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8,
    0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8, 0x78, 0xF8,
    0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4,
    0x14, 0x94, 0x54, 0xD4, 0x34, 0xB4, 0x74, 0xF4,
    0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC,
    0x1C, 0x9C, 0x5C, 0xDC, 0x3C, 0xBC, 0x7C, 0xFC,
    0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2,
    0x12, 0x92, 0x52, 0xD2, 0x32, 0xB2, 0x72, 0xF2,
    0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA,
    0x1A, 0x9A, 0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA,
    0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6,
    0x16, 0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6,
    0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE,
    0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE,
    0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61, 0xE1,
    0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1,
    0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9, 0x69, 0xE9,
    0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9,
    0x05, 0x85, 0x45, 0xC5, 0x25, 0xA5, 0x65, 0xE5,
    0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5,
    0x0D, 0x8D, 0x4D, 0xCD, 0x2D, 0xAD, 0x6D, 0xED,
    0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD,
    0x03, 0x83, 0x43, 0xC3, 0x23, 0xA3, 0x63, 0xE3,
    0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3,
    0x0B, 0x8B, 0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB,
    0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB,
    0x07, 0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7,
    0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7,
    0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF,
    0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F, 0xFF,
];

/// Draw a single glyph character to the framebuffer
/// 
/// Based on fbdev_video.cpp draw_text_impl():
/// - Shadow pass: draws shadow pixels at (x + px + fd.x + 1, y + py + fd.y + 1)
/// - Main pass: draws text pixels at (x + px + fd.x, y + py + fd.y)
/// - Bit 0 = leftmost pixel
pub unsafe fn draw_char(
    fb_ptr: *mut u8,
    fb_pitch: u32,
    fb_bpp: u32,
    x: i32,
    y: i32,
    ch: u8,
    text_color: u32,
    shadow_color: u32,
    shadow: bool,
) {
    let glyph_idx = (ch as usize).min(127);
    let glyphs = SMALL_FONT_GLYPHS;
    let font_data = SMALL_FONT_DATA;
    let font_h = SMALL_FONT_H;
    
    draw_char_impl(fb_ptr, fb_pitch, fb_bpp, x, y, ch, text_color, shadow_color, shadow,
                   glyph_idx, glyphs, font_data, font_h);
}

/// Draw a single glyph character to the framebuffer (big font)
pub unsafe fn draw_char_big(
    fb_ptr: *mut u8,
    fb_pitch: u32,
    fb_bpp: u32,
    x: i32,
    y: i32,
    ch: u8,
    text_color: u32,
    shadow_color: u32,
    shadow: bool,
) {
    let glyph_idx = (ch as usize).min(127);
    let glyphs = BIG_FONT_GLYPHS;
    let font_data = BIG_FONT_DATA;
    let font_h = BIG_FONT_H;
    
    draw_char_impl(fb_ptr, fb_pitch, fb_bpp, x, y, ch, text_color, shadow_color, shadow,
                   glyph_idx, glyphs, font_data, font_h);
}

unsafe fn draw_char_impl(
    fb_ptr: *mut u8,
    fb_pitch: u32,
    fb_bpp: u32,
    x: i32,
    y: i32,
    ch: u8,
    text_color: u32,
    shadow_color: u32,
    shadow: bool,
    glyph_idx: usize,
    glyphs: &[FontGlyph],
    font_data: &[u8],
    _font_h: c_int,
) {
    let fd = glyphs[glyph_idx];
    let w = fd.width as usize;
    let h = fd.height as usize;
    
    if w == 0 || h == 0 {
        return;
    }
    
    // Bytes per row (rounded up)
    let step = (w + 7) >> 3;
    if fd.data_offset >= font_data.len() {
        eprintln!("Font: data_offset {} >= font_data.len() {}, skipping glyph", fd.data_offset, font_data.len());
        return;
    }
    let glyph_data = &font_data[fd.data_offset..];
    
    for py in 0..h {
        let row_offset = py * step;
        if row_offset >= glyph_data.len() {
            break;
        }
        
        for px in 0..w {
            let byte_idx = row_offset + (px >> 3);
            if byte_idx >= glyph_data.len() {
                break;
            }
            
            let bit = glyph_data[byte_idx];
            // Bit 0 = leftmost pixel
            let on = (bit >> (px & 7)) & 1 != 0;
            
            if !on {
                continue;
            }
            
            // Main text position
            let mx = x + (px as i32) + fd.x;
            let my = y + (py as i32) + fd.y;
            if mx >= 0 && my >= 0 {
                write_pixel(fb_ptr, fb_pitch, fb_bpp, mx, my, text_color);
            }
            
            // Shadow pass
            if shadow {
                let sx = x + (px as i32) + fd.x + 1;
                let sy = y + (py as i32) + fd.y + 1;
                if sx >= 0 && sy >= 0 {
                    write_pixel(fb_ptr, fb_pitch, fb_bpp, sx, sy, shadow_color);
                }
            }
        }
    }
}

/// Write a single pixel to the framebuffer with format conversion
#[inline]
pub unsafe fn write_pixel(fb_ptr: *mut u8, fb_pitch: u32, fb_bpp: u32, x: i32, y: i32, color: u32) {
    let offset = (y as usize) * (fb_pitch as usize) + (x as usize) * ((fb_bpp / 8) as usize);
    let dest = fb_ptr.add(offset);
    
    match fb_bpp {
        32 => {
            *(dest as *mut u32) = color;
        }
        16 => {
            // XRGB8888 to RGB565
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;
            let rgb565 = ((r & 0xF8) as u16) << 8 | ((g & 0xFC) as u16) << 3 | (b >> 3) as u16;
            *(dest as *mut u16) = rgb565;
        }
        _ => {}
    }
}

/// Draw a string to the framebuffer
pub unsafe fn draw_text(
    fb_ptr: *mut u8,
    fb_pitch: u32,
    fb_bpp: u32,
    x: i32,
    y: i32,
    text: &[u8],
    color: u32,
) {
    let mut cx = x;
    for &ch in text {
        let fd = SMALL_FONT_GLYPHS[(ch as usize).min(127)];
        draw_char(fb_ptr, fb_pitch, fb_bpp, cx, y, ch, color, 0x000000, true);
        cx += fd.step_width;
    }
}

/// Draw a string to the framebuffer (big font)
pub unsafe fn draw_text_big(
    fb_ptr: *mut u8,
    fb_pitch: u32,
    fb_bpp: u32,
    x: i32,
    y: i32,
    text: &[u8],
    color: u32,
) {
    let mut cx = x;
    for &ch in text {
        let fd = BIG_FONT_GLYPHS[(ch as usize).min(127)];
        draw_char_big(fb_ptr, fb_pitch, fb_bpp, cx, y, ch, color, 0x000000, true);
        cx += fd.step_width;
    }
}

/// Measure text width (in pixels) using small font
pub fn measure_text(text: &[u8]) -> i32 {
    let mut width = 0;
    for &ch in text {
        width += SMALL_FONT_GLYPHS[(ch as usize).min(127)].step_width;
    }
    width
}

/// Measure text width (in pixels) using big font
pub fn measure_text_big(text: &[u8]) -> i32 {
    let mut width = 0;
    for &ch in text {
        width += BIG_FONT_GLYPHS[(ch as usize).min(127)].step_width;
    }
    width
}
