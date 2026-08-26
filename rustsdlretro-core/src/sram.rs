use std::ffi::{c_void, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::CoreError;

// libretro memory type constants (from libretro.h)
pub const RETRO_MEMORY_SAVE_RAM: u32 = 0;
pub const RETRO_MEMORY_RTC: u32 = 1;

/// Get the save directory for a specific core.
pub fn get_save_dir(system_dir: &Path, core_name: &str) -> PathBuf {
    system_dir.join("saves").join(core_name)
}

/// Build full path to a state file.
pub fn state_path(save_dir: &Path, game_name: &str) -> PathBuf {
    save_dir.join(format!("{}.state", game_name))
}

/// Build full path to an SRAM file.
pub fn sram_path(save_dir: &Path, game_name: &str) -> PathBuf {
    save_dir.join(format!("{}.sav", game_name))
}

/// Build full path to an RTC file.
pub fn rtc_path(save_dir: &Path, game_name: &str) -> PathBuf {
    save_dir.join(format!("{}.rtc", game_name))
}

/// Ensure the save directory exists for a core.
pub fn ensure_save_dir(system_dir: &Path, core_name: &str) -> Result<PathBuf, CoreError> {
    let dir = get_save_dir(system_dir, core_name);
    std::fs::create_dir_all(&dir).map_err(|e| CoreError {
        message: format!("Failed to create save directory {}: {}", dir.display(), e),
    })?;
    Ok(dir)
}

/// Save SRAM data from the core into a .sav file.
/// Uses retro_get_memory_data(RETRO_MEMORY_SAVE_RAM) and retro_get_memory_size().
pub fn save_sram(core_handle: *mut c_void, game_name: &str, save_dir: &Path) -> Result<(), CoreError> {
    let size_fn = get_memory_size_fn(core_handle)?;
    let data_fn = get_memory_data_fn(core_handle)?;

    // Try SAVE_RAM first
    let sram_size = unsafe { size_fn(RETRO_MEMORY_SAVE_RAM) };
    if sram_size > 0 {
        eprintln!("Saving SRAM: {} bytes", sram_size);
        let data_ptr = unsafe { data_fn(RETRO_MEMORY_SAVE_RAM) };
        if !data_ptr.is_null() {
            let data = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, sram_size as usize) };
            std::fs::write(sram_path(save_dir, game_name), data).map_err(|e| CoreError {
                message: format!("Failed to write SRAM file: {}", e),
            })?;
        }
    }

    // Try RTC (real-time clock) if SAVE_RAM was 0 or also exists
    let rtc_size = unsafe { size_fn(RETRO_MEMORY_RTC) };
    if rtc_size > 0 {
        eprintln!("Saving RTC: {} bytes", rtc_size);
        let data_ptr = unsafe { data_fn(RETRO_MEMORY_RTC) };
        if !data_ptr.is_null() {
            let data = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, rtc_size as usize) };
            std::fs::write(rtc_path(save_dir, game_name), data).map_err(|e| CoreError {
                message: format!("Failed to write RTC file: {}", e),
            })?;
        }
    }

    Ok(())
}

/// Load SRAM/RTC data from disk into the core.
pub fn load_sram(core_handle: *mut c_void, game_name: &str, save_dir: &Path) -> Result<(), CoreError> {
    let set_fn = get_memory_data_setter_fn(core_handle)?;

    // Try loading SRAM
    if sram_path(save_dir, game_name).exists() {
        let data = std::fs::read(sram_path(save_dir, game_name)).map_err(|e| CoreError {
            message: format!("Failed to read SRAM file: {}", e),
        })?;

        // Verify size matches what the core expects
        let sram_size = unsafe { (get_memory_size_fn(core_handle)?)(RETRO_MEMORY_SAVE_RAM) };
        if data.len() == sram_size as usize && !data.is_empty() {
            eprintln!("Loading SRAM: {} bytes", data.len());
            let ptr = unsafe { set_fn(RETRO_MEMORY_SAVE_RAM) };
            if !ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
                }
            }
        } else if sram_size > 0 {
            eprintln!(
                "SRAM size mismatch: file={} expected={}. Skipping.",
                data.len(),
                sram_size
            );
        }
    }

    // Try loading RTC
    if rtc_path(save_dir, game_name).exists() {
        let data = std::fs::read(rtc_path(save_dir, game_name)).map_err(|e| CoreError {
            message: format!("Failed to read RTC file: {}", e),
        })?;

        let rtc_size = unsafe { (get_memory_size_fn(core_handle)?)(RETRO_MEMORY_RTC) };
        if data.len() == rtc_size as usize && !data.is_empty() {
            eprintln!("Loading RTC: {} bytes", data.len());
            let ptr = unsafe { set_fn(RETRO_MEMORY_RTC) };
            if !ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
                }
            }
        } else if rtc_size > 0 {
            eprintln!(
                "RTC size mismatch: file={} expected={}. Skipping.",
                data.len(),
                rtc_size
            );
        }
    }

    Ok(())
}

// FFI function pointer types for memory access
type RetroGetMemorySizeFn = unsafe extern "C" fn(id: u32) -> usize;
type RetroGetMemoryDataFn = unsafe extern "C" fn(id: u32) -> *mut c_void;

fn get_memory_size_fn(handle: *mut c_void) -> Result<RetroGetMemorySizeFn, CoreError> {
    let name = CString::new("retro_get_memory_size").unwrap();
    let sym = unsafe { libc::dlsym(handle, name.as_ptr()) };
    if sym.is_null() {
        return Err(CoreError {
            message: "Core missing retro_get_memory_size".into(),
        });
    }
    Ok(unsafe { std::mem::transmute(sym) })
}

fn get_memory_data_fn(handle: *mut c_void) -> Result<RetroGetMemoryDataFn, CoreError> {
    let name = CString::new("retro_get_memory_data").unwrap();
    let sym = unsafe { libc::dlsym(handle, name.as_ptr()) };
    if sym.is_null() {
        return Err(CoreError {
            message: "Core missing retro_get_memory_data".into(),
        });
    }
    Ok(unsafe { std::mem::transmute(sym) })
}

fn get_memory_data_setter_fn(handle: *mut c_void) -> Result<RetroGetMemoryDataFn, CoreError> {
    // Same function works for both getting and setting (returns writable pointer)
    get_memory_data_fn(handle)
}
