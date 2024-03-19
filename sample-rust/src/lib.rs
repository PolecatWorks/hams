use std::{ffi::CStr, marker::PhantomData};

use ffi_log2::LogParam;
use hamserror::HamsError;
use log::info;

pub mod config;
pub mod ffi;
pub mod hamserror;
pub mod smoke;

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

/// Hams is an FFI struct to opaquely handle the object that was created by the Hams API.
///
/// This allows the developer to use the Hams C API using safe rust.
pub struct Hams<'a> {
    pub c: *mut ffi::Hams,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Hams<'a> {
    /// Construct the new Hams.
    /// The return of thi call will have created an object via FFI to handle and manage
    /// your alive and readyness checks.
    /// It also manages your monitoring via prometheus exports
    pub fn new<S: Into<String>>(name: S) -> Result<Hams<'a>, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        info!("Registering HaMS: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::hams_new(c_name.as_ptr()) };
        if c.is_null() {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to create Hams object".to_string(),
            ));
        }
        Ok(Hams {
            c,
            _marker: PhantomData,
        })
    }
}

/// This trait automatically handles the deallocation of the hams api when the Hams object
/// goes out of scope
impl<'a> Drop for Hams<'a> {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        let retval = unsafe { ffi::hams_free(self.c) };
        if retval == 0 {
            panic!("FAILED to free HaMS");
        }

        info!("HaMS freed")
    }
}

pub struct ProbeManual<'a> {
    pub c: *mut ffi::Probe,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ProbeManual<'a> {
    /// Construct a new manual probe
    pub fn manual_new<S: Into<String>>(
        name: S,
        valid: bool,
    ) -> Result<ProbeManual<'a>, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        info!("New ManualHealthProbe: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::probe_manual_new(c_name.as_ptr(), valid) };
        if c.is_null() {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to create Probe object".to_string(),
            ));
        }
        Ok(ProbeManual {
            c,
            _marker: PhantomData,
        })
    }
}

impl<'a> Drop for ProbeManual<'a> {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        let retval = unsafe { ffi::probe_manual_free(self.c) };
        if retval == 0 {
            panic!("FAILED to free Probe");
        }

        info!("Probe freed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe() {
        let probe = ProbeManual::manual_new("test_probe", true).unwrap();

        drop(probe);
    }
}
