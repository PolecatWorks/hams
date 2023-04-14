//! # Sample Rust
//!
//! `sample-rust` is an example crate to create a web service and demonstrate using hams
//! providing health and readyness checks.
//!
//! Following this pattern: <https://stackoverflow.com/questions/50107792/what-is-the-better-way-to-wrap-a-ffi-struct-that-owns-or-borrows-data>

use ffi_log2::LogParam;
use log::info;
use std::{fmt::Display, marker::PhantomData};

use crate::hamserror::HamsError;
mod ffi;
mod hamserror;

/// Hams is an FFI struct to opaquely handle the object that was created by the Hams API.
pub struct Hams<'a> {
    pub c: *mut ffi::Hams,
    _marker: PhantomData<&'a ()>,
}

/// Initialise logging
pub fn hams_logger_init(param: LogParam) -> Result<(), HamsError> {
    if unsafe { ffi::hams_logger_init(param) } == 0 {
        return Err(HamsError::Message("Logging did not register".to_string()));
    }
    Ok(())
}

#[cfg_attr(doc, aquamarine::aquamarine)]
///
/// Register logging for uservice
/// ```mermaid
/// sequenceDiagram
///     participant Main
///     participant sample/Hams
///     participant sample/HamsFfi
///     participant hams/HamsFfi
///     participant hams/Ham
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Main register library and SoService
///
///     Main->>+sample/Hams: new()
///     sample/Hams->>+sample/HamsFfi: hams_init
///     sample/HamsFfi->>+hams/HamsFfi: FFI
///     hams/HamsFfi-->>hams/Ham: new()
///     hams/Ham-->>hams/HamsFfi: (hams/Hams)
///     hams/HamsFfi-->>sample/HamsFfi: FFI
///     sample/HamsFfi-->>sample/Hams: (opaque)
///     sample/Hams-->>Main: (sample/Hams)
///
///     end
/// ```
///
/// Create an ergonomic API to allow the developer to use the Hams C API.
impl<'a> Hams<'a> {
    /// Construct the new Hams.
    /// The return of thi call will have created an object via FFI to handle and manage
    /// your alive and readyness checks.
    /// It also manages your monitoring via prometheus exports
    pub fn new<S: Into<String>>(name: S) -> Result<Hams<'a>, HamsError>
    where
        S: Display,
    {
        info!("Registering HaMS: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;

        // if reply from function is null then reply with error
        let hams = unsafe { ffi::hams_init(c_name.as_ptr()) };
        if hams.is_null() {
            return Err(HamsError::Message("Null reply from registering".to_owned()));
        }
        Ok(Hams {
            c: hams,
            _marker: PhantomData,
        })
    }

    /// Start the web service to expose the health endpoints of hams
    pub fn start(&self) -> Result<(), HamsError> {
        let retval = unsafe { ffi::hams_start(self.c) };
        if retval == 0 {
            Err(HamsError::Message("Failed to start HaMS".to_string()))
        } else {
            Ok(())
        }
    }

    /// Stop the web server for hams
    pub fn stop(&self) -> Result<(), HamsError> {
        let retval = unsafe { ffi::hams_stop(self.c) };
        if retval == 0 {
            Err(HamsError::Message("Failed to start HaMS".to_string()))
        } else {
            Ok(())
        }
    }
}

/// This trait automatically handles the deallocation of the hams api when the Hams object
/// goes out of scope
impl<'a> Drop for Hams<'a> {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        let retval = unsafe { ffi::hams_free(self.c) };
        if retval == 0 {
            panic!("FAILED to freem HaMS");
        }

        info!("HaMS freed")
    }
}

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};

    use ffi_log2::log_param;

    use super::*;

    #[test]
    fn logger_init() {
        hams_logger_init(log_param()).unwrap();
    }

    #[test]
    fn init_free() {
        let my_hams = Hams::new("name").unwrap();

        println!("initialised HaMS");

        drop(my_hams);

        println!("shoud have dropped by here");
    }

    #[test]
    fn start_stop() {
        let my_hams = Hams::new("name").unwrap();
        println!("initialised HaMS");

        my_hams.start().expect("HaMS started successfully");

        sleep(Duration::from_millis(200));

        my_hams.stop().expect("HaMS stopped successfully");
    }
}
