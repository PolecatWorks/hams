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

pub fn hams_version() -> String {
    let c_str = unsafe { ffi::hams_version() };
    if c_str.is_null() {
        return String::new();
    }
    let r_str = unsafe { CStr::from_ptr(c_str) };
    let result = r_str.to_str().unwrap().to_string();
    unsafe { ffi::hams_version_free(c_str) };
    result
}

/// Initialise logging
pub fn hams_logger_init(param: LogParam) -> Result<(), HamsError> {
    if unsafe { ffi::hams_logger_init(param) } == 0 {
        return Err(HamsError::Message("Logging did not register".to_string()));
    }
    Ok(())
}
