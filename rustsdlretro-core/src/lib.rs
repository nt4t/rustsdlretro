use std::ffi::{CString, CStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::mem::ManuallyDrop;

pub use video::VideoBackend;

use libc::{c_void, dlopen, dlsym, dlerror, RTLD_LAZY, dlclose};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod video;
pub mod input;
pub mod audio;
#[cfg(feature = "null-audio")]
pub mod audio_null;
pub mod font;
pub mod core_options;
pub mod gui;
pub mod sram;
pub mod zip_rom;

#[cfg(feature = "minifb")]
pub mod video_minifb;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "api")]
pub mod api;

pub struct ResolutionState {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

impl Default for ResolutionState {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            fps: 0.0,
        }
    }
}

pub struct Core {
    handle: *mut c_void,
    need_fullpath: bool,
    core_name: String,
    resolution: Arc<Mutex<ResolutionState>>,
}

impl Core {
    pub fn handle(&self) -> *mut c_void {
        self.handle
    }
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
pub type RetroSerializeSizeFn = unsafe extern "C" fn() -> usize;
pub type RetroSerializeFn = unsafe extern "C" fn(data: *mut c_void, len: usize) -> bool;
pub type RetroUnserializeFn = unsafe extern "C" fn(data: *const c_void, len: usize) -> bool;

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

/// C shim: receives variadic format args from core, expands them via vsnprintf,
/// then calls rust_log_callback with the formatted string.
extern "C" {
    fn rustsdlretro_log_handler(level: u32, fmt: *const libc::c_char, ...);
}

#[allow(dead_code)]
extern "C" fn dummy_environment_cb(_key: u32, _data: *mut c_void) -> bool {
    false
}



/// Rust log handler — called from C shim with pre-formatted string.
#[no_mangle]
extern "C" fn rust_log_callback(level: u32, message: *const libc::c_char) {
    let msg = unsafe { CStr::from_ptr(message).to_string_lossy().into_owned() };
    eprintln!("[LOG level={}]", level);
    if !msg.is_empty() {
        eprintln!("{}", msg);
    }
}

/// Fallback log handler (non-variadic, format strings won't expand).
unsafe extern "C" fn log_callback(level: u32, message: *const libc::c_char) {
    let msg = CStr::from_ptr(message).to_string_lossy().into_owned();
    eprintln!("[LOG level={}]", level);
    if !msg.is_empty() {
        eprintln!("{}", msg);
    }
}

static mut CORE_OPTIONS: Option<core_options::CoreOptions> = None;
static mut VARIABLE_UPDATE_PENDING: bool = false;

pub fn get_core_options_raw() -> Option<&'static core_options::CoreOptions> {
    unsafe { CORE_OPTIONS.as_ref() }
}

pub fn get_core_options_raw_mut() -> Option<&'static mut core_options::CoreOptions> {
    unsafe { CORE_OPTIONS.as_mut() }
}

// Unified environment callback that handles both logging and system directory
extern "C" fn log_environment_cb(key: u32, data: *mut libc::c_void) -> bool {
    // Key 27 = RETRO_ENVIRONMENT_GET_LOG_INTERFACE
    if key == 27 {
        let log_info = data as *mut retro_log_callback;
        if !log_info.is_null() {
            unsafe {
                // Point the core to our C shim which expands variadic format strings,
                // then calls rust_log_callback with the pre-formatted message.
                (*log_info).log = Some(rustsdlretro_log_handler);
            }
        }
        return true;
    }
    // Key 9 = RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY
    if key == 9 {
        let dir_ptr = data as *mut *const libc::c_char;
        if !dir_ptr.is_null() {
            unsafe {
                *dir_ptr = SYSTEM_DIR;
            }
        }
        return true;
    }
    if key == 10 {
        let info = data as *mut retro_pixel_format;
        if !info.is_null() {
            let format = unsafe { *info };
            let (_name, bpp) = match format {
                0 => ("0RGB8888", 32),
                1 => ("XRGB8888", 32),
                2 => ("RGB565", 16),
                _ => ("UNKNOWN", 0),
            };
            unsafe {
                video::CORE_FORMAT.bpp = bpp as u32;
            }
        }
        return true;
    }

    if key == 52 {
        let version = data as *mut u32;
        if !version.is_null() {
            unsafe {
                *version = core_options::V2_API_VERSION;
            }
        }
        return true;
    }
    if key == 53 {
        let defs = data as *mut retro_core_option_definition;
        if !defs.is_null() {
            unsafe {
                let definitions = core_options::parse_v1_definitions(defs);
CORE_OPTIONS = Some(core_options::CoreOptions {
                      supports_v2: false,
                      v2: None,
                      v1: Some(core_options::CoreOptionsV1 { definitions, values: std::collections::HashMap::new() }),
                      old_vars: Vec::new(),
                      old_values: std::collections::HashMap::new(),
                      v2_values: std::collections::HashMap::new(),
                  });
            }
        }
        return true;
    }
    if key == 67 {
        let v2_opts = data;
        if !v2_opts.is_null() {
            unsafe {
                let v2_ptr = v2_opts as *mut retro_core_options_v2;
                if !v2_ptr.is_null() {
                    let definitions = core_options::parse_v2_definitions((*v2_ptr).definitions);
CORE_OPTIONS = Some(core_options::CoreOptions {
                          supports_v2: true,
                          v2: Some(core_options::CoreOptionsV2 {
                              categories: Vec::new(),
                              definitions,
                          }),
                          v1: None,
                          old_vars: Vec::new(),
                          old_values: std::collections::HashMap::new(),
                          v2_values: std::collections::HashMap::new(),
                      });
                }
            }
        }
        return true;
    }
    if key == 68 {
        // SET_CORE_OPTIONS_V2_INTL
        let intl_opts = data;
        if !intl_opts.is_null() {
            unsafe {
                let intl_ptr = intl_opts as *mut retro_core_options_v2_intl;
                if !intl_ptr.is_null() {
                    let us_ptr = (*intl_ptr).us;
                    if !us_ptr.is_null() {
                        let definitions = core_options::parse_v2_definitions((*us_ptr).definitions);
                        CORE_OPTIONS = Some(core_options::CoreOptions {
                            supports_v2: true,
                            v2: Some(core_options::CoreOptionsV2 {
                                categories: Vec::new(),
                                definitions,
                            }),
                            v1: None,
                            old_vars: Vec::new(),
                            old_values: std::collections::HashMap::new(),
                            v2_values: std::collections::HashMap::new(),
                        });
                    }
                }
            }
        }
        return true;
    }
    if key == 55 {
        let display = data as *mut retro_core_option_display;
        if !display.is_null() {
            // Core option display toggle - no logging
        }
        return true;
    }
    if key == 32 || key == 37 {
        let geom = data as *mut retro_game_geometry;
        if !geom.is_null() {
            let geometry = unsafe { *geom };
            let w = geometry.base_width;
            let h = geometry.base_height;
            let fps = unsafe {
                if key == 32 {
                    let av_info = data as *mut retro_system_av_info;
                    if !av_info.is_null() {
                        (*av_info).timing.fps
                    } else {
                        60.0
                    }
                } else {
                    60.0
                }
            };
            unsafe {
                if let Some(state) = RESOLUTION_STATE.get() {
                    let mut s = state.lock().unwrap();
                    let changed = s.width != w || s.height != h || s.fps != fps;
                    s.width = w;
                    s.height = h;
                    s.fps = fps;
                    drop(s);

                    if changed {
                        unsafe {
                            if let Some(ref mut v) = MAIN_VIDEO {
                                (*v).set_core_format(w, h, video::CORE_FORMAT.bpp);
                            }
                        }
                    }
                }
            }
        }
        if key == 32 {
            let av_info = data as *mut retro_system_av_info;
            if !av_info.is_null() {
                let new_sample_rate = unsafe { (*av_info).timing.sample_rate } as u32;
                // Queue the rate change instead of applying immediately during callback
                // This avoids memory corruption when called from within retro_run()
                if let Ok(queue) = AUDIO_RATE_CHANGE_QUEUE.lock() {
                    queue.set(Some(new_sample_rate));
                }
            }
        }
        return true;
    }
    if key == 16 {
        // SET_VARIABLES - old-style core options
        let vars = data as *mut retro_variable;
        if !vars.is_null() {
            unsafe {
                let mut old_vars = Vec::new();
                let mut i = 0;
                loop {
                    let var = vars.add(i);
                    let key_ptr = (*var).key;
                    let value_ptr = (*var).value;
                    if key_ptr.is_null() {
                        break;
                    }
                    let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
                    if value_ptr.is_null() {
                        i += 1;
                        continue;
                    }
                    let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
                    if let Some((title, values)) = core_options::parse_old_variable_string(&value) {
                        let default_index = values.iter().position(|v| v == &values[0]).unwrap_or(0);
                        old_vars.push(core_options::OldVariable {
                            key: key.clone(),
                            title,
                            values: values.clone(),
                            default_index,
                        });
                        // Set initial value to default
                        if let Some(ref mut core_opts) = CORE_OPTIONS {
                            core_opts.set_old_value(&key, &values[0]);
                        }
                    }
                    i += 1;
                    if i > 256 {
                        break;
                    }
                }
                 if !old_vars.is_empty() {
                    if let Some(ref mut core_opts) = CORE_OPTIONS {
                        core_opts.old_vars = old_vars;
                    }
                }
            }
        }
        return true;
    }
    if key == 15 {
        // GET_VARIABLE - get current value of an old-style variable
        let var = data as *mut retro_variable;
        if !var.is_null() {
            unsafe {
                let key_ptr = (*var).key;
                if !key_ptr.is_null() {
                    let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
                    // Check v2_values first, then old_values, then defaults
                    let current_val = if let Some(ref core_opts) = CORE_OPTIONS {
                        core_opts.get_current_value(&key)
                    } else {
                        None
                    };
                    if let Some(ref val) = current_val {
                        let c_val = CString::new(val.as_str()).unwrap();
                        (*var).value = c_val.as_ptr();
                        std::mem::forget(c_val);
                    }
                }
            }
        }
        return true;
    }
    if key == 17 {
        // GET_VARIABLE_UPDATE
        let update = data as *mut bool;
        if !update.is_null() {
            unsafe {
                *update = VARIABLE_UPDATE_PENDING;
                VARIABLE_UPDATE_PENDING = false;
            }
        }
        return true;
    }
    false
}

// Queue for deferred audio sample rate changes (avoid modifying during retro_run callback)
pub static AUDIO_RATE_CHANGE_QUEUE: std::sync::Mutex<std::cell::Cell<Option<u32>>> = 
    std::sync::Mutex::new(std::cell::Cell::new(None));

static mut SYSTEM_DIR: *const libc::c_char = ptr::null();
static RESOLUTION_STATE: std::sync::OnceLock<Arc<Mutex<ResolutionState>>> = std::sync::OnceLock::new();
pub static mut MAIN_AUDIO: *mut c_void = ptr::null_mut();
// Video backend stored as a raw pointer to a boxed trait object
// Only accessed from main thread, no synchronization needed
pub static mut MAIN_VIDEO: Option<ManuallyDrop<Box<dyn video::VideoBackend>>> = None;

pub fn set_resolution_state(state: Arc<Mutex<ResolutionState>>) {
    RESOLUTION_STATE.set(state).ok();
}

pub fn set_system_directory(handle: *mut c_void, dir: &str) {
    let _set_env: RetroSetEnvironmentFn = unsafe { get_symbol!(handle, "retro_set_environment", RetroSetEnvironmentFn) };
    let c_dir = CString::new(dir).unwrap();
   unsafe {
        SYSTEM_DIR = c_dir.as_ptr() as *const libc::c_char;
        std::mem::forget(c_dir);
        // Note: We don't register the callback here anymore.
        // The unified log_environment_cb (registered in core.init()) handles both logging and system directory.
    }
}

static mut AUDIO_CB_COUNT: usize = 0;
pub static mut AUDIO_BATCH_CB_COUNT: usize = 0;

extern "C" fn audio_sample_cb(left: i16, right: i16) {
    unsafe {
        AUDIO_CB_COUNT += 1;
        if AUDIO_CB_COUNT == 1 || AUDIO_CB_COUNT % 10000 == 0 {
            eprintln!("audio_sample_cb called {} times, left={}, right={}", AUDIO_CB_COUNT, left, right);
        }
        if !MAIN_AUDIO.is_null() {
            let audio = &mut *(MAIN_AUDIO as *mut audio::AudioDriver);
            audio.push_stereo_pair(left, right);
        }
    }
}

extern "C" fn audio_sample_batch_cb(data: *const i16, frames: usize) -> usize {
    unsafe {
        AUDIO_BATCH_CB_COUNT += 1;
        if AUDIO_BATCH_CB_COUNT == 1 {
            eprintln!("audio_sample_batch_cb: FIRST CALL frames={} data_ptr={:?} MAIN_AUDIO_ptr={:?}", 
                frames, data, MAIN_AUDIO);
        }
        if !MAIN_AUDIO.is_null() && !data.is_null() && frames > 0 {
            let slice = std::slice::from_raw_parts(data, frames * 2);
            #[cfg(feature = "null-audio")]
            {
                let audio = &mut *(MAIN_AUDIO as *mut audio_null::NullAudioDriver);
                audio.push_batch(slice);
            }
            #[cfg(not(feature = "null-audio"))]
            {
                let audio = &mut *(MAIN_AUDIO as *mut audio::AudioDriver);
                audio.push_batch(slice);
            }
            AUDIO_SAMPLES_PER_FRAME += frames * 2;
        } else if frames > 0 {
            eprintln!("audio_sample_batch_cb: frames={} but data is null", frames);
        }
        frames
    }
}

pub static mut AUDIO_SAMPLES_PER_FRAME: usize = 0;

pub struct Throttle {
    frame_time: u64,
    next_frame: u64,
}

impl Throttle {
    pub fn frame_time(&self) -> u64 {
        self.frame_time
    }

    pub fn new(fps: f64) -> Self {
        let frame_time = (1_000_000.0 / fps) as u64;
        Self {
            frame_time,
            next_frame: now_usec() + frame_time,
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
        Ok(Core {
            handle,
            need_fullpath: false,
            core_name: String::new(),
            resolution: Arc::new(Mutex::new(ResolutionState::default())),
        })
    }

    pub fn get_resolution_state(&self) -> Arc<Mutex<ResolutionState>> {
        Arc::clone(&self.resolution)
    }

    pub fn get_core_options(&self) -> Option<&core_options::CoreOptions> {
        unsafe { CORE_OPTIONS.as_ref() }
    }

    /// Get the libretro core name (e.g., "Beetle PSX", "snes9x2010")
    pub fn get_core_name(&self) -> &str {
        &self.core_name
    }

    pub fn init(&mut self) -> Result<(), CoreError> {
        let _set_env: RetroSetEnvironmentFn = unsafe { get_symbol!(self.handle, "retro_set_environment", RetroSetEnvironmentFn) };
        unsafe { _set_env(Some(log_environment_cb)) };
        eprintln!("Environment callback registered");

        let get_info: RetroGetSystemInfoFn = unsafe { get_symbol!(self.handle, "retro_get_system_info", RetroGetSystemInfoFn) };
        let mut info = retro_system_info::default();
        unsafe { get_info(&mut info) };
        self.need_fullpath = info.need_fullpath;
        self.core_name = unsafe { CStr::from_ptr(info.library_name).to_string_lossy().into_owned() };
        eprintln!("Core: {} need_fullpath={}", self.core_name, self.need_fullpath);

        let init: RetroInitFn = unsafe { get_symbol!(self.handle, "retro_init", RetroInitFn) };
        unsafe { init() };
        eprintln!("Core initialized");

        let set_vr: RetroSetVideoRefreshFn = unsafe { get_symbol!(self.handle, "retro_set_video_refresh", RetroSetVideoRefreshFn) };
        unsafe { set_vr(None) };

        let set_audio: RetroSetAudioSampleFn = unsafe { get_symbol!(self.handle, "retro_set_audio_sample", RetroSetAudioSampleFn) };
        unsafe { set_audio(Some(audio_sample_cb)) };
        eprintln!("Audio sample callback registered");

        let set_audio_batch: RetroSetAudioSampleBatchFn = unsafe { get_symbol!(self.handle, "retro_set_audio_sample_batch", RetroSetAudioSampleBatchFn) };
        unsafe { set_audio_batch(Some(audio_sample_batch_cb)) };
        eprintln!("Audio sample batch callback registered");

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

        eprintln!("Loading ROM: need_fullpath={}, path={}", self.need_fullpath, path.display());

        // Handle ZIP files
        if zip_rom::is_zip(path) {
            return self.load_game_from_zip(&load, path);
        }

        // Regular file: read directly into memory buffer
        let rom_data = std::fs::read(path).map_err(|e| {
            CoreError { message: format!("Failed to read ROM: {}", e) }
        })?;
        eprintln!("ROM size: {} bytes", rom_data.len());

        let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            CoreError { message: "Path contains null bytes".into() }
        })?;

        let mut game_info = retro_game_info {
            path: c_path.as_ptr(),
            data: rom_data.as_ptr() as *const c_void,
            size: rom_data.len(),
            meta: ptr::null(),
        };
        std::mem::forget(rom_data);

        let success = unsafe { load(&mut game_info) };
        if !success {
            return Err(CoreError { message: "retro_load_game returned false".into() });
        }
        eprintln!("Load OK");
        Ok(())
    }

    fn load_game_from_zip(&self, load: &RetroLoadGameFn, path: &Path) -> Result<(), CoreError> {
        eprintln!("ZIP archive detected, extracting ROM...");

        if self.need_fullpath {
            // Cores that need fullpath: extract to temp file
            let (temp_path, _guard) = zip_rom::extract_zip_to_temp(path)
                .map_err(|e| CoreError { message: format!("ZIP extraction failed: {}", e) })?;

            eprintln!("Extracted ROM to temp file: {}", temp_path.display());

            let c_path = CString::new(temp_path.as_os_str().as_bytes()).map_err(|_| {
                CoreError { message: "Temp path contains null bytes".into() }
            })?;

            // Read the extracted ROM for the data pointer (some cores want both)
            let rom_data = std::fs::read(&temp_path).map_err(|e| {
                CoreError { message: format!("Failed to read temp ROM: {}", e) }
            })?;

            let mut game_info = retro_game_info {
                path: c_path.as_ptr(),
                data: rom_data.as_ptr() as *const c_void,
                size: rom_data.len(),
                meta: ptr::null(),
            };
            // Leak both the CString and rom_data so they persist for the core
            std::mem::forget(c_path);
            std::mem::forget(rom_data);

            let success = unsafe { load(&mut game_info) };
            if !success {
                return Err(CoreError { message: "retro_load_game returned false".into() });
            }
        } else {
            // Cores that don't need fullpath: extract to memory buffer
            let (rom_data, filename) = zip_rom::extract_zip_to_memory(path)
                .map_err(|e| CoreError { message: format!("ZIP extraction failed: {}", e) })?;

            eprintln!("Extracted ROM '{}' from ZIP ({} bytes)", filename, rom_data.len());

            // Use a synthetic path for logging; core won't access it since need_fullpath=false
            let c_path = CString::new("zip://archive.zip/rom_file").map_err(|_| {
                CoreError { message: "Path contains null bytes".into() }
            })?;

            let mut game_info = retro_game_info {
                path: c_path.as_ptr(),
                data: rom_data.as_ptr() as *const c_void,
                size: rom_data.len(),
                meta: ptr::null(),
            };
            std::mem::forget(c_path);
            std::mem::forget(rom_data);

            let success = unsafe { load(&mut game_info) };
            if !success {
                return Err(CoreError { message: "retro_load_game returned false".into() });
            }
        }

        eprintln!("Load OK (from ZIP)");
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CoreError> {
        let run: RetroRunFn = unsafe { get_symbol!(self.handle, "retro_run", RetroRunFn) };
        unsafe { run() };
        Ok(())
    }

    pub fn run_and_log(&mut self, frame_count: u64) -> Result<(), CoreError> {
        if frame_count % 300 == 0 {
            eprintln!("retro_run called {} times", frame_count);
        }
        self.run()
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

    /// Serialize the core's internal state into a buffer.
    /// Returns the serialized data on success, or an error if serialization is unsupported/failed.
    pub fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        let serialize_size: RetroSerializeSizeFn = unsafe {
            get_symbol!(self.handle, "retro_serialize_size", RetroSerializeSizeFn)
        };
        let serialize: RetroSerializeFn = unsafe {
            get_symbol!(self.handle, "retro_serialize", RetroSerializeFn)
        };

        let size = unsafe { serialize_size() };
        if size == 0 {
            return Err(CoreError {
                message: "Core reports zero serialization size (not supported?)".into(),
            });
        }

        eprintln!("Saving state: {} bytes", size);
        let mut buffer = vec![0u8; size];
        let success = unsafe { serialize(buffer.as_mut_ptr() as *mut c_void, size) };

        if !success {
            return Err(CoreError {
                message: format!("retro_serialize failed (size={})", size),
            });
        }

        Ok(buffer)
    }

    /// Load a serialized state from a buffer.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), CoreError> {
        let unserialize: RetroUnserializeFn = unsafe {
            get_symbol!(self.handle, "retro_unserialize", RetroUnserializeFn)
        };
        let success = unsafe { unserialize(data.as_ptr() as *const c_void, data.len()) };

        if !success {
            return Err(CoreError {
                message: "retro_unserialize failed (state may be incompatible)".into(),
            });
        }

        Ok(())
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
