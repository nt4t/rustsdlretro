use std::ffi::{CString, CStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use libc::{c_void, dlopen, dlsym, dlerror, RTLD_LAZY, dlclose};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Core {
    handle: *mut c_void,
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
        Ok(Core { handle })
    }

    pub fn init(&mut self) -> Result<(), CoreError> {
        let get_info: RetroGetSystemInfoFn = unsafe { get_symbol!(self.handle, "retro_get_system_info", RetroGetSystemInfoFn) };
        let mut info = retro_system_info::default();
        unsafe { get_info(&mut info) };
        eprintln!("Core: {}", CStr::from_ptr(info.library_name).to_string_lossy());

        let set_env: RetroSetEnvironmentFn = unsafe { get_symbol!(self.handle, "retro_set_environment", RetroSetEnvironmentFn) };
        unsafe { set_env(Some(dummy_environment_cb)) };

        let init: RetroInitFn = unsafe { get_symbol!(self.handle, "retro_init", RetroInitFn) };
        unsafe { init() };
        eprintln!("Core initialized");

        let set_vr: RetroSetVideoRefreshFn = unsafe { get_symbol!(self.handle, "retro_set_video_refresh", RetroSetVideoRefreshFn) };
        unsafe { set_vr(None) };

        let set_audio: RetroSetAudioSampleFn = unsafe { get_symbol!(self.handle, "retro_set_audio_sample", RetroSetAudioSampleFn) };
        unsafe { set_audio(None) };

        let set_audio_batch: RetroSetAudioSampleBatchFn = unsafe { get_symbol!(self.handle, "retro_set_audio_sample_batch", RetroSetAudioSampleBatchFn) };
        unsafe { set_audio_batch(None) };

        let set_poll: RetroSetInputPollFn = unsafe { get_symbol!(self.handle, "retro_set_input_poll", RetroSetInputPollFn) };
        unsafe { set_poll(None) };

        let set_state: RetroSetInputStateFn = unsafe { get_symbol!(self.handle, "retro_set_input_state", RetroSetInputStateFn) };
        unsafe { set_state(None) };

        Ok(())
    }

    pub fn load_game(&mut self, path: &Path) -> Result<(), CoreError> {
        let load: RetroLoadGameFn = unsafe { get_symbol!(self.handle, "retro_load_game", RetroLoadGameFn) };
        let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            CoreError { message: "Path contains null bytes".into() }
        })?;
        let mut game_info = retro_game_info {
            path: c_path.as_ptr(),
            data: ptr::null(),
            size: 0,
            meta: ptr::null(),
        };
        let success = unsafe { load(&mut game_info) };
        if !success {
            return Err(CoreError { message: "retro_load_game returned false".into() });
        }
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
