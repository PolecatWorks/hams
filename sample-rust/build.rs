use std::env;
use std::path::Path;

fn main() {
    let dir = env::var("OUT_DIR").unwrap();

    let target_path = Path::new(&dir).join("../../..");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=native={}", target_path.display());

    // // From here: https://crates.io/crates/bind-builder BUT cannot get it working so using rustc-link-search instead. Followed by using install_name_tool as noted in README.md
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../lib");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
}
// rustflags = ["-C", "link-args=-Wl,-rpath,$ORIGIN/../lib/"]
