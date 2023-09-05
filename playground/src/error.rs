use std::error::Error;
use std::fmt;

use crate::health_check::Poisoned;

#[derive(Debug)]
pub enum HamsError {
    /// A standard error with configurable message
    Message(&'static str),
    InvalidData(&'static str),
    Poisoned(Poisoned),
    /// A dynamic error message from a String
    DynMessage(String),
    /// Runtime has been cancelled
    Cancelled,
}

impl fmt::Display for HamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HamsError::Message(msg) => write!(f, "custom error: {msg}",),
            HamsError::DynMessage(msg) => write!(f, "custom error: {msg}"),
            HamsError::Cancelled => write!(f, "runtime cancelled"),
            HamsError::InvalidData(msg) => write!(f, "invalid data: {msg}"),
            HamsError::Poisoned(msg) => write!(f, "poisoned: {msg}"),
        }
    }
}
