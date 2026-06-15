use std::ffi::{CString, CStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::mem::ManuallyDrop;

use libc::{c_void, dlopen, dlsym, dlerror, RTLD_LAZY, dlclose};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod video;
pub mod input;
pub mod audio;
pub mod font;
pub mod core_options;
pub mod gui;

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
    eprintln!("[beetle LOG level={}]", level);
    eprintln!("[beetle] {}", msg);
}

static mut CORE_OPTIONS: Option<core_options::CoreOptions> = None;

pub fn get_core_options_raw() -> Option<&'static core_options::CoreOptions> {
    unsafe { CORE_OPTIONS.as_ref() }
}

extern "C" fn log_environment_cb(key: u32, data: *mut libc::c_void) -> bool {
    if key == 31 {
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
    if key == 13 {
        eprintln!("Core requested system directory");
        return true;
    }
    if key == 52 {
        let version = data as *mut u32;
        if !version.is_null() {
            unsafe {
                *version = core_options::V2_API_VERSION;
                eprintln!("Core options API version: {}", *version);
            }
        }
        return true;
    }
    if key == 53 {
        let defs = data as *mut retro_core_option_definition;
        if !defs.is_null() {
            unsafe {
                let definitions = core_options::parse_v1_definitions(defs);
                eprintln!("Core options (v1): {} options loaded", definitions.len());
                for def in &definitions {
                    eprintln!("  Option: {} = {}", def.key, def.desc);
                    if !def.values.is_empty() {
                        eprintln!("    Values: {:?}", def.values.iter().map(|v| &v.value).collect::<Vec<_>>());
                    }
                    if let Some(ref default) = def.default_value {
                        eprintln!("    Default: {}", default);
                    }
                }
                CORE_OPTIONS = Some(core_options::CoreOptions {
                     supports_v2: false,
                     v2: None,
                     v1: Some(core_options::CoreOptionsV1 { definitions }),
                     old_vars: Vec::new(),
                     old_values: std::collections::HashMap::new(),
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
                    eprintln!("Core options (v2): {} options loaded", definitions.len());
                    for def in &definitions {
                        eprintln!("  Option: {} = {}", def.key, def.desc);
                        if !def.values.is_empty() {
                            eprintln!("    Values: {:?}", def.values.iter().map(|v| &v.value).collect::<Vec<_>>());
                        }
                    }
                   CORE_OPTIONS = Some(core_options::CoreOptions {
                         supports_v2: true,
                         v2: Some(core_options::CoreOptionsV2 {
                             categories: Vec::new(),
                             definitions,
                         }),
                         v1: None,
                         old_vars: Vec::new(),
                         old_values: std::collections::HashMap::new(),
                     });
                }
            }
        }
        return true;
    }
    if key == 55 {
        let display = data as *mut retro_core_option_display;
        if !display.is_null() {
            unsafe {
                let key_ptr = (*display).key;
                let visible = (*display).visible;
                let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
                eprintln!("Core option display: {} visible={}", key, visible);
            }
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
                        eprintln!("Resolution changed: {}x{} @ {:.2} FPS", w, h, fps);
                        unsafe {
                            if !video::MAIN_VIDEO.is_null() {
                                let video = &mut *(video::MAIN_VIDEO as *mut video::FbdevVideo);
                                video.set_core_format(w, h, video::CORE_FORMAT.bpp);
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
                unsafe {
                    if !MAIN_AUDIO.is_null() {
                        let audio = &mut *(MAIN_AUDIO as *mut audio::AudioDriver);
                        if new_sample_rate != audio.sample_rate {
                            eprintln!("Sample rate change requested: {} Hz", new_sample_rate);
                            audio.restart_with_rate(new_sample_rate);
                        }
                    }
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
                        eprintln!("  Old var: {} = {} (values: {:?})", key, title, values);
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
                    } else {
                        eprintln!("Old variables: {} options (no CORE_OPTIONS yet)", old_vars.len());
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
                    if let Some(ref core_opts) = CORE_OPTIONS {
                        if let Some(current_val) = core_opts.get_current_value(&key) {
                            let c_val = CString::new(current_val).unwrap();
                            (*var).value = c_val.as_ptr();
                            std::mem::forget(c_val);
                            eprintln!("GET_VARIABLE: {} = {}", key, (*var).value as *const c_void);
                        }
                    }
                }
            }
        }
        return true;
    }
    eprintln!("ENV CB: unknown key={}", key);
    false
}

static mut SYSTEM_DIR: *const libc::c_char = ptr::null();
static RESOLUTION_STATE: std::sync::OnceLock<Arc<Mutex<ResolutionState>>> = std::sync::OnceLock::new();
pub static mut MAIN_AUDIO: *mut c_void = ptr::null_mut();

pub fn set_resolution_state(state: Arc<Mutex<ResolutionState>>) {
    RESOLUTION_STATE.set(state).ok();
}

pub fn set_system_directory(handle: *mut c_void, dir: &str) {
    let set_env: RetroSetEnvironmentFn = unsafe { get_symbol!(handle, "retro_set_environment", RetroSetEnvironmentFn) };
    let c_dir = CString::new(dir).unwrap();
    unsafe {
        SYSTEM_DIR = c_dir.as_ptr() as *const libc::c_char;
        std::mem::forget(c_dir);
        set_env(Some(system_dir_env_callback));
        eprintln!("System directory set to: {}", dir);
    }
}

extern "C" fn system_dir_env_callback(key: u32, data: *mut libc::c_void) -> bool {
    if key == 13 {
        let dir_ptr = data as *mut *const libc::c_char;
        if !dir_ptr.is_null() {
            unsafe {
                *dir_ptr = SYSTEM_DIR;
            }
        }
        return true;
    }
    false
}

static mut AUDIO_CB_COUNT: usize = 0;
static mut AUDIO_BATCH_CB_COUNT: usize = 0;

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
        if !MAIN_AUDIO.is_null() {
            let audio = &mut *(MAIN_AUDIO as *mut audio::AudioDriver);
            if !data.is_null() && frames > 0 {
                let slice = std::slice::from_raw_parts(data, frames * 2);
                audio.push_batch(slice);
            } else if frames > 0 {
                eprintln!("audio_sample_batch_cb: frames={} but data is null", frames);
            }
        } else {
            eprintln!("audio_sample_batch_cb: MAIN_AUDIO is null!");
        }
        frames
    }
}

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
        Ok(Core {
            handle,
            need_fullpath: false,
            resolution: Arc::new(Mutex::new(ResolutionState::default())),
        })
    }

    pub fn get_resolution_state(&self) -> Arc<Mutex<ResolutionState>> {
        Arc::clone(&self.resolution)
    }

    pub fn get_core_options(&self) -> Option<&core_options::CoreOptions> {
        unsafe { CORE_OPTIONS.as_ref() }
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
        let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            CoreError { message: "Path contains null bytes".into() }
        })?;
        
        eprintln!("Loading ROM: need_fullpath={}, path={}", self.need_fullpath, path.display());
        
        let rom_data = std::fs::read(path).map_err(|e| {
            CoreError { message: format!("Failed to read ROM: {}", e) }
        })?;
        eprintln!("ROM size: {} bytes", rom_data.len());
        
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
}

impl Drop for Core {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dlclose(self.handle) };
            self.handle = ptr::null_mut();
        }
    }
}
