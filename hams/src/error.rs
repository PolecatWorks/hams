//! describe errors in Hams

use std::{ffi::NulError, fmt};

use ffi_helpers::error_handling;
use libc::{c_char, c_int};
use thiserror::Error;

/// Error type for handling errors on FFI calls
#[derive(Error, Debug)]
pub enum HamsError {
    /// A standard error with configurable message
    #[error("Generic error: `{0}`")]
    Message(String),
    /// A Nul was found
    #[error("NulError response")]
    NulError,
    /// An error with unknown source
    #[error("Unknown error")]
    Unknown,
}

impl From<NulError> for HamsError {
    fn from(_: NulError) -> HamsError {
        HamsError::NulError
    }
}

/// Convert FFI error messages to Result
///
/// When functions set error_msg during FFI calls the calling function can then use this
/// function to pickup that error and convert it into a Result with appropriate HamsError
/// returning from it.
///
/// If no error is found then Ok(()) is replied.
///
/// Errors can also be retunred for failure to allocate buffer
pub fn ffi_error_to_result() -> Result<(), HamsError> {
    let err_msg_length = error_handling::last_error_length();

    // then allocate a big enough buffer
    let mut buffer = vec![0; err_msg_length as usize];
    let bytes_written = unsafe {
        let buf = buffer.as_mut_ptr() as *mut c_char;
        let len = buffer.len() as c_int;
        error_handling::error_message_utf8(buf, len)
    };

    // then interpret the message
    match bytes_written {
        -1 => Err(HamsError::Message(
            "FFI error buffer wasn't big enough!".to_string(),
        )),
        0 => Ok(()), // Not actual error found
        len if len > 0 => {
            buffer.truncate(len as usize - 1);
            let msg = String::from_utf8(buffer).unwrap();
            // println!("Error: {}", msg);
            Err(HamsError::Message(format!("Error: {}", msg)))
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use ffi_helpers::{catch_panic, error_handling::clear_last_error};
    use libc::c_int;

    use super::{ffi_error_to_result, HamsError};

    #[no_mangle]
    unsafe extern "C" fn set_last_error() -> c_int {
        ffi_helpers::update_last_error(HamsError::Message("JUST ME".to_string()));
        0
    }

    #[no_mangle]
    unsafe extern "C" fn some_infallible_operation() -> c_int {
        catch_panic!(Ok(1))
    }

    #[no_mangle]
    unsafe extern "C" fn some_fallible_operation() -> c_int {
        catch_panic!(
            panic!("Shucks that was bad");
        )
    }

    #[test]
    fn test_read_last_error_with_handler() {
        clear_last_error();
        println!("Setting error content");
        unsafe { set_last_error() };
        assert!(ffi_error_to_result().is_err(), "Error should be returned");

        clear_last_error();
        println!("No error content");
        unsafe { some_infallible_operation() }; // No actual error is set
        assert!(
            ffi_error_to_result().is_ok(),
            "Error should NOT be returned"
        );

        clear_last_error();
        println!("Actual Panic");
        unsafe { some_fallible_operation() };
        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }
}

// impl From<Box<dyn Any + Send + 'static>> for Error {
//    fn from(other: Box<dyn Any + Send + 'static>) -> Error {
//      if let Some(owned) = other.downcast_ref::<String>() {
//        Error::Message(owned.clone())
//      } else if let Some(owned) = other.downcast_ref::<String>() {
//        Error::Message(owned.to_string())
//      } else {
//        Error::Unknown
//      }
//    }
// }
