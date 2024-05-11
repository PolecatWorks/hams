use std::ffi::CStr;
use std::ffi::CString;

use ffi_log2::LogParam;
use hamserror::HamsError;

pub mod client;
pub mod config;
pub mod ffi;
pub mod hams;
pub mod hamserror;
pub mod probes;
pub mod smoke;

pub use hams::Hams;
pub use probes::ProbeManual;

/// Name of the Crate
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn hello_world() {
    unsafe { ffi::hello_world() }
}

#[no_mangle]
pub extern "C" fn prometheus_response() -> *const libc::c_char {
    println!("Callback from C2");

    let prometheus = String::from("Hello from Rust");

    let c_str_prometheus = CString::new(prometheus).unwrap();
    c_str_prometheus.into_raw()
}

#[no_mangle]
pub extern "C" fn prometheus_response_free(ptr: *mut libc::c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    };
}

pub fn hello_callback2() {
    let mut x = String::from("Hello from Rust");

    unsafe { ffi::hello_callback2(prometheus_response, prometheus_response_free) }
}

pub fn hams_version() -> String {
    let c_str = unsafe { ffi::hams_version() };
    let r_str = unsafe { CStr::from_ptr(c_str) };
    r_str.to_str().unwrap().to_string()
}

/// Initialise logging
pub fn hams_logger_init(param: LogParam) -> Result<(), HamsError> {
    if unsafe { ffi::hams_logger_init(param) } == 0 {
        return Err(HamsError::Message("Logging did not register".to_string()));
    }
    Ok(())
}
