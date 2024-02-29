use std::marker::PhantomData;

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

/// Initialise logging
pub fn hams_logger_init(param: LogParam) -> Result<(), HamsError> {
    if unsafe { ffi::hams_logger_init(param) } == 0 {
        return Err(HamsError::Message("Logging did not register".to_string()));
    }
    Ok(())
}

/// Hams is an FFI struct to opaquely handle the object that was created by the Hams API.
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
        let c = unsafe { ffi::hams_init(c_name.as_ptr()) };
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
            panic!("FAILED to freem HaMS");
        }

        info!("HaMS freed")
    }
}
