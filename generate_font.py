import re

# Read bmfont.inl
with open('fonts/bmfont.inl', 'r') as f:
    content = f.read()

# Extract small font binary data
small_match = re.search(r'const unsigned char font_small_bin_data\[\] = \{([^}]+)\}', content, re.DOTALL)
small_data = [int(x.strip(), 16) for x in small_match.group(1).split(',') if x.strip()]

# Extract big font binary data
big_match = re.search(r'const unsigned char font_big_bin_data\[\] = \{([^}]+)\}', content, re.DOTALL)
big_data = [int(x.strip(), 16) for x in big_match.group(1).split(',') if x.strip()]

# Extract small font glyphs
small_glyphs = []
for m in re.finditer(r'/\* (0x[0-9A-F]+) \*/ \{(\d+), (\d+), (\d+), (\d+), (\d+) - 8, font_small_bin_data \+ (0x[0-9A-F]+)\}', content):
    code = int(m.group(1), 16)
    sw = int(m.group(2))
    w = int(m.group(3))
    h = int(m.group(4))
    x = int(m.group(5))
    y = int(m.group(6)) - 8
    data_offset = int(m.group(7), 16)
    if code < 128:
        small_glyphs.append((code, sw, w, h, x, y, data_offset))

# Extract big font glyphs
big_glyphs = []
for m in re.finditer(r'/\* (0x[0-9A-F]+) \*/ \{(\d+), (\d+), (\d+), (\d+), (\d+) - 16, font_big_bin_data \+ (0x[0-9A-F]+)\}', content):
    code = int(m.group(1), 16)
    sw = int(m.group(2))
    w = int(m.group(3))
    h = int(m.group(4))
    x = int(m.group(5))
    y = int(m.group(6)) - 16
    data_offset = int(m.group(7), 16)
    if code < 128:
        big_glyphs.append((code, sw, w, h, x, y, data_offset))

def generate_rust():
    rust_code = '''// Auto-generated from fonts/bmfont.inl - DO NOT EDIT MANUALLY
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
'''
    for i, b in enumerate(small_data):
        rust_code += f'    0x{b:02X}, '
        if (i + 1) % 16 == 0:
            rust_code += '\n'
    rust_code += '];\n\npub const SMALL_FONT_GLYPHS: &[FontGlyph] = &[\n'
    small_glyphs.sort(key=lambda g: g[0])
    for code, sw, w, h, x, y, data_offset in small_glyphs:
        rust_code += f'    FontGlyph {{ step_width: {sw}, width: {w}, height: {h}, x: {x}, y: {y}, data_offset: {data_offset} }},  // 0x{code:02X}\n'
    rust_code += '];\n\npub const BIG_FONT_H: c_int = 16;\npub const BIG_FONT_DATA: &[u8] = &[\n'
    for i, b in enumerate(big_data):
        rust_code += f'    0x{b:02X}, '
        if (i + 1) % 16 == 0:
            rust_code += '\n'
    rust_code += '];\n\npub const BIG_FONT_GLYPHS: &[FontGlyph] = &[\n'
    big_glyphs.sort(key=lambda g: g[0])
    for code, sw, w, h, x, y, data_offset in big_glyphs:
        rust_code += f'    FontGlyph {{ step_width: {sw}, width: {w}, height: {h}, x: {x}, y: {y}, data_offset: {data_offset} }},  // 0x{code:02X}\n'
    rust_code += '];\n'
    return rust_code

def generate_rendering():
    return '''
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
    ch: u8,
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
'''

full_rust = generate_rust() + generate_rendering()

with open('sdlretro-core/src/font.rs', 'w') as f:
    f.write(full_rust)

print(f'Generated font.rs with {len(small_glyphs)} small glyphs and {len(big_glyphs)} big glyphs')
print(f'Small font data: {len(small_data)} bytes')
print(f'Big font data: {len(big_data)} bytes')
