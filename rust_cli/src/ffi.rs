use std::{fmt::Display, ptr};

use ffi_log2::LogParam;
use log::info;

use hams::error::HamsError;

/// Opaque object representing HaMS objects
#[repr(C)]
pub struct Hams {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "hams", kind = "dylib")]
extern "C" {
    /// Configure logging for UService
    fn hams_logger_init(param: LogParam) -> i32;
    /// Init a HaMS and return the reference to the UService object
    fn hams_init(name: *const libc::c_char) -> *mut Hams;
    /// Free an UService
    fn hams_free(hams: *mut Hams) -> u32;
}

// Initialise logging
pub fn hams_logger_init_ffi(param: LogParam) -> Result<(), HamsError> {
    if unsafe { hams_logger_init(param) } != 0 {
        return Err(HamsError::Message("Logging did not register".to_string()));
    }
    Ok(())
}

/**
 * Create a HaMS instance
 */
pub fn hams_init_ffi<S: Into<String>>(name: S) -> Result<*mut Hams, HamsError>
where
    S: Display,
{
    info!("Registering HaMS: {}", &name);
    let c_name = std::ffi::CString::new(name.into())?;

    // if reply from function is null then reply with error
    let hams = unsafe { hams_init(c_name.as_ptr()) };
    if hams == ptr::null_mut() {
        return Err(HamsError::Message("Null reply from registering".to_owned()));
    }
    Ok(hams)
}

/**
 * Deregister the shared library
 */
pub fn hams_free_ffi(library: *mut Hams) -> Result<(), HamsError> {
    // change return type to be Result so taht we can capture error
    unsafe {
        hams_free(library);
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use ffi_log2::log_param;

    use super::*;

    #[test]
    fn logger_init() {
        hams_logger_init_ffi(log_param());
    }

    #[test]
    fn init_free() {
        let my_hams = hams_init_ffi("name").unwrap();

        // assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        hams_free_ffi(my_hams).unwrap();
    }
}
