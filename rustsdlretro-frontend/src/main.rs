use rustsdlretro_core::Core;
use rustsdlretro_core::video::VideoBackend;
use rustsdlretro_core::zip_rom;
#[cfg(feature = "config")]
use rustsdlretro_core::config::{Config, Renderer};
use rustsdlretro_core::input::InputReader;
use rustsdlretro_core::gui::Gui;
use rustsdlretro_core::Throttle;
use rustsdlretro_core::ResolutionState;
use std::path::Path;
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::mem::ManuallyDrop;

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn sigint_handler(_sig: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn setup_signal_handler() {
    unsafe {
        libc::signal(libc::SIGINT, sigint_handler as usize);
    }
}

extern "C" fn video_refresh_cb(pixels: *const c_void, w: u32, h: u32, pitch: usize) {
    unsafe {
        if let Some(ref mut v) = rustsdlretro_core::MAIN_VIDEO {
            (*v).push_frame(pixels, w, h, pitch);
        }
    }
}

extern "C" fn input_poll_cb() {
    // Input is polled by background thread, nothing to do here
}

extern "C" fn input_state_cb(port: u32, device: u32, index: u32, id: u32) -> i16 {
    unsafe {
        if !MAIN_INPUT.is_null() {
            return (*MAIN_INPUT).get_state(port, device, index, id);
        }
        -1
    }
}

static mut MAIN_INPUT: *mut InputReader = std::ptr::null_mut();

#[cfg(feature = "minifb")]
static mut MINIFB_VIDEO: *mut rustsdlretro_core::video_minifb::MinifbVideo = std::ptr::null_mut();

fn create_video_backend() -> Box<dyn VideoBackend> {
    eprintln!("Opening framebuffer...");
    let video = rustsdlretro_core::video::FbdevVideo::new()
        .expect("Failed to open framebuffer");
    eprintln!("Framebuffer: {}x{}bpp", video.fb_width(), video.fb_bpp());
    Box::new(video)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <core.so> <game.rom>", args[0]);
        std::process::exit(1);
    }

    let core_path = &args[1];
    let rom_path = &args[2];

    // Create video backend
    let mut video: Box<dyn VideoBackend> = {
        #[cfg(feature = "config")]
        {
            let config = Config::load_default();
            eprintln!("Renderer: {:?}", config.renderer);
            match config.renderer {
                Renderer::Fbdev => create_video_backend(),
                #[cfg(feature = "minifb")]
                Renderer::Minifb => {
                    eprintln!("Opening minifb window ({}x{})...", config.window.width, config.window.height);
                    let v = rustsdlretro_core::video_minifb::MinifbVideo::new(
                        config.window.width,
                        config.window.height,
                        config.window.scale,
                        config.window.borderless,
                        &config.window.title,
                    ).expect("Failed to create minifb window");
                    eprintln!("Minifb window ready: {}x{}", v.fb_width(), v.fb_height());
                    Box::new(v)
                }
                #[cfg(not(feature = "minifb"))]
                Renderer::Minifb => {
                    eprintln!("Error: minifb feature not enabled");
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "config"))]
        create_video_backend()
    };

    eprintln!("Creating input reader...");
    #[cfg(feature = "minifb")]
    let mut input = InputReader::new_keyboard();
    #[cfg(not(feature = "minifb"))]
    let mut input = match InputReader::new() {
        Ok(i) => i,
        Err(e) => { eprintln!("Failed to open input: {}", e); std::process::exit(1); }
    };
    eprintln!("Input ready");

    eprintln!("Initializing GUI...");
    let mut gui = Gui::new();

    eprintln!("Loading core: {}", core_path);
    let mut core = match Core::new(Path::new(core_path)) {
        Ok(c) => c,
        Err(e) => { eprintln!("Failed to open core: {}", e); std::process::exit(1); }
    };

    let sys_dir = std::env::var("HOME").unwrap_or_default() + "/.config/rustsdlretro";
    std::fs::create_dir_all(&sys_dir).ok();
    eprintln!("System directory: {}", sys_dir);
    rustsdlretro_core::set_system_directory(core.handle(), &sys_dir);

    eprintln!("Initializing core...");
    if core.init().is_err() {
        eprintln!("Failed to init core");
        std::process::exit(1);
    }
    eprintln!("Init OK");

    // Store minifb video pointer for keyboard polling
    #[cfg(feature = "minifb")]
    unsafe {
        if let Some(minifb_video) = video.as_any_mut().and_then(|x| x.downcast_mut::<rustsdlretro_core::video_minifb::MinifbVideo>()) {
            MINIFB_VIDEO = minifb_video;
        }
    }

    unsafe {
        rustsdlretro_core::MAIN_VIDEO = Some(ManuallyDrop::new(video));
        MAIN_INPUT = Box::into_raw(Box::new(input));
    }

    // Set input callbacks before loading ROM
    core.set_callbacks(
        None,
        Some(input_poll_cb),
        Some(input_state_cb),
    );
    eprintln!("Input callbacks registered");

    eprintln!("Loading ROM: {}", rom_path);
    if let Err(e) = core.load_game(Path::new(rom_path)) {
        eprintln!("Failed to load game: {}", e);
        std::process::exit(1);
    }
    eprintln!("Load OK");

    let rom_name = if zip_rom::is_zip(Path::new(rom_path)) {
        zip_rom::get_zip_rom_name(Path::new(rom_path))
    } else {
        Path::new(rom_path).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Game")
            .to_string()
    };
    gui.set_rom_name(&rom_name);

    // Set up save directory using actual libretro core name (e.g., "Beetle PSX")
    let core_name = core.get_core_name().to_string();
    gui.set_core_name(&core_name);
    eprintln!("Using core name: {}", core_name);
    let save_dir = rustsdlretro_core::sram::ensure_save_dir(Path::new(&sys_dir), &core_name).ok();
    if let Some(ref dir) = save_dir {
        eprintln!("Save directory: {}", dir.display());
        // Auto-load SRAM after ROM load (silent on failure)
        if rustsdlretro_core::sram::load_sram(core.handle(), &rom_name, dir).is_ok() {
            eprintln!("SRAM loaded for '{}'", rom_name);
        }
    }

    let res_state = core.get_resolution_state();

    // Get AV info after ROM loaded (core may not have valid AV info before load)
    let av_info = core.get_system_av_info();
    let geometry = &av_info.geometry;
    let timing = &av_info.timing;
    let core_w = geometry.base_width;
    let core_h = geometry.base_height;
    let fps = timing.fps;

    {
        let mut s = res_state.lock().unwrap();
        s.width = core_w;
        s.height = core_h;
        s.fps = fps;
    }

    eprintln!("AV: {}x{} @ {:.2} FPS", core_w, core_h, fps);

    unsafe {
        if let Some(ref mut v) = rustsdlretro_core::MAIN_VIDEO {
            (*v).set_core_format(core_w, core_h, 32);
        }
    }

    rustsdlretro_core::set_resolution_state(res_state.clone());

    // Set video refresh callback after AV info is available
    core.set_video_refresh(Some(video_refresh_cb));
    eprintln!("Video callback registered");

    let sample_rate = timing.sample_rate;
    eprintln!("Audio sample rate (raw f64): {}", sample_rate);
    let sample_rate_u32 = sample_rate as u32;
    eprintln!("Audio sample rate (u32): {} Hz", sample_rate_u32);

    eprintln!("Initializing audio...");
    let audio_driver = rustsdlretro_core::audio::AudioDriver::new(sample_rate_u32);
    match audio_driver {
        Ok(driver) => {
            eprintln!("Audio driver initialized at {} Hz", sample_rate_u32);
            unsafe { rustsdlretro_core::MAIN_AUDIO = Box::into_raw(Box::new(driver)) as *mut c_void; }
        }
        Err(e) => {
            eprintln!("ALSA audio unavailable: {}", e);
            #[cfg(feature = "null-audio")]
            {
                eprintln!("Falling back to null audio driver (silent mode)");
                let null_driver = rustsdlretro_core::audio_null::NullAudioDriver::new(sample_rate_u32);
                unsafe { rustsdlretro_core::MAIN_AUDIO = Box::into_raw(Box::new(null_driver)) as *mut c_void; }
            }
            #[cfg(not(feature = "null-audio"))]
            {
                eprintln!("No null-audio feature compiled. Audio will be silent.");
                unsafe { rustsdlretro_core::MAIN_AUDIO = ptr::null_mut(); }
            }
        }
    }

    eprintln!("Running at ~{:.1} FPS. Press Ctrl+C to exit.", fps);

    setup_signal_handler();

    let mut throttle = Throttle::new(fps);
    let mut frame_count: u64 = 0;
    let mut last_fps_time = std::time::Instant::now();
    let mut last_fps_frames: u64 = 0;

    while RUNNING.load(Ordering::SeqCst) {

        // Poll keyboard input for minifb mode
        #[cfg(feature = "minifb")]
        unsafe {
            if !MINIFB_VIDEO.is_null() && !MAIN_INPUT.is_null() {
                (*MAIN_INPUT).poll_with_video(&*MINIFB_VIDEO);
            }
        }

        // Check if minifb window requests close (ESC)
        unsafe {
            if let Some(ref video) = rustsdlretro_core::MAIN_VIDEO {
                if (*video).should_close() {
                    eprintln!("Window close requested");
                    break;
                }
            }
        }

        // Handle GUI input
        let menu_open = unsafe {
            if let (Some(video), Some(input)) = (rustsdlretro_core::MAIN_VIDEO.as_ref(), MAIN_INPUT.as_ref()) {
                gui.handle_input(&*input, (*video).fb_height()) == rustsdlretro_core::gui::GuiState::MenuOpen
            } else {
                false
            }
        };

        // Check for save/load state keys (F2/F4) — works regardless of menu open
        let handle_save_load = unsafe {
            if let Some(input) = MAIN_INPUT.as_ref() {
                gui.check_save_load_keys(input)
            } else {
                None
            }
        };
        if let Some(action) = handle_save_load {
            // Get save dir for state file path
            let state_path = save_dir.as_ref().map(|dir| rustsdlretro_core::sram::state_path(dir, &rom_name));
            match action {
                rustsdlretro_core::gui::SaveLoadAction::Save => {
                    if let (Some(ref dir), Some(path)) = (save_dir.as_ref(), state_path) {
                        match core.save_state() {
                            Ok(state_data) => {
                                if std::fs::write(&path, &state_data).is_ok() {
                                    gui.show_flash_message("State Saved");
                                    eprintln!("[SAVELOAD] Saved {} bytes to {}", state_data.len(), path.file_name().unwrap_or_default().to_str().unwrap_or("?"));
                                } else {
                                    gui.show_flash_message("Save Failed");
                                    eprintln!("[SAVELOAD] Failed to write: {}", path.display());
                                }
                            },
                            Err(e) => {
                                gui.show_flash_message("Save Failed");
                                eprintln!("[SAVELOAD] Save failed: {}", e);
                            }
                        }
                    } else {
                        gui.show_flash_message("No Save Dir");
                        eprintln!("[SAVELOAD] No save dir or path available");
                    }
                },
                rustsdlretro_core::gui::SaveLoadAction::Load => {
                    if let Some(ref path) = state_path {
                        match std::fs::read(path) {
                            Ok(state_data) => {
                                match core.load_state(&state_data) {
                                    Ok(()) => {
                                        gui.show_flash_message("State Loaded");
                                        eprintln!("[SAVELOAD] Loaded {} bytes from {}", state_data.len(), path.file_name().unwrap_or_default().to_str().unwrap_or("?"));
                                    },
                                    Err(e) => {
                                        gui.show_flash_message("Load Failed");
                                        eprintln!("[SAVELOAD] Load failed: {}", e);
                                    }
                                }
                            },
                            Err(_) => {
                                // No state file exists — silently ignore, no flash message
                                eprintln!("No save state found at: {}", path.display());
                            }
                        }
                    } else {
                        gui.show_flash_message("No Save Dir");
                    }
                },
            }
        }

        if !menu_open && core.run().is_err() {
            eprintln!("Failed to run frame");
            break;
        }
        
        // Check and apply queued audio rate changes (deferred from environment callback)
        unsafe {
            if let Ok(queue) = rustsdlretro_core::AUDIO_RATE_CHANGE_QUEUE.lock() {
                if let Some(new_rate) = queue.take() {
                    if !rustsdlretro_core::MAIN_AUDIO.is_null() {
                        #[cfg(not(feature = "null-audio"))]
                        {
                            eprintln!("Applying deferred audio rate change: {} Hz", new_rate);
                            let audio_ptr = rustsdlretro_core::MAIN_AUDIO as *mut rustsdlretro_core::audio::AudioDriver;
                            (*audio_ptr).restart_with_rate(new_rate);
                        }
                    }
                }
            }
        }
        
        unsafe { rustsdlretro_core::AUDIO_SAMPLES_PER_FRAME = 0; }
        frame_count += 1;

        let current_fps = {
            let s = res_state.lock().unwrap();
            s.fps
        };
        if current_fps > 0.0 {
            let new_frame_time = (1_000_000.0 / current_fps) as u64;
            if new_frame_time != throttle.frame_time() {
                eprintln!("FPS changed to {:.2}, updating throttle", current_fps);
                throttle = Throttle::new(current_fps);
            }
        }

        let usecs = throttle.check_wait();
        if usecs > 0 {
            let mut remaining = usecs;
            while remaining > 0 {
                let sleep_us = (remaining.min(5000)) as u64;
                std::thread::sleep(Duration::from_micros(sleep_us));
                remaining = throttle.check_wait();
            }
        } else {
            unsafe {
                if let Some(ref mut v) = rustsdlretro_core::MAIN_VIDEO {
                    (*v).set_skip_frame();
                }
            }
        }
        // Render GUI overlay
        unsafe {
            if let Some(ref mut v) = rustsdlretro_core::MAIN_VIDEO {
                let w = (**v).fb_width();
                let h = (**v).fb_height();
                gui.render(&mut ***v, w, h);
            }
        }
        // For minifb, we need to update the window each frame
        unsafe {
            if let Some(ref mut v) = rustsdlretro_core::MAIN_VIDEO {
                (*v).update_window();
            }
        }
        let now = std::time::Instant::now();
        if now.duration_since(last_fps_time).as_secs() >= 5 {
            let elapsed_secs = now.duration_since(last_fps_time).as_secs_f64();
            let frames_this_interval = frame_count - last_fps_frames;
            let actual_fps = frames_this_interval as f64 / elapsed_secs;
            eprintln!("FPS: {:.1}", actual_fps);
            last_fps_time = now;
            last_fps_frames = frame_count;
        }
    }

    eprintln!("\nUnloading...");
    unsafe {
        if !rustsdlretro_core::MAIN_AUDIO.is_null() {
            eprintln!("Stopping audio...");
            #[cfg(feature = "null-audio")]
            {
                // Try to stop as null audio driver first
                let null_audio = &mut *(rustsdlretro_core::MAIN_AUDIO as *mut rustsdlretro_core::audio_null::NullAudioDriver);
                null_audio.stop();
                let _ = Box::from_raw(rustsdlretro_core::MAIN_AUDIO as *mut rustsdlretro_core::audio_null::NullAudioDriver);
            }
            #[cfg(not(feature = "null-audio"))]
            {
                let audio = &mut *(rustsdlretro_core::MAIN_AUDIO as *mut rustsdlretro_core::audio::AudioDriver);
                audio.stop();
                let _ = Box::from_raw(rustsdlretro_core::MAIN_AUDIO as *mut rustsdlretro_core::audio::AudioDriver);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        // Drop the video backend
        if let Some(manually_drop) = rustsdlretro_core::MAIN_VIDEO.take() {
            let _ = ManuallyDrop::into_inner(manually_drop);
        }
        if !MAIN_INPUT.is_null() {
            let _ = Box::from_raw(MAIN_INPUT);
        }
    }
    // Auto-save SRAM before unloading (silent on failure)
    if let Some(ref dir) = save_dir {
        if rustsdlretro_core::sram::save_sram(core.handle(), &rom_name, dir).is_ok() {
            eprintln!("SRAM saved for '{}'", rom_name);
        }
    }

    core.unload();
    eprintln!("Done. ({} total frames)", frame_count);
}
