use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let libretro_h = PathBuf::from(&crate_dir).parent().unwrap().join("doc").join("libretro.h");
    let src_dir = PathBuf::from(&crate_dir).join("src");

    println!("cargo:rerun-if-changed={}", libretro_h.display());
    println!("cargo:rerun-if-changed={}", src_dir.join("log_helper.c").display());

    // Compile the C log helper (shim for variadic retro_log_printf_t)
    cc::Build::new()
        .file(src_dir.join("log_helper.c"))
        .compile("rustsdlretro-log-helper");

    // Tell linker to expose rust_log_callback from lib.rs
    println!("cargo:rustc-link-arg=-Wl,--export-dynamic");

    let bindings = bindgen::Builder::default()
        .header(libretro_h.to_str().unwrap())
        .derive_default(true)
        .layout_tests(true)
        .allowlist_type("retro_.*")
        .allowlist_function("retro_.*")
        .allowlist_var("retro_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings");
}
