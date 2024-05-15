use std::ffi::CStr;

use ffi_log2::LogParam;
use hamserror::HamsError;

pub mod ffi;
pub mod hams;
pub mod hamserror;
pub mod probes;

pub use hams::Hams;
pub use probes::ProbeManual;

/// Name of the Crate
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn hello_world() {
    unsafe { ffi::hello_world() }
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
