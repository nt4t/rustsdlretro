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
pub static mut MAIN_VIDEO: Option<FbdevVideo> = None;

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

        let frame_w = frame_w as i32;
        let frame_h = frame_h as i32;

        self.frame_drawn = true;
        let fb_bpp = self.fb_bpp;
        let fb_ptr = self.fb_ptr;
        let fb_pitch = self.fb_pitch as usize;
        let offset_x = ((self.fb_width as i32) - frame_w) / 2;
        let offset_y = ((self.fb_height as i32) - frame_h) / 2;

        if fb_bpp == 32 && core_bpp == 32 {
            for y in 0..frame_h {
                let src_row = unsafe { (pixels as *const u8).add((y as usize) * pitch) };
                let row = (offset_y + y) as usize;
                let dest_offset = row * fb_pitch + (offset_x as usize) * 4;
                let dest_row = unsafe { fb_ptr.add(dest_offset) };
                unsafe {
                    ptr::copy_nonoverlapping(src_row, dest_row, (frame_w as usize) * 4);
                }
            }
        } else if fb_bpp == 16 && core_bpp == 16 {
            for y in 0..frame_h {
                let src_row = unsafe { (pixels as *const u8).add((y as usize) * pitch) };
                let row = (offset_y + y) as usize;
                let dest_offset = row * fb_pitch + (offset_x as usize) * 2;
                let dest_row = unsafe { fb_ptr.add(dest_offset) };
                unsafe {
                    ptr::copy_nonoverlapping(src_row, dest_row, (frame_w as usize) * 2);
                }
            }
        } else if fb_bpp == 32 && core_bpp == 16 {
            for y in 0..frame_h {
                let row = (offset_y + y) as usize;
                let dest_offset = row * fb_pitch + (offset_x as usize) * 4;
                let dest_row = unsafe { fb_ptr.add(dest_offset) } as *mut u32;
                let src_row = unsafe { (pixels as *const u16).add((y as usize) * (pitch / 2)) };
                unsafe {
                    for x in 0..frame_w {
                        *dest_row.add(x as usize) = rgb565_to_xrgb8888(*src_row.add(x as usize));
                    }
                }
            }
        } else if fb_bpp == 16 && core_bpp == 32 {
            for y in 0..frame_h {
                let row = (offset_y + y) as usize;
                let dest_offset = row * fb_pitch + (offset_x as usize) * 2;
                let dest_row = unsafe { fb_ptr.add(dest_offset) } as *mut u16;
                let src_row = unsafe { (pixels as *const u32).add((y as usize) * (pitch / 4)) };
                unsafe {
                    for x in 0..frame_w {
                        *dest_row.add(x as usize) = xrgb8888_to_rgb565(*src_row.add(x as usize));
                    }
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
