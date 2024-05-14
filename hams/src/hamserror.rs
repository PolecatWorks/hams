use std::{error::Error, ffi::NulError, fmt};

// Error type for handling errors on FFI calls
#[derive(Debug)]
pub enum HamsError {
    Message(String),
    NulError,
    Unknown,
    TryFromIntError,
}

impl fmt::Display for HamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HamsError::Message(msg) => write!(f, "Custom error: {}", msg),
            HamsError::NulError => write!(f, "Null was retuned"),
            HamsError::TryFromIntError => write!(f, "Try conversion from int"),
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
