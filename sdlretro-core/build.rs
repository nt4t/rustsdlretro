use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let libretro_h = PathBuf::from(&crate_dir).parent().unwrap().join("libretro.h");

    println!("cargo:rerun-if-changed={}", libretro_h.display());

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
