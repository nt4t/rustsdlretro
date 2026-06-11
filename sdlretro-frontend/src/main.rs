use sdlretro_core::Core;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <core.so> <game.rom>", args[0]);
        std::process::exit(1);
    }

    let core_path = &args[1];
    let rom_path = &args[2];

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

    eprintln!("Loading ROM: {}", rom_path);
    if core.load_game(Path::new(rom_path)).is_err() {
        eprintln!("Failed to load game");
        std::process::exit(1);
    }
    eprintln!("Load OK");

    eprintln!("Running 3 frames...");
    for i in 0..3 {
        if core.run().is_err() {
            eprintln!("Failed to run frame {}", i);
            std::process::exit(1);
        }
        eprintln!("  Frame {}", i + 1);
    }

    eprintln!("Unloading...");
    core.unload();
    eprintln!("Done.");
}
