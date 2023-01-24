use ffi_log2::LogParam;
use log::info;
use std::{fmt::Display, marker::PhantomData};

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
        if hams.is_null() {
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

    pub fn stop(&self) -> Result<(), HamsError> {
        let retval = unsafe { ffi::hams_stop(self.c) };
        if retval == 0 {
            Err(HamsError::Message("Failed to start HaMS".to_string()))
        } else {
            Ok(())
        }
    }
}

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
