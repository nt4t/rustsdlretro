use std::ffi::{CString, CStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use libc::{c_void, dlopen, dlsym, dlerror, RTLD_LAZY, dlclose};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod video;
pub mod input;

pub struct Core {
    handle: *mut c_void,
    need_fullpath: bool,
}

pub type RetroInitFn = unsafe extern "C" fn();
pub type RetroDeinitFn = unsafe extern "C" fn();
pub type RetroSetEnvironmentFn = unsafe extern "C" fn(callback: retro_environment_t);
pub type RetroSetVideoRefreshFn = unsafe extern "C" fn(callback: retro_video_refresh_t);
pub type RetroSetAudioSampleFn = unsafe extern "C" fn(callback: retro_audio_sample_t);
pub type RetroSetAudioSampleBatchFn = unsafe extern "C" fn(callback: retro_audio_sample_batch_t);
pub type RetroSetInputPollFn = unsafe extern "C" fn(callback: retro_input_poll_t);
pub type RetroSetInputStateFn = unsafe extern "C" fn(callback: retro_input_state_t);
pub type RetroLoadGameFn = unsafe extern "C" fn(game: *const retro_game_info) -> bool;
pub type RetroUnloadGameFn = unsafe extern "C" fn();
pub type RetroRunFn = unsafe extern "C" fn();
pub type RetroGetSystemInfoFn = unsafe extern "C" fn(info: *mut retro_system_info);
pub type RetroGetSystemAvInfoFn = unsafe extern "C" fn(av_info: *mut retro_system_av_info);

#[derive(Debug)]
pub struct CoreError {
    pub message: String,
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CoreError: {}", self.message)
    }
}

impl std::error::Error for CoreError {}

unsafe fn get_symbol_ptr(handle: *mut c_void, name: &str) -> *mut c_void {
    let c_name = CString::new(name).unwrap();
    let sym = dlsym(handle, c_name.as_ptr());
    if sym.is_null() {
        let err_str = dlerror();
        let msg = CStr::from_ptr(err_str).to_string_lossy().into_owned();
        panic!("Core missing symbol {}: {}", name, msg);
    }
    sym
}

macro_rules! get_symbol {
    ($handle:expr, $name:expr, $ty:ty) => {{
        let ptr = unsafe { get_symbol_ptr($handle, $name) };
        unsafe { std::mem::transmute::<*mut c_void, $ty>(ptr) }
    }};
}

extern "C" fn dummy_environment_cb(_key: u32, _data: *mut c_void) -> bool {
    false
}

extern "C" fn log_callback(level: u32, message: *const libc::c_char) {
    let msg = unsafe { CStr::from_ptr(message).to_string_lossy().into_owned() };
    eprintln!("[beetle] {}", msg);
}

extern "C" fn log_environment_cb(key: u32, data: *mut libc::c_void) -> bool {
    eprintln!("Environment callback: key={}", key);
    if key == 33 {
        let log_info = data as *mut retro_log_callback;
        if !log_info.is_null() {
            unsafe {
                let fn_ptr: retro_log_printf_t = std::mem::transmute(log_callback as *const c_void);
                (*log_info).log = fn_ptr;
                eprintln!("Log interface registered");
            }
        }
        return true;
    }
    if key == 10 {
        let info = data as *mut retro_pixel_format;
        if !info.is_null() {
            let format = unsafe { *info };
            let (name, bpp) = match format {
                0 => ("0RGB8888", 32),
                1 => ("XRGB8888", 32),
                2 => ("RGB565", 16),
                _ => ("UNKNOWN", 0),
            };
            unsafe {
                video::CORE_FORMAT.bpp = bpp as u32;
                eprintln!("Video: pixel format set to {} (id={})", name, format);
            }
        }
        return true;
    }
    false
}

extern "C" fn audio_sample_cb(_left: i16, _right: i16) {
}

extern "C" fn audio_sample_batch_cb(_data: *const i16, _frames: usize) -> usize {
    0
}

pub struct Throttle {
    frame_time: u64,
    next_frame: u64,
}

impl Throttle {
    pub fn new(fps: f64) -> Self {
        let frame_time = (1_000_000.0 / fps) as u64;
        Self {
            frame_time,
            next_frame: now_usec(),
        }
    }

    pub fn check_wait(&mut self) -> i64 {
        let now = now_usec();
        let result = self.next_frame as i64 - now as i64;
        if result > 0 {
            result
        } else {
            self.next_frame += self.frame_time;
            result
        }
    }

    pub fn skip_check(&mut self) {
        self.next_frame += self.frame_time;
    }
}

fn now_usec() -> u64 {
    let mut ts: libc::timespec = unsafe { std::mem::zeroed() };
    unsafe {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
    }
    ts.tv_sec as u64 * 1_000_000 + ts.tv_nsec as u64 / 1_000
}

impl Core {
    pub fn new(path: &Path) -> Result<Self, CoreError> {
        let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            CoreError { message: "Path contains null bytes".into() }
        })?;
        let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_LAZY) };
        if handle.is_null() {
            let err_str = unsafe { dlerror() };
            let msg = unsafe { CStr::from_ptr(err_str) }.to_string_lossy().into_owned();
            return Err(CoreError { message: format!("dlopen: {}", msg) });
        }
        Ok(Core { handle, need_fullpath: false })
    }

    pub fn init(&mut self) -> Result<(), CoreError> {
        let get_info: RetroGetSystemInfoFn = unsafe { get_symbol!(self.handle, "retro_get_system_info", RetroGetSystemInfoFn) };
        let mut info = retro_system_info::default();
        unsafe { get_info(&mut info) };
        self.need_fullpath = info.need_fullpath;
        eprintln!("Core: {} need_fullpath={}", unsafe { CStr::from_ptr(info.library_name).to_string_lossy() }, self.need_fullpath);

        let set_env: RetroSetEnvironmentFn = unsafe { get_symbol!(self.handle, "retro_set_environment", RetroSetEnvironmentFn) };
        unsafe { set_env(Some(log_environment_cb)) };

        let init: RetroInitFn = unsafe { get_symbol!(self.handle, "retro_init", RetroInitFn) };
        unsafe { init() };
        eprintln!("Core initialized");

        let set_vr: RetroSetVideoRefreshFn = unsafe { get_symbol!(self.handle, "retro_set_video_refresh", RetroSetVideoRefreshFn) };
        unsafe { set_vr(None) };

        let set_audio: RetroSetAudioSampleFn = unsafe { get_symbol!(self.handle, "retro_set_audio_sample", RetroSetAudioSampleFn) };
        unsafe { set_audio(Some(audio_sample_cb)) };

        let set_audio_batch: RetroSetAudioSampleBatchFn = unsafe { get_symbol!(self.handle, "retro_set_audio_sample_batch", RetroSetAudioSampleBatchFn) };
        unsafe { set_audio_batch(Some(audio_sample_batch_cb)) };

        Ok(())
    }

    pub fn set_callbacks(&self, video_cb: retro_video_refresh_t, poll_cb: retro_input_poll_t, state_cb: retro_input_state_t) {
        let set_vr: RetroSetVideoRefreshFn = unsafe { get_symbol!(self.handle, "retro_set_video_refresh", RetroSetVideoRefreshFn) };
        unsafe { set_vr(video_cb) };
        let set_poll: RetroSetInputPollFn = unsafe { get_symbol!(self.handle, "retro_set_input_poll", RetroSetInputPollFn) };
        unsafe { set_poll(poll_cb) };
        let set_state: RetroSetInputStateFn = unsafe { get_symbol!(self.handle, "retro_set_input_state", RetroSetInputStateFn) };
        unsafe { set_state(state_cb) };
    }

    pub fn set_video_refresh(&self, video_cb: retro_video_refresh_t) {
        let set_vr: RetroSetVideoRefreshFn = unsafe { get_symbol!(self.handle, "retro_set_video_refresh", RetroSetVideoRefreshFn) };
        unsafe { set_vr(video_cb) };
    }

    pub fn load_game(&mut self, path: &Path) -> Result<(), CoreError> {
        let load: RetroLoadGameFn = unsafe { get_symbol!(self.handle, "retro_load_game", RetroLoadGameFn) };
        let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            CoreError { message: "Path contains null bytes".into() }
        })?;
        
        eprintln!("Loading ROM: need_fullpath={}, path={}", self.need_fullpath, path.display());
        
        let mut game_info = if self.need_fullpath {
            retro_game_info {
                path: c_path.as_ptr(),
                data: ptr::null(),
                size: 0,
                meta: ptr::null(),
            }
        } else {
            let rom_data = std::fs::read(path).map_err(|e| {
                CoreError { message: format!("Failed to read ROM: {}", e) }
            })?;
            eprintln!("ROM size: {} bytes", rom_data.len());
            let game_info = retro_game_info {
                path: ptr::null(),
                data: rom_data.as_ptr() as *const c_void,
                size: rom_data.len(),
                meta: ptr::null(),
            };
            std::mem::forget(rom_data);
            game_info
        };
        
        let success = unsafe { load(&mut game_info) };
        if !success {
            return Err(CoreError { message: "retro_load_game returned false".into() });
        }
        eprintln!("Load OK");
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CoreError> {
        let run: RetroRunFn = unsafe { get_symbol!(self.handle, "retro_run", RetroRunFn) };
        unsafe { run() };
        Ok(())
    }

    pub fn unload_game(&mut self) {
        let unload: RetroUnloadGameFn = unsafe { get_symbol!(self.handle, "retro_unload_game", RetroUnloadGameFn) };
        unsafe { unload() };
    }

    pub fn get_system_av_info(&self) -> retro_system_av_info {
        let mut av_info = retro_system_av_info::default();
        let get_av_info: RetroGetSystemAvInfoFn = unsafe { get_symbol!(self.handle, "retro_get_system_av_info", RetroGetSystemAvInfoFn) };
        unsafe { get_av_info(&mut av_info) };
        av_info
    }

    pub fn unload(&mut self) {
        self.unload_game();
        let deinit: RetroDeinitFn = unsafe { get_symbol!(self.handle, "retro_deinit", RetroDeinitFn) };
        unsafe { deinit() };
        if !self.handle.is_null() {
            unsafe { dlclose(self.handle) };
            self.handle = ptr::null_mut();
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dlclose(self.handle) };
            self.handle = ptr::null_mut();
        }
    }
}
