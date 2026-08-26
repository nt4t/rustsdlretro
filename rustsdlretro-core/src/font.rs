// Auto-generated from fonts/bmfont.inl - DO NOT EDIT MANUALLY
// Bitmap font renderer for framebuffer overlay

use std::os::raw::c_int;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FontGlyph {
    pub step_width: c_int,
    pub width: c_int,
    pub height: c_int,
    pub x: c_int,
    pub y: c_int,
    pub data_offset: usize,
}

pub const SMALL_FONT_H: c_int = 8;
pub const SMALL_FONT_DATA: &[u8] = &[
    0x03,     0x03,     0x03,     0x03,     0x03,     0x00,     0x03,     0x05,     0x05,     0x0A,     0x1F,     0x1F,     0x0A,     0x1F,     0x1F,     0x0A, 
    0x0A,     0x1E,     0x0B,     0x0F,     0x1E,     0x1A,     0x0F,     0x0A,     0x13,     0x1B,     0x18,     0x0C,     0x04,     0x36,     0x32,     0x0E, 
    0x1B,     0x0E,     0x2F,     0x3B,     0x3B,     0x2E,     0x01,     0x01,     0x04,     0x02,     0x03,     0x03,     0x03,     0x03,     0x02,     0x04, 
    0x01,     0x02,     0x06,     0x06,     0x06,     0x06,     0x02,     0x01,     0x04,     0x15,     0x1F,     0x0E,     0x1F,     0x15,     0x04,     0x04, 
    0x04,     0x1F,     0x04,     0x04,     0x02,     0x03,     0x01,     0x0F,     0x03,     0x03,     0x04,     0x04,     0x04,     0x06,     0x02,     0x03, 
    0x03,     0x01,     0x0E,     0x1B,     0x1B,     0x1B,     0x1B,     0x1B,     0x0E,     0x06,     0x07,     0x06,     0x06,     0x06,     0x06,     0x06, 
    0x0F,     0x18,     0x18,     0x0E,     0x03,     0x03,     0x1F,     0x0F,     0x18,     0x18,     0x0E,     0x18,     0x18,     0x0F,     0x18,     0x1C, 
    0x1A,     0x19,     0x1F,     0x18,     0x18,     0x0F,     0x01,     0x0F,     0x18,     0x18,     0x18,     0x0F,     0x0E,     0x03,     0x0F,     0x1B, 
    0x1B,     0x1B,     0x0E,     0x1F,     0x18,     0x0C,     0x0C,     0x06,     0x06,     0x06,     0x0E,     0x1B,     0x1B,     0x0E,     0x1B,     0x1B, 
    0x0E,     0x0E,     0x1B,     0x1B,     0x1B,     0x1E,     0x18,     0x0E,     0x03,     0x03,     0x00,     0x03,     0x03,     0x03,     0x03,     0x00, 
    0x02,     0x03,     0x01,     0x08,     0x0C,     0x06,     0x03,     0x06,     0x0C,     0x08,     0x0F,     0x00,     0x0F,     0x01,     0x03,     0x06, 
    0x0C,     0x06,     0x03,     0x01,     0x0F,     0x18,     0x0C,     0x06,     0x06,     0x00,     0x06,     0x1E,     0x33,     0x3F,     0x3B,     0x3F, 
    0x03,     0x1E,     0x0E,     0x1B,     0x1B,     0x1B,     0x1F,     0x1B,     0x1B,     0x0F,     0x1B,     0x0F,     0x1B,     0x1B,     0x1B,     0x0F, 
    0x1E,     0x03,     0x03,     0x03,     0x03,     0x03,     0x1E,     0x0F,     0x1B,     0x1B,     0x1B,     0x1B,     0x1B,     0x0F,     0x1F,     0x03, 
    0x0F,     0x03,     0x03,     0x03,     0x1F,     0x1F,     0x03,     0x0F,     0x03,     0x03,     0x03,     0x03,     0x0E,     0x03,     0x03,     0x1B, 
    0x1B,     0x1B,     0x1E,     0x1B,     0x1B,     0x1F,     0x1B,     0x1B,     0x1B,     0x1B,     0x03,     0x03,     0x03,     0x03,     0x03,     0x03, 
    0x03,     0x0C,     0x0C,     0x0C,     0x0C,     0x0C,     0x0C,     0x07,     0x33,     0x1B,     0x0F,     0x07,     0x0F,     0x1B,     0x33,     0x03, 
    0x03,     0x03,     0x03,     0x03,     0x03,     0x0F,     0x41,     0x63,     0x77,     0x7F,     0x6B,     0x63,     0x63,     0x31,     0x33,     0x37, 
    0x3F,     0x3B,     0x33,     0x23,     0x1E,     0x33,     0x33,     0x33,     0x33,     0x33,     0x1E,     0x0F,     0x1B,     0x1B,     0x1B,     0x0F, 
    0x03,     0x03,     0x1E,     0x33,     0x33,     0x33,     0x33,     0x3B,     0x1E,     0x30,     0x0F,     0x1B,     0x1B,     0x1B,     0x0F,     0x0B, 
    0x1B,     0x0E,     0x03,     0x03,     0x06,     0x0C,     0x0C,     0x07,     0x3F,     0x0C,     0x0C,     0x0C,     0x0C,     0x0C,     0x0C,     0x1B, 
    0x1B,     0x1B,     0x1B,     0x1B,     0x1B,     0x0E,     0x33,     0x33,     0x33,     0x1E,     0x1E,     0x0C,     0x0C,     0x63,     0x63,     0x6B, 
    0x7F,     0x3E,     0x3E,     0x36,     0x33,     0x33,     0x1E,     0x0C,     0x1E,     0x33,     0x33,     0x33,     0x33,     0x1E,     0x0C,     0x0C, 
    0x0C,     0x0C,     0x1F,     0x18,     0x1C,     0x0E,     0x07,     0x03,     0x1F,     0x07,     0x03,     0x03,     0x03,     0x03,     0x03,     0x03, 
    0x07,     0x01,     0x01,     0x01,     0x03,     0x02,     0x06,     0x06,     0x04,     0x07,     0x06,     0x06,     0x06,     0x06,     0x06,     0x06, 
    0x07,     0x02,     0x07,     0x05,     0x3F,     0x01,     0x02,     0x0E,     0x18,     0x1E,     0x1B,     0x1E,     0x03,     0x03,     0x0F,     0x1B, 
    0x1B,     0x1B,     0x0F,     0x0E,     0x03,     0x03,     0x03,     0x0E,     0x18,     0x18,     0x1E,     0x1B,     0x1B,     0x1B,     0x1E,     0x0E, 
    0x1B,     0x1F,     0x03,     0x1E,     0x06,     0x03,     0x07,     0x03,     0x03,     0x03,     0x03,     0x1E,     0x1B,     0x1B,     0x1E,     0x18, 
    0x0E,     0x03,     0x03,     0x0F,     0x1B,     0x1B,     0x1B,     0x1B,     0x03,     0x00,     0x03,     0x03,     0x03,     0x03,     0x03,     0x06, 
    0x00,     0x06,     0x06,     0x06,     0x06,     0x06,     0x03,     0x03,     0x03,     0x1B,     0x0F,     0x07,     0x0F,     0x1B,     0x03,     0x03, 
    0x03,     0x03,     0x03,     0x03,     0x03,     0x7F,     0xDB,     0xDB,     0xDB,     0xDB,     0x0F,     0x1B,     0x1B,     0x1B,     0x1B,     0x0E, 
    0x1B,     0x1B,     0x1B,     0x0E,     0x0F,     0x1B,     0x1B,     0x0F,     0x03,     0x03,     0x1E,     0x1B,     0x1B,     0x1E,     0x18,     0x18, 
    0x0B,     0x0F,     0x03,     0x03,     0x03,     0x0E,     0x03,     0x0F,     0x0C,     0x07,     0x03,     0x03,     0x07,     0x03,     0x03,     0x03, 
    0x06,     0x1B,     0x1B,     0x1B,     0x1B,     0x1E,     0x1B,     0x1B,     0x0E,     0x0E,     0x04,     0x63,     0x6B,     0x6B,     0x3E,     0x36, 
    0x1B,     0x1B,     0x0E,     0x1B,     0x1B,     0x1B,     0x1B,     0x1B,     0x1E,     0x18,     0x0E,     0x1F,     0x0C,     0x06,     0x03,     0x1F, 
    0x07,     0x03,     0x03,     0x03,     0x03,     0x03,     0x03,     0x07,     0x03,     0x03,     0x03,     0x03,     0x03,     0x03,     0x03,     0x03, 
    0x07,     0x06,     0x06,     0x06,     0x06,     0x06,     0x06,     0x07,     0x16,     0x0D, ];

pub const SMALL_FONT_GLYPHS: &[FontGlyph] = &[
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x00
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x01
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x02
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x03
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x04
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x05
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x06
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x07
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x08
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x09
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x0A
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x0B
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x0C
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x0D
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x0E
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x0F
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x10
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x11
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x12
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x13
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x14
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x15
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x16
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x17
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x18
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x19
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x1A
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x1B
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x1C
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x1D
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x1E
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x1F
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 0 },  // 0x20
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 0 },  // 0x21
    FontGlyph { step_width: 4, width: 3, height: 2, x: 0, y: -7, data_offset: 7 },  // 0x22
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 9 },  // 0x23
    FontGlyph { step_width: 6, width: 5, height: 8, x: 0, y: -7, data_offset: 16 },  // 0x24
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 24 },  // 0x25
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 31 },  // 0x26
    FontGlyph { step_width: 2, width: 1, height: 2, x: 0, y: -7, data_offset: 38 },  // 0x27
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 40 },  // 0x28
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 48 },  // 0x29
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 56 },  // 0x2A
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -6, data_offset: 63 },  // 0x2B
    FontGlyph { step_width: 3, width: 2, height: 3, x: 0, y: -2, data_offset: 68 },  // 0x2C
    FontGlyph { step_width: 5, width: 4, height: 1, x: 0, y: -4, data_offset: 71 },  // 0x2D
    FontGlyph { step_width: 3, width: 2, height: 2, x: 0, y: -2, data_offset: 72 },  // 0x2E
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 74 },  // 0x2F
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 82 },  // 0x30
    FontGlyph { step_width: 4, width: 3, height: 7, x: 0, y: -7, data_offset: 89 },  // 0x31
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 96 },  // 0x32
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 103 },  // 0x33
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 110 },  // 0x34
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 117 },  // 0x35
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 124 },  // 0x36
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 131 },  // 0x37
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 138 },  // 0x38
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 145 },  // 0x39
    FontGlyph { step_width: 3, width: 2, height: 5, x: 0, y: -5, data_offset: 152 },  // 0x3A
    FontGlyph { step_width: 3, width: 2, height: 6, x: 0, y: -5, data_offset: 157 },  // 0x3B
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 163 },  // 0x3C
    FontGlyph { step_width: 5, width: 4, height: 3, x: 0, y: -4, data_offset: 170 },  // 0x3D
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 173 },  // 0x3E
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 180 },  // 0x3F
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -6, data_offset: 187 },  // 0x40
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 194 },  // 0x41
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 201 },  // 0x42
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 208 },  // 0x43
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 215 },  // 0x44
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 222 },  // 0x45
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 229 },  // 0x46
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 236 },  // 0x47
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 243 },  // 0x48
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 250 },  // 0x49
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 257 },  // 0x4A
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 264 },  // 0x4B
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 271 },  // 0x4C
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 278 },  // 0x4D
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 285 },  // 0x4E
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 292 },  // 0x4F
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 299 },  // 0x50
    FontGlyph { step_width: 7, width: 6, height: 8, x: 0, y: -7, data_offset: 306 },  // 0x51
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 314 },  // 0x52
    FontGlyph { step_width: 5, width: 4, height: 7, x: 0, y: -7, data_offset: 321 },  // 0x53
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 328 },  // 0x54
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 335 },  // 0x55
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 342 },  // 0x56
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 349 },  // 0x57
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 356 },  // 0x58
    FontGlyph { step_width: 7, width: 6, height: 7, x: 0, y: -7, data_offset: 363 },  // 0x59
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 370 },  // 0x5A
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 377 },  // 0x5B
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -8, data_offset: 385 },  // 0x5C
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 393 },  // 0x5D
    FontGlyph { step_width: 4, width: 3, height: 3, x: 0, y: -7, data_offset: 401 },  // 0x5E
    FontGlyph { step_width: 6, width: 6, height: 1, x: 0, y: -1, data_offset: 404 },  // 0x5F
    FontGlyph { step_width: 3, width: 2, height: 2, x: 0, y: -7, data_offset: 405 },  // 0x60
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 407 },  // 0x61
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 412 },  // 0x62
    FontGlyph { step_width: 5, width: 4, height: 5, x: 0, y: -5, data_offset: 419 },  // 0x63
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 424 },  // 0x64
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 431 },  // 0x65
    FontGlyph { step_width: 4, width: 3, height: 7, x: 0, y: -7, data_offset: 436 },  // 0x66
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 443 },  // 0x67
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 449 },  // 0x68
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 456 },  // 0x69
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 463 },  // 0x6A
    FontGlyph { step_width: 6, width: 5, height: 7, x: 0, y: -7, data_offset: 471 },  // 0x6B
    FontGlyph { step_width: 3, width: 2, height: 7, x: 0, y: -7, data_offset: 478 },  // 0x6C
    FontGlyph { step_width: 9, width: 8, height: 5, x: 0, y: -5, data_offset: 485 },  // 0x6D
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 490 },  // 0x6E
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 495 },  // 0x6F
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 500 },  // 0x70
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 506 },  // 0x71
    FontGlyph { step_width: 5, width: 4, height: 5, x: 0, y: -5, data_offset: 512 },  // 0x72
    FontGlyph { step_width: 5, width: 4, height: 5, x: 0, y: -5, data_offset: 517 },  // 0x73
    FontGlyph { step_width: 4, width: 3, height: 7, x: 0, y: -7, data_offset: 522 },  // 0x74
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 529 },  // 0x75
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 534 },  // 0x76
    FontGlyph { step_width: 8, width: 7, height: 5, x: 0, y: -5, data_offset: 539 },  // 0x77
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 544 },  // 0x78
    FontGlyph { step_width: 6, width: 5, height: 6, x: 0, y: -5, data_offset: 549 },  // 0x79
    FontGlyph { step_width: 6, width: 5, height: 5, x: 0, y: -5, data_offset: 555 },  // 0x7A
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 560 },  // 0x7B
    FontGlyph { step_width: 3, width: 2, height: 8, x: 0, y: -7, data_offset: 568 },  // 0x7C
    FontGlyph { step_width: 4, width: 3, height: 8, x: 0, y: -7, data_offset: 576 },  // 0x7D
    FontGlyph { step_width: 6, width: 5, height: 2, x: 0, y: -7, data_offset: 584 },  // 0x7E
    FontGlyph { step_width: 3, width: 0, height: 0, x: 0, y: 0, data_offset: 586 },  // 0x7F
];

pub const BIG_FONT_H: c_int = 16;
pub const BIG_FONT_DATA: &[u8] = &[
    0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F, 
    0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F, 
    0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x00,     0x01,     0x05,     0x05,     0x05,     0x12,     0x12,     0x3F,     0x12, 
    0x12,     0x12,     0x3F,     0x12,     0x12,     0x04,     0x04,     0x0E,     0x15,     0x05,     0x05,     0x0E,     0x14,     0x14,     0x15,     0x0E, 
    0x04,     0x04,     0x02,     0x05,     0x25,     0x12,     0x08,     0x24,     0x52,     0x50,     0x20,     0x0E,     0x11,     0x01,     0x01,     0x0E, 
    0x11,     0x11,     0x11,     0x1E,     0x01,     0x01,     0x01,     0x04,     0x02,     0x01,     0x01,     0x01,     0x01,     0x01,     0x02,     0x04, 
    0x01,     0x02,     0x04,     0x04,     0x04,     0x04,     0x04,     0x02,     0x01,     0x04,     0x15,     0x0E,     0x15,     0x04,     0x04,     0x04, 
    0x1F,     0x04,     0x04,     0x02,     0x02,     0x01,     0x0F,     0x01,     0x04,     0x04,     0x04,     0x02,     0x02,     0x02,     0x01,     0x01, 
    0x01,     0x0E,     0x11,     0x11,     0x19,     0x15,     0x13,     0x11,     0x11,     0x0E,     0x04,     0x06,     0x05,     0x04,     0x04,     0x04, 
    0x04,     0x04,     0x1F,     0x0E,     0x11,     0x10,     0x08,     0x04,     0x02,     0x01,     0x01,     0x1F,     0x0E,     0x11,     0x10,     0x10, 
    0x0C,     0x10,     0x10,     0x11,     0x0E,     0x10,     0x18,     0x14,     0x12,     0x11,     0x1F,     0x10,     0x10,     0x10,     0x1F,     0x01, 
    0x01,     0x0F,     0x10,     0x10,     0x10,     0x11,     0x0E,     0x0E,     0x11,     0x01,     0x01,     0x0F,     0x11,     0x11,     0x11,     0x0E, 
    0x1F,     0x10,     0x10,     0x08,     0x04,     0x02,     0x01,     0x01,     0x01,     0x0E,     0x11,     0x11,     0x11,     0x0E,     0x11,     0x11, 
    0x11,     0x0E,     0x0E,     0x11,     0x11,     0x11,     0x1E,     0x10,     0x10,     0x11,     0x0E,     0x01,     0x00,     0x00,     0x00,     0x00, 
    0x00,     0x01,     0x02,     0x00,     0x00,     0x00,     0x00,     0x00,     0x02,     0x02,     0x01,     0x04,     0x02,     0x01,     0x02,     0x04, 
    0x0F,     0x00,     0x0F,     0x01,     0x02,     0x04,     0x02,     0x01,     0x0E,     0x11,     0x10,     0x08,     0x04,     0x04,     0x04,     0x00, 
    0x04,     0x3E,     0x41,     0x59,     0x55,     0x55,     0x55,     0x39,     0x01,     0x3E,     0x0E,     0x11,     0x11,     0x11,     0x11,     0x1F, 
    0x11,     0x11,     0x11,     0x0F,     0x11,     0x11,     0x11,     0x0F,     0x11,     0x11,     0x11,     0x0F,     0x0E,     0x11,     0x01,     0x01, 
    0x01,     0x01,     0x01,     0x11,     0x0E,     0x0F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x0F,     0x1F,     0x01, 
    0x01,     0x01,     0x07,     0x01,     0x01,     0x01,     0x1F,     0x1F,     0x01,     0x01,     0x01,     0x07,     0x01,     0x01,     0x01,     0x01, 
    0x0E,     0x11,     0x01,     0x01,     0x19,     0x11,     0x11,     0x11,     0x1E,     0x11,     0x11,     0x11,     0x11,     0x1F,     0x11,     0x11, 
    0x11,     0x11,     0x1F,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x1F,     0x3C,     0x30,     0x10,     0x10,     0x10, 
    0x10,     0x10,     0x11,     0x0E,     0x11,     0x11,     0x09,     0x05,     0x03,     0x05,     0x09,     0x11,     0x11,     0x01,     0x01,     0x01, 
    0x01,     0x01,     0x01,     0x01,     0x01,     0x1F,     0x41,     0x41,     0x63,     0x55,     0x49,     0x41,     0x41,     0x41,     0x41,     0x11, 
    0x11,     0x11,     0x13,     0x15,     0x19,     0x11,     0x11,     0x11,     0x0E,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x0E,     0x0F,     0x11,     0x11,     0x11,     0x0F,     0x01,     0x01,     0x01,     0x01,     0x0E,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x15,     0x09,     0x16,     0x0F,     0x11,     0x11,     0x11,     0x0F,     0x05,     0x09,     0x11,     0x11,     0x0E,     0x11,     0x01,     0x01, 
    0x0E,     0x10,     0x10,     0x11,     0x0E,     0x1F,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x11,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x0E,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x0A,     0x04, 
    0x41,     0x41,     0x49,     0x49,     0x49,     0x49,     0x49,     0x49,     0x36,     0x11,     0x11,     0x11,     0x0A,     0x04,     0x0A,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x11,     0x0A,     0x04,     0x04,     0x04,     0x04,     0x04,     0x1F,     0x10,     0x10,     0x08,     0x04, 
    0x02,     0x01,     0x01,     0x1F,     0x07,     0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x07,     0x01,     0x01,     0x01, 
    0x02,     0x02,     0x02,     0x04,     0x04,     0x04,     0x07,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x07,     0x04, 
    0x0A,     0x11,     0xFF,     0x01,     0x02,     0x0E,     0x11,     0x10,     0x1E,     0x11,     0x11,     0x1E,     0x01,     0x01,     0x0F,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x0F,     0x0E,     0x11,     0x01,     0x01,     0x01,     0x11,     0x0E,     0x10,     0x10,     0x1E,     0x11, 
    0x11,     0x11,     0x11,     0x11,     0x1E,     0x0E,     0x11,     0x11,     0x1F,     0x01,     0x11,     0x0E,     0x0C,     0x02,     0x02,     0x0F, 
    0x02,     0x02,     0x02,     0x02,     0x02,     0x1E,     0x11,     0x11,     0x11,     0x11,     0x1E,     0x10,     0x11,     0x0E,     0x01,     0x01, 
    0x0F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x04,     0x00,     0x07,     0x04,     0x04,     0x04,     0x04,     0x04,     0x1F, 
    0x10,     0x00,     0x1C,     0x10,     0x10,     0x10,     0x10,     0x10,     0x10,     0x11,     0x0E,     0x01,     0x01,     0x11,     0x09,     0x05, 
    0x03,     0x05,     0x09,     0x11,     0x07,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x04,     0x1F,     0x37,     0x49,     0x49, 
    0x49,     0x49,     0x41,     0x41,     0x0F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x0E,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x0E,     0x0F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x0F,     0x01,     0x01,     0x1E,     0x11,     0x11,     0x11,     0x11, 
    0x11,     0x1E,     0x10,     0x10,     0x19,     0x05,     0x03,     0x01,     0x01,     0x01,     0x01,     0x0E,     0x11,     0x01,     0x0E,     0x10, 
    0x11,     0x0E,     0x02,     0x0F,     0x02,     0x02,     0x02,     0x02,     0x02,     0x0C,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11, 
    0x0E,     0x11,     0x11,     0x11,     0x11,     0x11,     0x0A,     0x04,     0x41,     0x41,     0x49,     0x49,     0x49,     0x49,     0x36,     0x11, 
    0x11,     0x0A,     0x04,     0x0A,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1E,     0x10,     0x11,     0x0E,     0x1F, 
    0x10,     0x08,     0x04,     0x02,     0x01,     0x1F,     0x0C,     0x02,     0x02,     0x02,     0x01,     0x02,     0x02,     0x02,     0x0C,     0x01, 
    0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x01,     0x03,     0x04,     0x04,     0x04,     0x08,     0x04,     0x04,     0x04, 
    0x03,     0x26,     0x19,     0x1F,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x11,     0x1F, ];

pub const BIG_FONT_GLYPHS: &[FontGlyph] = &[
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 0 },  // 0x00
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 9 },  // 0x01
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 18 },  // 0x02
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 27 },  // 0x03
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 36 },  // 0x04
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 45 },  // 0x05
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 54 },  // 0x06
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 63 },  // 0x07
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 72 },  // 0x08
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 81 },  // 0x09
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 90 },  // 0x0A
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 99 },  // 0x0B
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 108 },  // 0x0C
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 117 },  // 0x0D
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 126 },  // 0x0E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 135 },  // 0x0F
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 144 },  // 0x10
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 153 },  // 0x11
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 162 },  // 0x12
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 171 },  // 0x13
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 180 },  // 0x14
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 189 },  // 0x15
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 198 },  // 0x16
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 207 },  // 0x17
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 216 },  // 0x18
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 225 },  // 0x19
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 234 },  // 0x1A
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 243 },  // 0x1B
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 252 },  // 0x1C
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 261 },  // 0x1D
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 270 },  // 0x1E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 279 },  // 0x1F
    FontGlyph { step_width: 8, width: 0, height: 0, x: 0, y: 0, data_offset: 288 },  // 0x20
    FontGlyph { step_width: 8, width: 1, height: 9, x: 3, y: -9, data_offset: 288 },  // 0x21
    FontGlyph { step_width: 8, width: 3, height: 3, x: 2, y: -9, data_offset: 297 },  // 0x22
    FontGlyph { step_width: 8, width: 6, height: 9, x: 1, y: -9, data_offset: 300 },  // 0x23
    FontGlyph { step_width: 8, width: 5, height: 13, x: 1, y: -11, data_offset: 309 },  // 0x24
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 322 },  // 0x25
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 331 },  // 0x26
    FontGlyph { step_width: 8, width: 1, height: 3, x: 3, y: -9, data_offset: 340 },  // 0x27
    FontGlyph { step_width: 8, width: 3, height: 9, x: 3, y: -9, data_offset: 343 },  // 0x28
    FontGlyph { step_width: 8, width: 3, height: 9, x: 1, y: -9, data_offset: 352 },  // 0x29
    FontGlyph { step_width: 8, width: 5, height: 5, x: 1, y: -9, data_offset: 361 },  // 0x2A
    FontGlyph { step_width: 8, width: 5, height: 5, x: 1, y: -7, data_offset: 366 },  // 0x2B
    FontGlyph { step_width: 8, width: 2, height: 3, x: 2, y: -1, data_offset: 371 },  // 0x2C
    FontGlyph { step_width: 8, width: 4, height: 1, x: 2, y: -5, data_offset: 374 },  // 0x2D
    FontGlyph { step_width: 8, width: 1, height: 1, x: 3, y: -1, data_offset: 375 },  // 0x2E
    FontGlyph { step_width: 8, width: 3, height: 9, x: 2, y: -9, data_offset: 376 },  // 0x2F
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 385 },  // 0x30
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 394 },  // 0x31
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 403 },  // 0x32
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 412 },  // 0x33
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 421 },  // 0x34
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 430 },  // 0x35
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 439 },  // 0x36
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 448 },  // 0x37
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 457 },  // 0x38
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 466 },  // 0x39
    FontGlyph { step_width: 8, width: 1, height: 7, x: 3, y: -7, data_offset: 475 },  // 0x3A
    FontGlyph { step_width: 8, width: 2, height: 9, x: 2, y: -7, data_offset: 482 },  // 0x3B
    FontGlyph { step_width: 8, width: 3, height: 5, x: 2, y: -7, data_offset: 491 },  // 0x3C
    FontGlyph { step_width: 8, width: 4, height: 3, x: 2, y: -6, data_offset: 496 },  // 0x3D
    FontGlyph { step_width: 8, width: 3, height: 5, x: 2, y: -7, data_offset: 499 },  // 0x3E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 504 },  // 0x3F
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 513 },  // 0x40
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 522 },  // 0x41
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 531 },  // 0x42
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 540 },  // 0x43
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 549 },  // 0x44
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 558 },  // 0x45
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 567 },  // 0x46
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 576 },  // 0x47
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 585 },  // 0x48
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 594 },  // 0x49
    FontGlyph { step_width: 8, width: 6, height: 9, x: 1, y: -9, data_offset: 603 },  // 0x4A
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 612 },  // 0x4B
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 621 },  // 0x4C
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 630 },  // 0x4D
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 639 },  // 0x4E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 648 },  // 0x4F
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 657 },  // 0x50
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 666 },  // 0x51
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 675 },  // 0x52
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 684 },  // 0x53
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 693 },  // 0x54
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 702 },  // 0x55
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 711 },  // 0x56
    FontGlyph { step_width: 8, width: 7, height: 9, x: 0, y: -9, data_offset: 720 },  // 0x57
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 729 },  // 0x58
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 738 },  // 0x59
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 747 },  // 0x5A
    FontGlyph { step_width: 8, width: 3, height: 9, x: 3, y: -9, data_offset: 756 },  // 0x5B
    FontGlyph { step_width: 8, width: 3, height: 9, x: 2, y: -9, data_offset: 765 },  // 0x5C
    FontGlyph { step_width: 8, width: 3, height: 9, x: 1, y: -9, data_offset: 774 },  // 0x5D
    FontGlyph { step_width: 8, width: 5, height: 3, x: 1, y: -9, data_offset: 783 },  // 0x5E
    FontGlyph { step_width: 8, width: 8, height: 1, x: 0, y: 1, data_offset: 786 },  // 0x5F
    FontGlyph { step_width: 8, width: 2, height: 2, x: 2, y: -9, data_offset: 787 },  // 0x60
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 789 },  // 0x61
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 796 },  // 0x62
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 805 },  // 0x63
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 812 },  // 0x64
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 821 },  // 0x65
    FontGlyph { step_width: 8, width: 4, height: 9, x: 2, y: -9, data_offset: 828 },  // 0x66
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 837 },  // 0x67
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 846 },  // 0x68
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 855 },  // 0x69
    FontGlyph { step_width: 8, width: 5, height: 11, x: 1, y: -9, data_offset: 864 },  // 0x6A
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 875 },  // 0x6B
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 884 },  // 0x6C
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 893 },  // 0x6D
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 900 },  // 0x6E
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 907 },  // 0x6F
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 914 },  // 0x70
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 923 },  // 0x71
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 932 },  // 0x72
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 939 },  // 0x73
    FontGlyph { step_width: 8, width: 4, height: 8, x: 2, y: -8, data_offset: 946 },  // 0x74
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 954 },  // 0x75
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 961 },  // 0x76
    FontGlyph { step_width: 8, width: 7, height: 7, x: 0, y: -7, data_offset: 968 },  // 0x77
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 975 },  // 0x78
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -7, data_offset: 982 },  // 0x79
    FontGlyph { step_width: 8, width: 5, height: 7, x: 1, y: -7, data_offset: 991 },  // 0x7A
    FontGlyph { step_width: 8, width: 4, height: 9, x: 2, y: -9, data_offset: 998 },  // 0x7B
    FontGlyph { step_width: 8, width: 1, height: 9, x: 3, y: -9, data_offset: 1007 },  // 0x7C
    FontGlyph { step_width: 8, width: 4, height: 9, x: 1, y: -9, data_offset: 1016 },  // 0x7D
    FontGlyph { step_width: 8, width: 6, height: 2, x: 1, y: -9, data_offset: 1025 },  // 0x7E
    FontGlyph { step_width: 8, width: 5, height: 9, x: 1, y: -9, data_offset: 1027 },  // 0x7F
];

#[allow(dead_code)]
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
    draw_char_impl(fb_ptr, fb_pitch, fb_bpp, x, y, ch, text_color, shadow_color, shadow,
                   glyph_idx, glyphs, font_data);
}

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
    draw_char_impl(fb_ptr, fb_pitch, fb_bpp, x, y, ch, text_color, shadow_color, shadow,
                   glyph_idx, glyphs, font_data);
}

unsafe fn draw_char_impl(
    fb_ptr: *mut u8,
    fb_pitch: u32,
    fb_bpp: u32,
    x: i32,
    y: i32,
    _ch: u8,
    text_color: u32,
    shadow_color: u32,
    shadow: bool,
    glyph_idx: usize,
    glyphs: &[FontGlyph],
    font_data: &[u8],
) {
    let fd = glyphs[glyph_idx];
    let w = fd.width as usize;
    let h = fd.height as usize;
    if w == 0 || h == 0 {
        return;
    }
    let step = (w + 7) >> 3;
    if fd.data_offset >= font_data.len() {
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
            let on = (bit >> (px & 7)) & 1 != 0;
            if !on {
                continue;
            }
            let mx = x + (px as i32) + fd.x;
            let my = y + (py as i32) + fd.y;
            if mx >= 0 && my >= 0 {
                write_pixel(fb_ptr, fb_pitch, fb_bpp, mx, my, text_color);
            }
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

#[inline]
pub unsafe fn write_pixel(fb_ptr: *mut u8, fb_pitch: u32, fb_bpp: u32, x: i32, y: i32, color: u32) {
    let offset = (y as usize) * (fb_pitch as usize) + (x as usize) * ((fb_bpp / 8) as usize);
    let dest = fb_ptr.add(offset);
    match fb_bpp {
        32 => {
            *(dest as *mut u32) = color;
        }
        16 => {
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;
            let rgb565 = ((r & 0xF8) as u16) << 8 | ((g & 0xFC) as u16) << 3 | (b >> 3) as u16;
            *(dest as *mut u16) = rgb565;
        }
        _ => {}
    }
}

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
        draw_char(fb_ptr, fb_pitch, fb_bpp, cx, y, ch, color, 0x000000, false);
        cx += fd.step_width;
    }
}

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
        draw_char_big(fb_ptr, fb_pitch, fb_bpp, cx, y, ch, color, 0x000000, false);
        cx += fd.step_width;
    }
}

pub fn measure_text(text: &[u8]) -> i32 {
    let mut width = 0;
    for &ch in text {
        width += SMALL_FONT_GLYPHS[(ch as usize).min(127)].step_width;
    }
    width
}

pub fn measure_text_big(text: &[u8]) -> i32 {
    let mut width = 0;
    for &ch in text {
        width += BIG_FONT_GLYPHS[(ch as usize).min(127)].step_width;
    }
    width
}
