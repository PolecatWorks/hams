use std::{error::Error, fmt};

/// Error type for handling errors on Sample
#[derive(Debug)]
pub enum SampleError {
    Message(String),
    Unknown,
}

impl fmt::Display for SampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SampleError::Message(msg) => write!(f, "Custom error: {}", msg),
            SampleError::Unknown => todo!(),
        }
    }
}

impl Error for SampleError {}
