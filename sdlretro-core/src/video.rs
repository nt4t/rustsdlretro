use libc::{c_int, c_uint, c_ushort, c_void};
use std::ptr;

use std::ffi::CString;

// ioctl constants for fbdev
const FBIOGET_FSCREENINFO: c_uint = 0x4602;
const FBIOGET_VSCREENINFO: c_uint = 0x4600;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_fix_screeninfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: c_int,
    pub type_: c_int,
    pub type_aux: c_int,
    pub visual: c_int,
    pub xpanstep: c_ushort,
    pub ypanstep: c_ushort,
    pub ywrapstep: c_ushort,
    pub line_length: c_uint,
    pub mmio_start: usize,
    pub mmio_len: c_int,
    pub accel: c_int,
    pub capabilities: c_ushort,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_var_screeninfo {
    pub xres: c_uint,
    pub yres: c_uint,
    pub xres_virtual: c_uint,
    pub yres_virtual: c_uint,
    pub xoffset: c_int,
    pub yoffset: c_int,
    pub bits_per_pixel: c_uint,
    pub grayscale: c_uint,
    pub red: fb_bitfield,
    pub green: fb_bitfield,
    pub blue: fb_bitfield,
    pub transp: fb_bitfield,
    pub nonstd: c_uint,
    pub activate: c_uint,
    pub height: c_int,
    pub width: c_int,
    pub accelerate: c_uint,
    pub flags: c_uint,
    pub sync: c_uint,
    pub refresh: c_uint,
    pub omega: c_uint,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_bitfield {
    pub offset: c_uint,
    pub length: c_uint,
    pub lsb_right: c_uint,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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

/// Convert XRGB8888 to RGB565
#[inline]
fn xrgb8888_to_rgb565(src: u32) -> u16 {
    let r = (src >> 16) & 0xFF;
    let g = (src >> 8) & 0xFF;
    let b = src & 0xFF;
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    (r5 << 11) | (g6 << 5) | b5
}

pub static mut CORE_FORMAT: CoreFormat = CoreFormat::UNINITIALIZED;

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

        if unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix) } < 0 {
            unsafe { libc::close(fd) };
            return Err(format!("FBIOGET_FSCREENINFO failed: {}", std::io::Error::last_os_error()));
        }

        if unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var) } < 0 {
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
        })
    }

    pub fn set_core_format(&mut self, width: u32, height: u32, _bpp: u32) {
        self.core_width = width;
        self.core_height = height;
        self.offset_x = ((self.fb_width as i32) - (width as i32)) / 2;
        self.offset_y = ((self.fb_height as i32) - (height as i32)) / 2;
    }

    pub fn push_frame(&mut self, pixels: *const c_void, pitch: usize) {
        if self.skip_frame {
            self.skip_frame = false;
            self.frame_drawn = false;
            return;
        }

        let core_w = self.core_width;
        let core_h = self.core_height;
        
        if core_w == 0 || core_h == 0 {
            return;
        }

        self.frame_drawn = true;
        let fb_pp = (self.fb_bpp / 8) as i32;

        let src = pixels as *const u32;
        
        if self.fb_bpp == 32 {
            // 32-bit framebuffer: direct copy
            for y in 0..core_h {
                let src_row = unsafe { src.add((y as usize) * (pitch / 4)) };
                let row = (self.offset_y + (y as i32)) as usize;
                let col = self.offset_x as usize;
                let dest_offset = row * (self.fb_pitch as usize / 4) + col;
                let dest_row = unsafe { self.fb_ptr.add(dest_offset * 4) as *mut u32 };
                unsafe {
                    ptr::copy_nonoverlapping(src_row, dest_row, core_w as usize);
                }
            }
        } else if self.fb_bpp == 16 {
            // 16-bit framebuffer: XRGB8888 -> RGB565 conversion
            for y in 0..core_h {
                let src_row = unsafe { src.add((y as usize) * (pitch / 4)) };
                let row = (self.offset_y + (y as i32)) as usize;
                let col = self.offset_x as usize;
                let dest_offset = row * (self.fb_pitch as usize / 2) + col;
                let dest_row = unsafe { self.fb_ptr.add(dest_offset * 2) as *mut u16 };
                for x in 0..core_w {
                    let pixel = unsafe { *src_row.add(x as usize) };
                    let rgb565 = xrgb8888_to_rgb565(pixel);
                    unsafe { *dest_row.add(x as usize) = rgb565 };
                }
            }
        }
    }

    pub fn set_skip_frame(&mut self) {
        self.skip_frame = true;
        self.frame_drawn = false;
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

    pub fn fb_bpp(&self) -> u32 {
        self.fb_bpp
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
