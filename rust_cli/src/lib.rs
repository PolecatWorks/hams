use ffi_log2::LogParam;
use log::info;
use std::{fmt::Display, marker::PhantomData, ptr};

use crate::error::HamsError;
mod error;
mod ffi;

//  Following this pattern: https://stackoverflow.com/questions/50107792/what-is-the-better-way-to-wrap-a-ffi-struct-that-owns-or-borrows-data

pub struct Hams<'a> {
    pub c: *mut ffi::Hams,
    _marker: PhantomData<&'a ()>,
}

// Initialise logging
pub fn hams_logger_init(param: LogParam) -> Result<(), HamsError> {
    if unsafe { ffi::hams_logger_init(param) } == 0 {
        return Err(HamsError::Message("Logging did not register".to_string()));
    }
    Ok(())
}

impl<'a> Hams<'a> {
    pub fn new<S: Into<String>>(name: S) -> Result<Hams<'a>, HamsError>
    where
        S: Display,
    {
        info!("Registering HaMS: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;

        // if reply from function is null then reply with error
        let hams = unsafe { ffi::hams_init(c_name.as_ptr()) };
        if hams == ptr::null_mut() {
            return Err(HamsError::Message("Null reply from registering".to_owned()));
        }
        Ok(Hams {
            c: hams,
            _marker: PhantomData,
        })
    }

    pub fn start(&self) -> Result<(), HamsError> {
        let retval = unsafe { ffi::hams_start(self.c) };
        if retval == 0 {
            Err(HamsError::Message("Failed to start HaMS".to_string()))
        } else {
            Ok(())
        }
    }
}

// /**
//  * Create a HaMS instance
//  */
// pub fn hams_init_ffi<S: Into<String>>(name: S) -> Result<Hams, HamsError>
// where
//     S: Display,
// {
//     info!("Registering HaMS: {}", &name);
//     let c_name = std::ffi::CString::new(name.into())?;

//     // if reply from function is null then reply with error
//     let hams = unsafe { ffi::hams_init(c_name.as_ptr()) };
//     if hams == ptr::null_mut() {
//         return Err(HamsError::Message("Null reply from registering".to_owned()));
//     }
//     Ok(unsafe {*hams})
// }

/**
 * Deregister the shared library
 */
pub fn hams_free_ffi(mut library: ffi::Hams) -> Result<(), HamsError> {
    // change return type to be Result so that we can capture error
    let retval = unsafe { ffi::hams_free(&mut library) };
    if retval != 0 {
        Err(HamsError::Message("Failed to freem hams".to_string()))
    } else {
        Ok(())
    }
}

/// Start the HaMS http service
pub fn hams_start_ffi(library: &mut ffi::Hams) -> Result<(), HamsError> {
    let retval = unsafe { ffi::hams_start(library) };
    if retval != 0 {
        Err(HamsError::Message("Failed to start HaMS".to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use ffi_log2::log_param;

    use super::*;

    #[test]
    fn logger_init() {
        hams_logger_init(log_param()).unwrap();
    }

    #[test]
    fn init_free() {
        let my_hams = hams_init_ffi("name").unwrap();

        // assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        hams_free_ffi(my_hams).unwrap();
    }
}
