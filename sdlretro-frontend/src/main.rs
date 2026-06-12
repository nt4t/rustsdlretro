use sdlretro_core::Core;
use sdlretro_core::video::FbdevVideo;
use sdlretro_core::input::InputReader;
use sdlretro_core::Throttle;
use std::path::Path;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn sigint_handler(_sig: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn setup_signal_handler() {
    unsafe {
        libc::signal(libc::SIGINT, sigint_handler as usize);
    }
}

extern "C" fn video_refresh_cb(pixels: *const c_void, _w: u32, _h: u32, pitch: usize) {
    unsafe {
        if let Some(video) = MAIN_VIDEO.as_mut() {
            video.push_frame(pixels, pitch);
        }
    }
}

extern "C" fn input_poll_cb() {
    // Input is polled by background thread, nothing to do here
}

extern "C" fn input_state_cb(port: u32, device: u32, index: u32, id: u32) -> i16 {
    unsafe {
        if let Some(ref input) = MAIN_INPUT.as_ref() {
            return input.get_state(port, device, index, id);
        }
        -1
    }
}

static mut MAIN_VIDEO: Option<FbdevVideo> = None;
static mut MAIN_INPUT: Option<InputReader> = None;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <core.so> <game.rom>", args[0]);
        std::process::exit(1);
    }

    let core_path = &args[1];
    let rom_path = &args[2];

    eprintln!("Opening framebuffer...");
    let video = match FbdevVideo::new() {
        Ok(v) => v,
        Err(e) => { eprintln!("Failed to open framebuffer: {}", e); std::process::exit(1); }
    };
    eprintln!("Framebuffer: {}x{}bpp", video.fb_width(), video.fb_bpp());

    eprintln!("Opening keyboard input...");
    let input = match InputReader::new() {
        Ok(i) => i,
        Err(e) => { eprintln!("Failed to open input: {}", e); std::process::exit(1); }
    };
    eprintln!("Keyboard input ready");

    eprintln!("Loading core: {}", core_path);
    let mut core = match Core::new(Path::new(core_path)) {
        Ok(c) => c,
        Err(e) => { eprintln!("Failed to open core: {}", e); std::process::exit(1); }
    };

    eprintln!("Initializing core...");
    if core.init().is_err() {
        eprintln!("Failed to init core");
        std::process::exit(1);
    }
    eprintln!("Init OK");

    // Store video and input in statics for callbacks
    unsafe { MAIN_VIDEO = Some(video); }
    unsafe { MAIN_INPUT = Some(input); }

    // Get AV info to set core format and throttle
    let av_info = core.get_system_av_info();
    let geometry = &av_info.geometry;
    let timing = &av_info.timing;
    let core_w = geometry.base_width;
    let core_h = geometry.max_height;
    let fps = timing.fps;

    eprintln!("AV: {}x{} @ {:.2} FPS", core_w, core_h, fps);

    unsafe {
        if let Some(ref mut v) = MAIN_VIDEO {
            v.set_core_format(core_w, core_h, 32);
        }
    }

    // Set all callbacks at once
    core.set_callbacks(
        Some(video_refresh_cb),
        Some(input_poll_cb),
        Some(input_state_cb),
    );
    eprintln!("Callbacks registered");

    eprintln!("Loading ROM: {}", rom_path);
    if core.load_game(Path::new(rom_path)).is_err() {
        eprintln!("Failed to load game");
        std::process::exit(1);
    }
    eprintln!("Load OK");

    eprintln!("Running at ~{:.1} FPS. Press Ctrl+C to exit.", fps);

    setup_signal_handler();

    let mut throttle = Throttle::new(fps);
    let mut frame_count: u64 = 0;
    let mut last_fps_time = std::time::Instant::now();
    let mut last_fps_frames: u64 = 0;

    while RUNNING.load(Ordering::SeqCst) {
        if core.run().is_err() {
            eprintln!("Failed to run frame");
            break;
        }
        frame_count += 1;

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
                if let Some(ref mut v) = MAIN_VIDEO {
                    v.set_skip_frame();
                }
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
    core.unload();
    eprintln!("Done. ({} total frames)", frame_count);
}
