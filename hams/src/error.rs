//! describe errors in Hams

use std::{ffi::NulError, str::Utf8Error};

use ffi_helpers::error_handling;
use libc::{c_char, c_int};
use thiserror::Error;

/// FFI Enum for error handling mapping to C return codes
pub enum FFIEnum {
    /// No Error
    Success = 1,
    /// Null error
    NullError = 0,
    /// Unknown error
    UnknownError = -1,
    /// CString error
    CStringError = -2,
    /// AlreadyRunning error
    AlreadyRunning = -3,
    // NotRunning error
    NotRunning = -4,
}

/// Allow conversion from i32 to FFIEnum (C return codes to FFIEnum)
impl TryFrom<i32> for FFIEnum {
    type Error = HamsError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == FFIEnum::Success as i32 => Ok(FFIEnum::Success),
            x if x == FFIEnum::NullError as i32 => Ok(FFIEnum::NullError),
            x if x == FFIEnum::UnknownError as i32 => Ok(FFIEnum::UnknownError),
            x if x == FFIEnum::CStringError as i32 => Ok(FFIEnum::CStringError),
            x if x == FFIEnum::AlreadyRunning as i32 => Ok(FFIEnum::AlreadyRunning),
            x if x == FFIEnum::NotRunning as i32 => Ok(FFIEnum::NotRunning),
            x if x >= 0 => Err(HamsError::NotError(x)),
            _ => Err(HamsError::Unknown),
        }
    }
}

impl<T: AsRef<HamsError>> From<T> for FFIEnum {
    fn from(err: T) -> Self {
        match err.as_ref() {
            HamsError::CStringToString(_) => FFIEnum::CStringError,
            HamsError::NulError(_) => FFIEnum::NullError,
            _ => FFIEnum::UnknownError,
        }
    }
}

/// Error type for handling errors on FFI calls
#[derive(Error, Debug)]
pub enum HamsError {
    /// Error when CString cannot be converted to String
    #[error("CString to String conversion error")]
    CStringToString(#[from] std::ffi::IntoStringError),
    /// Error when converting to str
    #[error("CString to str conversion error")]
    Utf8Error(#[from] Utf8Error),

    /// Error when converting to an error when not an error
    #[error("Not an error as return was {0}")]
    NotError(i32),

    /// Probe is not good
    #[error("Probe is not good")]
    ProbeNotGood(String),
    /// Error when service is not running
    #[error("Service is not running")]
    NotRunning,
    /// Error when running preflight check
    #[error("Preflight check failed")]
    PreflightCheck,
    /// Error when running shutdown check
    #[error("Shutdown check failed")]
    ShutdownCheck,
    /// Error when start is called but service is already running
    #[error("Service is already running and cannot be started again")]
    AlreadyRunning,
    /// Cancelled service
    #[error("Service was cancelled")]
    Cancelled,
    /// Error when running callback
    #[error("Error calling callback")]
    CallbackError,

    /// Error when trying to read FFI error from buffer
    #[error("FFI error buffer wasn't big enough!")]
    FFIErrorBufferNotBigEnough,

    /// Error when converting number to int
    #[error("TryFromIntError converting to int")]
    TryFromIntError(#[from] std::num::TryFromIntError),

    /// Error when converting SystemTime to Duration
    #[error("SystemTimeError converting to Duration")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    /// Error when starting a thread
    #[error("io::Error eg from tokio start")]
    IoError(#[from] std::io::Error),
    /// Error when trying to join a thread
    #[error("JoinError2")]
    JoinError2,
    /// Error when trying to join thread
    #[error("JoinError")]
    JoinError(#[from] tokio::task::JoinError),
    /// Error when trying to send signal to mpsc
    #[error("Error sending mpsc signal to channel")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<()>),
    /// Error exchanging thread handle from HaMS into Option. Did not get a Thread
    #[error("NoThread to join on stop")]
    NoThread,
    /// PoisonError from accessing MutexGuard
    #[error("PoisonError from MutexGuard")]
    PoisonError,
    /// FFI Error
    #[error("FFI Error: {0}")]
    FFIError(String),
    /// A standard error with configurable message
    #[error("Generic error message (use sparigly): `{0}`")]
    Message(String),
    /// A Nul was found on FFI pointer
    #[error("NulError from FFI pointer")]
    NulError(#[from] NulError),
    /// An error with unknown source
    #[error("Unknown error")]
    Unknown,
}

/// Convert from PoisonError to HamsError
/// No obvious way to capture generic T using thiserror
impl<T> From<std::sync::PoisonError<T>> for HamsError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::PoisonError
    }
}

// Support AsRef<HamsError> to allow refs to be used alongside value for converstion to FFIEnum
impl AsRef<HamsError> for HamsError {
    fn as_ref(&self) -> &HamsError {
        self
    }
}

impl HamsError {
    /// Convert the error to a FFIEnum whilst creating the relevant error on the FFI side
    pub fn into_ffi_enum_with_update(self) -> FFIEnum {
        let ffienum = FFIEnum::from(&self);
        error_handling::update_last_error(self);
        ffienum
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
        -1 => Err(HamsError::FFIErrorBufferNotBigEnough),
        0 => Ok(()), // Not actual error found
        len if len > 0 => {
            buffer.truncate(len as usize - 1);
            let msg = String::from_utf8(buffer).unwrap();
            // println!("Error: {}", msg);
            Err(HamsError::FFIError(msg))
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
