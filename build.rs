extern crate cbindgen;

use cbindgen::Config;
use std::env;
use std::path::PathBuf;

// https://michael-f-bryan.github.io/rust-ffi-guide/cbindgen.html

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    cbindgen::generate(&crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);
}

fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
