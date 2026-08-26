use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let libretro_h = PathBuf::from(&crate_dir).parent().unwrap().join("doc").join("libretro.h");
    let src_dir = PathBuf::from(&crate_dir).join("src");

    println!("cargo:rerun-if-changed={}", libretro_h.display());
    println!("cargo:rerun-if-changed={}", src_dir.join("log_helper.c").display());

    // Compile log_helper.c into a static archive using cc (properly handles linking)
    let _ = cc::Build::new()
        .file(src_dir.join("log_helper.c"))
        .flag("-fvisibility=default")  // Export symbols for dlsym
        .compile("rustsdlretro-log-helper");

    // Link the C shim into the Rust crate
    println!("cargo:rustc-link-lib=static=rustsdlretro-log-helper");

    // Generate bindings.rs using bindgen
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
