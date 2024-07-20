use std::{ffi::NulError, fmt};
use thiserror::Error;

use crate::hams::config::HamsConfigBuilderError;

// Error type for handling errors on FFI calls
#[derive(Error, Debug)]
pub enum HamsError {
    /// A standard error with configurable message
    #[error("Generic error message (use sparigly): `{0}`")]
    Message(String),
    /// A Nul was found on FFI pointer
    #[error("NulError from FFI pointer")]
    NulError(#[from] NulError),

    /// Error when building config
    #[error("Error building config")]
    ConfigError(#[from] HamsConfigBuilderError),

    /// An error with unknown source
    #[error("Unknown error")]
    Unknown,
    /// Try conversion from int
    #[error("Try conversion from int")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}

// impl fmt::Display for HamsError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             HamsError::Message(msg) => write!(f, "Custom error: {}", msg),
//             HamsError::NulError => write!(f, "Null was retuned"),
//             HamsError::TryFromIntError => write!(f, "Try conversion from int"),
//             HamsError::Unknown => todo!(),
//         }
//     }
// }

// impl Error for HamsError {}

// impl From<NulError> for HamsError {
//     fn from(_: NulError) -> HamsError {
//         HamsError::NulError
//     }
// }
