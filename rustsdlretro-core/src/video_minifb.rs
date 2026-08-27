/// Minifb-based windowed video backend for desktop development/testing.
/// Provides the same VideoBackend interface as FbdevVideo but renders to an X11 window.

use crate::font;
use crate::video::{self, FrameSnapshot, VideoBackend};
use minifb::{Scale, ScaleMode, Window, WindowOptions};
use std::ffi::c_void;

/// Detect whether the system is running X11 or Wayland.
pub fn detect_display_server() -> &'static str {
    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => "wayland",
        Ok("x11" | "tty") => "x11",
        _ => {
            // Fallback: check for env vars
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                "wayland"
            } else if std::env::var_os("DISPLAY").is_some() {
                "x11"
            } else {
                "unknown"
            }
        }
    }
}

// Re-export minifb Key for use by frontend
pub use minifb::Key;

/// Minifb video backend
pub struct MinifbVideo {
    window: Window,
    buffer: Vec<u32>,
    width: u32,
    height: u32,
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

impl MinifbVideo {
    /// Create a new MinifbVideo window
    pub fn new(
        window_width: u32,
        window_height: u32,
        scale: u32,
        borderless: bool,
        title: &str,
    ) -> Result<Self, String> {
        let scale_factor = match scale {
            1 => Scale::X1,
            2 => Scale::X2,
            4 => Scale::X4,
            8 => Scale::X8,
            16 => Scale::X16,
            32 => Scale::X32,
            _ => Scale::X2,
        };

        let opts = WindowOptions {
            scale: scale_factor,
            scale_mode: ScaleMode::Stretch,
            borderless,
            resize: false,
            title: true,
            topmost: false,
            transparency: false,
            none: false,
        };

        let window = Window::new(title, window_width as usize, window_height as usize, opts)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        // Report display server type for diagnostics
        eprintln!("Display: {}", detect_display_server());

        let buffer = vec![0u32; (window_width * window_height) as usize];

        Ok(Self {
            window,
            buffer,
            width: window_width,
            height: window_height,
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

    /// Update the window with the current buffer
    pub fn update(&mut self) {
        let _ = self.window.update_with_buffer(&self.buffer, self.width as usize, self.height as usize);
    }

    /// Check if the window is still open
    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    /// Check if a key is currently pressed
    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    /// Check if the window should close (ESC pressed)
    pub fn should_close(&self) -> bool {
        self.window.is_key_down(Key::Escape)
    }

    /// Set the core format (resolution)
    pub fn set_core_format(&mut self, width: u32, height: u32, _bpp: u32) {
        self.core_width = width;
        self.core_height = height;
        self.offset_x = ((self.width as i32) - (width as i32)) / 2;
        self.offset_y = ((self.height as i32) - (height as i32)) / 2;
    }

    /// Push a frame from the core to the display buffer
    pub fn push_frame(&mut self, pixels: *const c_void, frame_w: u32, frame_h: u32, pitch: usize) {
        if self.skip_frame {
            self.skip_frame = false;
            self.frame_drawn = false;
            return;
        }

        let core_bpp = unsafe { crate::video::CORE_FORMAT.bpp };
        if core_bpp == 0 {
            return;
        }

        if frame_w == 0 || frame_h == 0 {
            return;
        }

        let frame_w = frame_w as usize;
        let frame_h = frame_h as usize;

        self.frame_drawn = true;
        let offset_x = ((self.width as usize) - frame_w) / 2;
        let offset_y = ((self.height as usize) - frame_h) / 2;

        // Clear the game area (center rectangle) to remove any previous content.
        // Also clear the left/right/top/bottom borders that may contain stale data
        // from a previously larger resolution frame.
        let buf_w = self.width as usize;
        for y in 0..self.height as usize {
            let row = y * buf_w;
            if y < offset_y || y >= offset_y + frame_h {
                // Top or bottom border — clear entire row
                for x in 0..buf_w {
                    self.buffer[row + x] = 0;
                }
            } else {
                // Middle rows — clear left and right borders only
                for x in 0..offset_x {
                    self.buffer[row + x] = 0;
                }
                let right_start = offset_x + frame_w;
                for x in right_start..buf_w {
                    self.buffer[row + x] = 0;
                }
            }
        }

        // Minifb expects ARGB8888 (u32 value 0xAARRGGBB). On little-endian the
        // in-memory byte order is B-G-R-A, so ARGB 0xAARRGGBB maps to bytes
        // BB GG RR AA in memory.
        if core_bpp == 32 {
            // Core is XRGB8888 - convert to minifb ARGB8888
            // Fast conversion: XRGB8888 (0x00RRGGBB) -> ARGB8888 (0xFF000000 | 0x00RRGGBB)
            // On little-endian, we can use ptr::copy_nonoverlapping for the bulk copy
            // and then fix up the alpha channel
            for y in 0..frame_h {
                let src_row = unsafe { (pixels as *const u32).add((y as usize) * (pitch / 4)) };
                let row_start = (offset_y + y) * self.width as usize + offset_x;
                unsafe {
                    let mut dest = self.buffer.as_mut_ptr().add(row_start);
                    for x in 0..frame_w {
                        let pixel = *src_row.add(x);
                        // XRGB8888 -> ARGB8888: set alpha to 0xFF
                        *dest = 0xFF000000 | pixel;
                        dest = dest.add(1);
                    }
                }
            }
        } else if core_bpp == 16 {
            // Core is RGB565 - convert to minifb ARGB8888
            // Use lookup table for fast conversion
            let mut rgb565_lut: [u32; 65536] = core::array::from_fn(|_| 0);
            for i in 0u16..=65535u16 {
                let r5 = (i >> 11) & 0x1F;
                let g6 = (i >> 5) & 0x3F;
                let b5 = i & 0x1F;
                let r = ((r5 << 3) | (r5 >> 2)) as u32;
                let g = ((g6 << 2) | (g6 >> 4)) as u32;
                let b = ((b5 << 3) | (b5 >> 2)) as u32;
                rgb565_lut[i as usize] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }

            for y in 0..frame_h {
                let src_row = unsafe { (pixels as *const u16).add((y as usize) * (pitch / 2)) };
                let row_start = (offset_y + y) * self.width as usize + offset_x;
                unsafe {
                    let mut dest = self.buffer.as_mut_ptr().add(row_start);
                    for x in 0..frame_w {
                        let pixel = *src_row.add(x);
                        *dest = rgb565_lut[pixel as usize];
                        dest = dest.add(1);
                    }
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

    /// Set skip_frame flag
    pub fn set_skip_frame(&mut self) {
        self.skip_frame = true;
        self.frame_drawn = false;
    }

    /// Clear the overlay area (fill with black)
    pub fn clear_overlay(&mut self, _fb_width: u32, _fb_height: u32) {
        // Fill entire buffer with black — needed when toggling menu state.
        // When entering playing state from menu, this removes leftover menu pixels.
        self.buffer.fill(0);
    }

    /// Draw a single pixel (32bpp only)
    pub fn draw_pixel_overlay(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return;
        }
        let r = (color >> 16) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = color & 0xFF;
        let offset = (y as usize) * self.width as usize + (x as usize);
        self.buffer[offset] = 0xFF000000 | (r << 16) | (g << 8) | b;
    }

    /// Draw a horizontal line (optimized bulk write)
    pub fn draw_hline_overlay(&mut self, x1: i32, x2: i32, y: i32, color: u32) {
        if y < 0 || y as u32 >= self.height {
            return;
        }
        let x1 = x1.max(0).min(self.width as i32);
        let x2 = x2.max(0).min(self.width as i32);
        if x1 >= x2 {
            return;
        }

        let r = (color >> 16) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = color & 0xFF;
        let minifb_color = 0xFF000000 | (r << 16) | (g << 8) | b;
        let row = (y as usize) * self.width as usize;
        unsafe {
            let mut dest = self.buffer.as_mut_ptr().add(row + (x1 as usize));
            for _ in 0..(x2 - x1) {
                *dest = minifb_color;
                dest = dest.add(1);
            }
        }
    }

    /// Draw a vertical line (optimized bulk write)
    pub fn draw_vline_overlay(&mut self, x: i32, y1: i32, y2: i32, color: u32) {
        if x < 0 || x as u32 >= self.width {
            return;
        }
        let y1 = y1.max(0).min(self.height as i32);
        let y2 = y2.max(0).min(self.height as i32);
        if y1 >= y2 {
            return;
        }

        let r = (color >> 16) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = color & 0xFF;
        let minifb_color = 0xFF000000 | (r << 16) | (g << 8) | b;
        let x_offset = x as usize;

        unsafe {
            for y in y1..y2 {
                let row = (y as usize) * self.width as usize + x_offset;
                *self.buffer.as_mut_ptr().add(row) = minifb_color;
            }
        }
    }

    /// Draw a filled rectangle (optimized bulk writes)
    pub fn draw_rect_overlay(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        let x1 = x1.max(0).min(self.width as i32);
        let y1 = y1.max(0).min(self.height as i32);
        let x2 = x2.max(0).min(self.width as i32);
        let y2 = y2.max(0).min(self.height as i32);
        if x1 >= x2 || y1 >= y2 {
            return;
        }

        let r = (color >> 16) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = color & 0xFF;
        let minifb_color = 0xFF000000 | (r << 16) | (g << 8) | b;
        let width = (x2 - x1) as usize;

        for y in y1..y2 {
            let row = (y as usize) * self.width as usize + (x1 as usize);
            unsafe {
                let mut dest = self.buffer.as_mut_ptr().add(row);
                for _ in 0..width {
                    *dest = minifb_color;
                    dest = dest.add(1);
                }
            }
        }
    }

    /// Draw text using the font renderer
    pub fn draw_text_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        // pitch must be in bytes per row (width * 4 for 32bpp)
        let pitch = self.width * 4;
        unsafe {
            font::draw_text(self.buffer.as_mut_ptr() as *mut u8, pitch, 32, x, y, text, color);
        }
    }

    /// Draw big text using the font renderer
    pub fn draw_text_big_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        let pitch = self.width * 4;
        unsafe {
            font::draw_text_big(self.buffer.as_mut_ptr() as *mut u8, pitch, 32, x, y, text, color);
        }
    }

    /// Draw a character using the font renderer
    pub fn draw_char_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        let pitch = self.width * 4;
        unsafe {
            font::draw_char(self.buffer.as_mut_ptr() as *mut u8, pitch, 32, x, y, ch, color, 0x000000, true);
        }
    }

    /// Draw a big character using the font renderer
    pub fn draw_char_big_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        let pitch = self.width * 4;
        unsafe {
            font::draw_char_big(self.buffer.as_mut_ptr() as *mut u8, pitch, 32, x, y, ch, color, 0x000000, true);
        }
    }

    /// Check if a frame was drawn
    pub fn frame_drawn(&self) -> bool {
        self.frame_drawn
    }

    /// Get the display width
    pub fn fb_width(&self) -> u32 {
        self.width
    }

    /// Get the display height
    pub fn fb_height(&self) -> u32 {
        self.height
    }

    /// Get the display bpp (always 32 for minifb)
    pub fn fb_bpp(&self) -> u32 {
        32
    }

    /// Set up snapshot buffer with core resolution size.
    #[cfg(feature = "api")]
    pub fn set_snapshot_callback(&mut self, width: u32, height: u32) {
        // Minifb stores ARGB8888 pixels. For PNG we want RGBA (standard).
        // Allocate buffer for core resolution in RGBA format.
        let bytes_per_pixel = 4;
        let pitch = (width * bytes_per_pixel) as usize;
        self.snapshot_buffer = Some(vec![0u8; pitch * height as usize]);
    }

    /// Take a snapshot of the current frame from the display buffer.
    #[cfg(feature = "api")]
    pub fn take_snapshot(&mut self) -> Option<FrameSnapshot> {
        let buf = self.snapshot_buffer.as_mut()?;
        if buf.is_empty() || self.core_width == 0 || self.core_height == 0 {
            return None;
        }

        // Copy the centered game area from display buffer to snapshot buffer.
        // Convert ARGB8888 (minifb format) -> RGBA for PNG encoding.
        let core_w = self.core_width as usize;
        let core_h = self.core_height as usize;
        let offset_x = self.offset_x.max(0) as usize;
        let offset_y = self.offset_y.max(0) as usize;

        let display_pitch = self.width as usize;

        for y in 0..core_h {
            let src_row = (offset_y + y) * display_pitch + offset_x;
            let dst_row = y * core_w;
            let dst = &mut buf[dst_row * 4..(dst_row + core_w) * 4];
            for x in 0..core_w {
                let pixel = self.buffer[src_row + x];
                // Minifb uses ARGB8888 (0xAARRGGBB). On little-endian:
                // Bytes in memory: BB GG RR AA
                // We want RGBA: R G B A
                dst[x * 4]     = (pixel >> 16) as u8; // R
                dst[x * 4 + 1] = (pixel >> 8) as u8;  // G
                dst[x * 4 + 2] = pixel as u8;          // B
                dst[x * 4 + 3] = 0xFF;                 // A (opaque)
            }
        }

        Some(FrameSnapshot {
            width: self.core_width,
            height: self.core_height,
            pixels: buf.clone(),
            core_bpp: 32,
        })
    }
}

impl VideoBackend for MinifbVideo {
    fn push_frame(&mut self, pixels: *const c_void, frame_w: u32, frame_h: u32, pitch: usize) {
        MinifbVideo::push_frame(self, pixels, frame_w, frame_h, pitch)
    }

    fn set_core_format(&mut self, width: u32, height: u32, bpp: u32) {
        MinifbVideo::set_core_format(self, width, height, bpp)
    }

    fn set_skip_frame(&mut self) {
        MinifbVideo::set_skip_frame(self)
    }

    fn clear_overlay(&mut self, fb_width: u32, fb_height: u32) {
        MinifbVideo::clear_overlay(self, fb_width, fb_height)
    }

    fn draw_pixel_overlay(&mut self, x: i32, y: i32, color: u32) {
        MinifbVideo::draw_pixel_overlay(self, x, y, color)
    }

    fn draw_hline_overlay(&mut self, x1: i32, x2: i32, y: i32, color: u32) {
        MinifbVideo::draw_hline_overlay(self, x1, x2, y, color)
    }

    fn draw_vline_overlay(&mut self, x: i32, y1: i32, y2: i32, color: u32) {
        MinifbVideo::draw_vline_overlay(self, x, y1, y2, color)
    }

    fn draw_rect_overlay(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        MinifbVideo::draw_rect_overlay(self, x1, y1, x2, y2, color)
    }

    fn draw_text_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        MinifbVideo::draw_text_overlay(self, x, y, text, color)
    }

    fn draw_text_big_overlay(&mut self, x: i32, y: i32, text: &[u8], color: u32) {
        MinifbVideo::draw_text_big_overlay(self, x, y, text, color)
    }

    fn draw_char_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        MinifbVideo::draw_char_overlay(self, x, y, ch, color)
    }

    fn draw_char_big_overlay(&mut self, x: i32, y: i32, ch: u8, color: u32) {
        MinifbVideo::draw_char_big_overlay(self, x, y, ch, color)
    }

    fn frame_drawn(&self) -> bool {
        MinifbVideo::frame_drawn(self)
    }

    fn fb_width(&self) -> u32 {
        MinifbVideo::fb_width(self)
    }

    fn fb_height(&self) -> u32 {
        MinifbVideo::fb_height(self)
    }

    fn fb_bpp(&self) -> u32 {
        MinifbVideo::fb_bpp(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn update_window(&mut self) {
        MinifbVideo::update(self);
    }

    fn should_close(&self) -> bool {
        MinifbVideo::should_close(self)
    }

    #[cfg(feature = "api")]
    fn set_snapshot_callback(
        &mut self,
        _callback: Option<fn(pixels: *const u8, width: u32, height: u32, core_bpp: u32)>,
    ) {
        // Initialize snapshot buffer with current core resolution
        let format = unsafe { crate::video::CORE_FORMAT };
        self.set_snapshot_callback(format.width, format.height);
    }

    #[cfg(feature = "api")]
    fn take_snapshot(&mut self) -> Option<FrameSnapshot> {
        MinifbVideo::take_snapshot(self)
    }
}
