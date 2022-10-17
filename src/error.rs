/// Error type for handling errors on FFI calls
#[derive(Debug)]
pub enum HamsError {
    Message(String),
    NulError,
    Unknown,
}

impl fmt::Display for HamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HamsError::Message(msg) => write!(f, "Custom error: {}", msg),
            HamsError::NulError => write!(f, "Null was retuned"),
            HamsError::Unknown => todo!(),
        }
    }
}

impl Error for HamsError {}

impl From<NulError> for HamsError {
    fn from(_: NulError) -> HamsError {
        HamsError::NulError
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

use std::{error::Error, ffi::NulError, fmt};
