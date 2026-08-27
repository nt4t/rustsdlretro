use libc::{c_int, c_uint, c_ushort, c_void};
use std::ptr;

use std::ffi::CString;

use crate::font;

/// Generic video backend trait for rendering core frames and GUI overlays.
/// Both FbdevVideo and MinifbVideo implement this trait.
pub trait VideoBackend {
    /// Push a frame from the core to the display
    fn push_frame(&mut self, pixels: *const c_void, frame_w: u32, frame_h: u32, pitch: usize);

    /// Set the core format (resolution)
    fn set_core_format(&mut self, width: u32, height: u32, bpp: u32);

    /// Set skip_frame flag (next frame will be skipped)
    fn set_skip_frame(&mut self);

    /// Clear the overlay area
    fn clear_overlay(&mut self, fb_width: u32, fb_height: u32);

    /// Draw a single pixel
    fn draw_pixel_overlay(&mut self, x: i32, y: i32, color: u32);

    /// Draw a horizontal line
    fn draw_hline_overlay(&mut self, x1: i32, x2: i32, y: i32, color: u32);

    /// Draw a vertical line
    fn draw_vline_overlay(&mut self, x: i32, y1: i32, y2: i32, color: u32);

    /// Draw a filled rectangle
    fn draw_rect_overlay(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32);

    /// Draw text
    fn draw_text_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32);

    /// Draw big text
    fn draw_text_big_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32);

    /// Draw a character
    fn draw_char_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32);

    /// Draw a big character
    fn draw_char_big_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32);

    /// Check if a frame was drawn
    fn frame_drawn(&self) -> bool;

    /// Get the display width
    fn fb_width(&self) -> u32;

    /// Get the display height
    fn fb_height(&self) -> u32;

    /// Get the display bpp
    fn fb_bpp(&self) -> u32;

    /// Check if this backend is MinifbVideo (for calling update_window)
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any>;

    /// Update the window (minifb only)
    fn update_window(&mut self);

    /// Check if the backend requests to close (minifb ESC only)
    fn should_close(&self) -> bool;

    /// Set a snapshot callback that receives raw frame data after pixel copy.
    /// Called from push_frame() before GUI overlay rendering. Returns true if capture succeeded.
    #[cfg(feature = "api")]
    fn set_snapshot_callback(
        &mut self,
        callback: Option<fn(pixels: *const u8, width: u32, height: u32, core_bpp: u32)>,
    );

    /// Take a snapshot of the current frame (used by API for PNG streaming).
    #[cfg(feature = "api")]
    fn take_snapshot(&mut self) -> Option<FrameSnapshot>;
}

// ioctl constants for fbdev
const FBIOGET_FSCREENINFO: c_uint = 0x4602;
const FBIOGET_VSCREENINFO: c_uint = 0x4600;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_fix_screeninfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: c_uint,
    pub type_: c_uint,
    pub type_aux: c_uint,
    pub visual: c_uint,
    pub xpanstep: c_ushort,
    pub ypanstep: c_ushort,
    pub ywrapstep: c_ushort,
    pub line_length: c_uint,
    pub mmio_start: usize,
    pub mmio_len: c_uint,
    pub accel: c_uint,
    pub capabilities: c_ushort,
    pub reserved: [c_ushort; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_var_screeninfo {
    pub xres: c_uint,
    pub yres: c_uint,
    pub xres_virtual: c_uint,
    pub yres_virtual: c_uint,
    pub xoffset: c_uint,
    pub yoffset: c_uint,
    pub bits_per_pixel: c_uint,
    pub grayscale: c_uint,
    pub red: fb_bitfield,
    pub green: fb_bitfield,
    pub blue: fb_bitfield,
    pub transp: fb_bitfield,
    pub nonstd: c_uint,
    pub activate: c_uint,
    pub height: c_uint,
    pub width: c_uint,
    pub accel_flags: c_uint,
    pub pixclock: c_uint,
    pub left_margin: c_uint,
    pub right_margin: c_uint,
    pub upper_margin: c_uint,
    pub lower_margin: c_uint,
    pub hsync_len: c_uint,
    pub vsync_len: c_uint,
    pub sync: c_uint,
    pub vmode: c_uint,
    pub rotate: c_uint,
    pub colorspace: c_uint,
    pub reserved: [c_uint; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_bitfield {
    pub offset: c_uint,
    pub length: c_uint,
    pub lsb_right: c_uint,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CoreFormat {
    pub bpp: u32,
    pub width: u32,
    pub height: u32,
}

impl CoreFormat {
    pub const UNINITIALIZED: Self = CoreFormat {
        bpp: 0,
        width: 0,
        height: 0,
    };
}

pub static mut CORE_FORMAT: CoreFormat = CoreFormat::UNINITIALIZED;

// ─── Frame Snapshot (for API PNG streaming) ────────────────────────────────────

/// A captured frame ready for encoding.
#[cfg(feature = "api")]
pub struct FrameSnapshot {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels  
    pub height: u32,
    /// Pixel data (XRGB8888 or RGB565 depending on core_bpp)
    pub pixels: Vec<u8>,
    /// Bits per pixel of the source (16 or 32)
    pub core_bpp: u32,
}

#[cfg(feature = "api")]
impl FrameSnapshot {
    fn new(width: u32, height: u32, core_bpp: u32) -> Self {
        let bytes_per_pixel = (core_bpp / 8) as usize;
        let pitch = width as usize * bytes_per_pixel;
        Self {
            width,
            height,
            pixels: vec![0u8; pitch * height as usize],
            core_bpp,
        }
    }
}

/// Callback type for frame snapshots.
pub type SnapshotCallback = Option<fn(pixels: *const u8, width: u32, height: u32, core_bpp: u32)>;

/// Global snapshot callback (set once by API module).
#[cfg(feature = "api")]
static mut SNAPSHOT_CALLBACK: std::cell::Cell<Option<fn(pixels: *const u8, width: u32, height: u32, core_bpp: u32)>> 
    = std::cell::Cell::new(None);

#[cfg(feature = "api")]
pub fn set_snapshot_callback(cb: SnapshotCallback) {
    unsafe { SNAPSHOT_CALLBACK.set(cb); }
}

#[cfg(feature = "api")]
pub fn call_snapshot_callback(pixels: *const u8, width: u32, height: u32, core_bpp: u32) {
    unsafe {
        if let Some(cb) = SNAPSHOT_CALLBACK.get() {
            cb(pixels, width, height, core_bpp);
        }
    }
}

#[inline]
fn xrgb8888_to_rgb565(p: u32) -> u16 {
    let r = (p >> 16) & 0xFF;
    let g = (p >> 8) & 0xFF;
    let b = p & 0xFF;
    ((r & 0xF8) as u16) << 8 | ((g & 0xFC) as u16) << 3 | (b >> 3) as u16
}

#[inline]
fn rgb565_to_xrgb8888(p: u16) -> u32 {
    let r5 = (p >> 11) & 0x1F;
    let g6 = (p >> 5) & 0x3F;
    let b5 = p & 0x1F;
    ((r5 << 3 | r5 >> 2) as u32) << 16 | ((g6 << 2 | g6 >> 4) as u32) << 8 | (b5 << 3 | b5 >> 2) as u32
}

pub struct FbdevVideo {
    fb_fd: c_int,
    fb_ptr: *mut u8,
    fb_len: usize,
    fb_width: u32,
    fb_height: u32,
    fb_pitch: u32,
    fb_bpp: u32,
    core_width: u32,
    core_height: u32,
    offset_x: i32,
    offset_y: i32,
    skip_frame: bool,
    frame_drawn: bool,
    /// Buffer for snapshot capture (API PNG streaming)
    #[cfg(feature = "api")]
    snapshot_buffer: Option<Vec<u8>>,
}

impl FbdevVideo {
    pub fn new() -> Result<Self, String> {
        let path = CString::new("/dev/fb0").map_err(|_| "fb0 path contains null bytes".to_string())?;
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            return Err(format!("Failed to open /dev/fb0: {}", std::io::Error::last_os_error()));
        }

        let mut fix: fb_fix_screeninfo = unsafe { std::mem::zeroed() };
        let mut var: fb_var_screeninfo = unsafe { std::mem::zeroed() };

        if unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO as _, &mut fix) } < 0 {
            unsafe { libc::close(fd) };
            return Err(format!("FBIOGET_FSCREENINFO failed: {}", std::io::Error::last_os_error()));
        }

      if unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO as _, &mut var) } < 0 {
            unsafe { libc::close(fd) };
            return Err(format!("FBIOGET_VSCREENINFO failed: {}", std::io::Error::last_os_error()));
        }

        let fb_len = fix.smem_len as usize;
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                fb_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(format!("mmap failed: {}", std::io::Error::last_os_error()));
        }

        Ok(FbdevVideo {
            fb_fd: fd,
            fb_ptr: ptr as *mut u8,
            fb_len,
            fb_width: var.xres,
            fb_height: var.yres,
            fb_pitch: fix.line_length,
            fb_bpp: var.bits_per_pixel,
            core_width: 0,
            core_height: 0,
            offset_x: 0,
            offset_y: 0,
            skip_frame: false,
            frame_drawn: false,
            #[cfg(feature = "api")]
            snapshot_buffer: None,
        })
    }

    pub fn set_core_format(&mut self, width: u32, height: u32, _bpp: u32) {
        self.core_width = width;
        self.core_height = height;
        self.offset_x = ((self.fb_width as i32) - (width as i32)) / 2;
        self.offset_y = ((self.fb_height as i32) - (height as i32)) / 2;
    }

    pub fn push_frame(&mut self, pixels: *const c_void, frame_w: u32, frame_h: u32, pitch: usize) {
        if self.skip_frame {
            self.skip_frame = false;
            self.frame_drawn = false;
            return;
        }

        let format = unsafe { CORE_FORMAT };
        let core_bpp = format.bpp;
        if core_bpp == 0 {
            return;
        }

        if frame_w == 0 || frame_h == 0 {
            return;
        }

        let frame_w = frame_w as usize;
        let frame_h = frame_h as usize;

        self.frame_drawn = true;
        let fb_bpp = self.fb_bpp;
        let fb_ptr = self.fb_ptr;
        let fb_pitch = self.fb_pitch as usize;
        let offset_x = ((self.fb_width as usize) - frame_w) / 2;
        let offset_y = ((self.fb_height as usize) - frame_h) / 2;

        if fb_bpp == 32 && core_bpp == 32 {
            for y in 0..frame_h {
                let src_row = unsafe { (pixels as *const u32).add((y as usize) * (pitch / 4)) };
                let row = offset_y + y;
                let dest_row = unsafe { fb_ptr.add(row * fb_pitch + offset_x * 4) } as *mut u32;
                unsafe {
                    ptr::copy_nonoverlapping(src_row, dest_row, frame_w);
                }
            }
        } else if fb_bpp == 16 && core_bpp == 16 {
            for y in 0..frame_h {
                let src_row = unsafe { (pixels as *const u16).add((y as usize) * (pitch / 2)) };
                let row = offset_y + y;
                let dest_row = unsafe { fb_ptr.add(row * fb_pitch + offset_x * 2) } as *mut u16;
                unsafe {
                    ptr::copy_nonoverlapping(src_row, dest_row, frame_w);
                }
            }
        } else if fb_bpp == 32 && core_bpp == 16 {
            // Optimize: process entire row at once using slice::from_raw_parts
            let mut src_buf = vec![0u16; frame_w];
            let mut dst_buf = vec![0u32; frame_w];
            for y in 0..frame_h {
                unsafe {
                    // Copy source row
                    let src_row = (pixels as *const u16).add((y as usize) * (pitch / 2));
                    ptr::copy_nonoverlapping(src_row, src_buf.as_mut_ptr(), frame_w);
                    // Convert to destination format
                    for i in 0..frame_w {
                        dst_buf[i] = rgb565_to_xrgb8888(src_buf[i]) as u32;
                    }
                    // Write to framebuffer
                    let row = offset_y + y;
                    let dest_row = fb_ptr.add(row * fb_pitch + offset_x * 4) as *mut u32;
                    ptr::copy_nonoverlapping(dst_buf.as_ptr(), dest_row, frame_w);
                }
            }
        } else if fb_bpp == 16 && core_bpp == 32 {
            // Optimize: process entire row at once using slice::from_raw_parts
            let mut src_buf = vec![0u32; frame_w];
            let mut dst_buf = vec![0u16; frame_w];
            for y in 0..frame_h {
                unsafe {
                    // Copy source row
                    let src_row = (pixels as *const u32).add((y as usize) * (pitch / 4));
                    ptr::copy_nonoverlapping(src_row, src_buf.as_mut_ptr(), frame_w);
                    // Convert to destination format
                    for i in 0..frame_w {
                        dst_buf[i] = xrgb8888_to_rgb565(src_buf[i]) as u16;
                    }
                    // Write to framebuffer
                    let row = offset_y + y;
                    let dest_row = fb_ptr.add(row * fb_pitch + offset_x * 2) as *mut u16;
                    ptr::copy_nonoverlapping(dst_buf.as_ptr(), dest_row, frame_w);
                }
            }
        }

        // Push captured frame for API PNG streaming (non-blocking)
        #[cfg(feature = "api")]
        {
            use crate::api;
            let mut snapshot = api::CapturedFrame::new(frame_w as u32, frame_h as u32);
            if core_bpp == 32 {
                // XRGB8888 -> RGBA conversion
                for y in 0..frame_h {
                    let src_row = unsafe { (pixels as *const u32).add((y as usize) * (pitch / 4)) };
                    let dst_offset = y as usize * frame_w as usize * 4;
                    let dst = &mut snapshot.pixels[dst_offset..dst_offset + frame_w as usize * 4];
                    for x in 0..frame_w {
                        let pixel = unsafe { *src_row.add(x) };
                        // XRGB8888 -> RGBA: add alpha=255
                        dst[x * 4]     = (pixel >> 16) as u8; // R
                        dst[x * 4 + 1] = (pixel >> 8) as u8;  // G
                        dst[x * 4 + 2] = pixel as u8;          // B
                        dst[x * 4 + 3] = 0xFF;                 // A
                    }
                }
            } else {
                // RGB565 -> RGBA conversion with lookup table
                let mut rgb565_lut: [u32; 65536] = core::array::from_fn(|_| 0);
                for i in 0u16..=65535u16 {
                    let r5 = (i >> 11) & 0x1F;
                    let g6 = (i >> 5) & 0x3F;
                    let b5 = i & 0x1F;
                    let r = ((r5 << 3) | (r5 >> 2)) as u8;
                    let g = ((g6 << 2) | (g6 >> 4)) as u8;
                    let b = ((b5 << 3) | (b5 >> 2)) as u8;
                    rgb565_lut[i as usize] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                }
                for y in 0..frame_h {
                    let src_row = unsafe { (pixels as *const u16).add((y as usize) * (pitch / 2)) };
                    let dst_offset = y as usize * frame_w as usize * 4;
                    let dst = &mut snapshot.pixels[dst_offset..dst_offset + frame_w as usize * 4];
                    for x in 0..frame_w {
                        let pixel = unsafe { *src_row.add(x) };
                        let rgba = rgb565_lut[pixel as usize];
                        dst[x * 4]     = (rgba >> 16) as u8; // R
                        dst[x * 4 + 1] = (rgba >> 8) as u8;  // G
                        dst[x * 4 + 2] = rgba as u8;          // B
                        dst[x * 4 + 3] = 0xFF;                // A
                    }
                }
            }
            api::push_captured_frame(snapshot);
        }
    }

    pub fn set_skip_frame(&mut self) {
        self.skip_frame = true;
        self.frame_drawn = false;
    }

    pub fn clear_overlay(&mut self, fb_width: u32, fb_height: u32) {
        self.draw_rect_overlay(0, 0, fb_width as i32, fb_height as i32, 0x000000);
    }

    pub fn draw_pixel_overlay(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 || x as u32 >= self.fb_width || y as u32 >= self.fb_height {
            return;
        }
        unsafe {
            font::write_pixel(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, color);
        }
    }

    /// Draw a horizontal line (optimized bulk write)
    pub fn draw_hline_overlay(&mut self, x1: i32, x2: i32, y: i32, color: u32) {
        if y < 0 || y as u32 >= self.fb_height {
            return;
        }
        let x1 = x1.max(0).min(self.fb_width as i32);
        let x2 = x2.max(0).min(self.fb_width as i32);
        if x1 >= x2 {
            return;
        }

        let width = (x2 - x1) as usize;
        let fb_pitch = self.fb_pitch as usize;
        let fb_bpp = self.fb_bpp;

        unsafe {
            if fb_bpp == 32 {
                let color_u32 = color;
                let row_offset = (y as usize) * fb_pitch + (x1 as usize) * 4;
                let mut dest = self.fb_ptr.add(row_offset) as *mut u32;
                for _ in 0..width {
                    *dest = color_u32;
                    dest = dest.add(1);
                }
            } else if fb_bpp == 16 {
                let r = ((color >> 16) & 0xFF) as u16;
                let g = ((color >> 8) & 0xFF) as u16;
                let b = (color & 0xFF) as u16;
                let rgb565 = (r & 0xF8) << 8 | (g & 0xFC) << 3 | (b >> 3);
                let row_offset = (y as usize) * fb_pitch + (x1 as usize) * 2;
                let mut dest = self.fb_ptr.add(row_offset) as *mut u16;
                for _ in 0..width {
                    *dest = rgb565;
                    dest = dest.add(1);
                }
            } else {
                for x in x1..x2 {
                    font::write_pixel(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, color);
                }
            }
        }
    }

    /// Draw a vertical line (optimized bulk write)
    pub fn draw_vline_overlay(&mut self, x: i32, y1: i32, y2: i32, color: u32) {
        if x < 0 || x as u32 >= self.fb_width {
            return;
        }
        let y1 = y1.max(0).min(self.fb_height as i32);
        let y2 = y2.max(0).min(self.fb_height as i32);
        if y1 >= y2 {
            return;
        }

        let height = (y2 - y1) as usize;
        let fb_pitch = self.fb_pitch as usize;
        let fb_bpp = self.fb_bpp;
        let x_offset = (x as usize) * ((fb_bpp / 8) as usize);

        unsafe {
            if fb_bpp == 32 {
                let color_u32 = color;
                let base_row = (y1 as usize) * fb_pitch + x_offset;
                let mut dest = self.fb_ptr.add(base_row) as *mut u32;
                for _ in 0..height {
                    *dest = color_u32;
                    dest = dest.add(fb_pitch / 4);
                }
            } else if fb_bpp == 16 {
                let r = ((color >> 16) & 0xFF) as u16;
                let g = ((color >> 8) & 0xFF) as u16;
                let b = (color & 0xFF) as u16;
                let rgb565 = (r & 0xF8) << 8 | (g & 0xFC) << 3 | (b >> 3);
                let base_row = (y1 as usize) * fb_pitch + x_offset;
                let mut dest = self.fb_ptr.add(base_row) as *mut u16;
                for _ in 0..height {
                    *dest = rgb565;
                    dest = dest.add(fb_pitch / 2);
                }
            } else {
                for y in y1..y2 {
                    font::write_pixel(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, color);
                }
            }
        }
    }

    pub fn draw_rect_overlay(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        let x1 = x1.max(0).min(self.fb_width as i32);
        let y1 = y1.max(0).min(self.fb_height as i32);
        let x2 = x2.max(0).min(self.fb_width as i32);
        let y2 = y2.max(0).min(self.fb_height as i32);
        if x1 >= x2 || y1 >= y2 {
            return;
        }

        let width = (x2 - x1) as usize;
        let fb_pitch = self.fb_pitch as usize;
        let fb_bpp = self.fb_bpp;

        unsafe {
            if fb_bpp == 32 {
                let color_u32 = color;
                for y in y1..y2 {
                    let row_offset = (y as usize) * fb_pitch + (x1 as usize) * 4;
                    let mut dest = self.fb_ptr.add(row_offset) as *mut u32;
                    for _ in 0..width {
                        *dest = color_u32;
                        dest = dest.add(1);
                    }
                }
            } else if fb_bpp == 16 {
                let r = ((color >> 16) & 0xFF) as u16;
                let g = ((color >> 8) & 0xFF) as u16;
                let b = (color & 0xFF) as u16;
                let rgb565 = (r & 0xF8) << 8 | (g & 0xFC) << 3 | (b >> 3);
                for y in y1..y2 {
                    let row_offset = (y as usize) * fb_pitch + (x1 as usize) * 2;
                    let mut dest = self.fb_ptr.add(row_offset) as *mut u16;
                    for _ in 0..width {
                        *dest = rgb565;
                        dest = dest.add(1);
                    }
                }
            } else {
                // Fallback to pixel-by-pixel for unsupported bpp
                for y in y1..y2 {
                    for x in x1..x2 {
                        font::write_pixel(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, color);
                    }
                }
            }
        }
    }

    pub fn draw_text_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        unsafe {
            font::draw_text(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, text, color);
        }
    }

    pub fn draw_text_big_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        unsafe {
            font::draw_text_big(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, text, color);
        }
    }

    pub fn draw_char_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        unsafe {
            font::draw_char(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, ch, color, 0x000000, true);
        }
    }

    pub fn draw_char_big_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        unsafe {
            font::draw_char_big(self.fb_ptr, self.fb_pitch, self.fb_bpp, x, y, ch, color, 0x000000, true);
        }
    }

    pub fn frame_drawn(&self) -> bool {
        self.frame_drawn
    }

    pub fn close(self) {
        if !self.fb_ptr.is_null() {
            unsafe {
                libc::munmap(self.fb_ptr as *mut c_void, self.fb_len);
            }
        }
        unsafe {
            libc::close(self.fb_fd);
        }
    }

    pub fn fb_width(&self) -> u32 {
        self.fb_width
    }

    pub fn fb_height(&self) -> u32 {
        self.fb_height
    }

    pub fn fb_bpp(&self) -> u32 {
        self.fb_bpp
    }

    /// Set the snapshot callback for this backend.
    #[cfg(feature = "api")]
    pub fn set_snapshot_callback(&mut self, width: u32, height: u32) {
        let bytes_per_pixel = 4; // Always capture as XRGB8888
        let pitch = (width * bytes_per_pixel) as usize;
        self.snapshot_buffer = Some(vec![0u8; pitch * height as usize]);
    }
}

impl Drop for FbdevVideo {
    fn drop(&mut self) {
        if !self.fb_ptr.is_null() {
            unsafe {
                libc::munmap(self.fb_ptr as *mut c_void, self.fb_len);
                self.fb_ptr = ptr::null_mut();
            }
        }
        if self.fb_fd >= 0 {
            unsafe {
                libc::close(self.fb_fd);
                self.fb_fd = -1;
            }
        }
    }
}

impl VideoBackend for FbdevVideo {
    fn push_frame(&mut self, pixels: *const c_void, frame_w: u32, frame_h: u32, pitch: usize) {
        FbdevVideo::push_frame(self, pixels, frame_w, frame_h, pitch)
    }

    fn set_core_format(&mut self, width: u32, height: u32, bpp: u32) {
        FbdevVideo::set_core_format(self, width, height, bpp)
    }

    fn set_skip_frame(&mut self) {
        FbdevVideo::set_skip_frame(self)
    }

    fn clear_overlay(&mut self, fb_width: u32, fb_height: u32) {
        FbdevVideo::clear_overlay(self, fb_width, fb_height)
    }

    fn draw_pixel_overlay(&mut self, x: i32, y: i32, color: u32) {
        FbdevVideo::draw_pixel_overlay(self, x, y, color)
    }

    fn draw_hline_overlay(&mut self, x1: i32, x2: i32, y: i32, color: u32) {
        FbdevVideo::draw_hline_overlay(self, x1, x2, y, color)
    }

    fn draw_vline_overlay(&mut self, x: i32, y1: i32, y2: i32, color: u32) {
        FbdevVideo::draw_vline_overlay(self, x, y1, y2, color)
    }

    fn draw_rect_overlay(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        FbdevVideo::draw_rect_overlay(self, x1, y1, x2, y2, color)
    }

    fn draw_text_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        FbdevVideo::draw_text_overlay(self, x, y, text, color)
    }

    fn draw_text_big_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        FbdevVideo::draw_text_big_overlay(self, x, y, text, color)
    }

    fn draw_char_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        FbdevVideo::draw_char_overlay(self, x, y, ch, color)
    }

    fn draw_char_big_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        FbdevVideo::draw_char_big_overlay(self, x, y, ch, color)
    }

    fn frame_drawn(&self) -> bool {
        FbdevVideo::frame_drawn(self)
    }

    fn fb_width(&self) -> u32 {
        FbdevVideo::fb_width(self)
    }

    fn fb_height(&self) -> u32 {
        FbdevVideo::fb_height(self)
    }

    fn fb_bpp(&self) -> u32 {
        FbdevVideo::fb_bpp(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }

    fn update_window(&mut self) {
        // No-op for fbdev
    }

    fn should_close(&self) -> bool {
        false
    }

    #[cfg(feature = "api")]
    fn set_snapshot_callback(
        &mut self,
        _callback: Option<fn(pixels: *const u8, width: u32, height: u32, core_bpp: u32)>,
    ) {
        // Callback is stored globally via call_snapshot_callback in push_frame
        // This method is for API initialization to set up snapshot buffer size
        let format = unsafe { CORE_FORMAT };
        self.set_snapshot_callback(format.width, format.height);
    }

    #[cfg(feature = "api")]
    fn take_snapshot(&mut self) -> Option<FrameSnapshot> {
        // Not implemented for fbdev - snapshot captured via callback during push_frame
        None
    }
}
