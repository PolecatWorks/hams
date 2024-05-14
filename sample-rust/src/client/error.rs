use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    /// Error when starting a thread
    #[error("io::Error eg from tokio start")]
    IoError(#[from] std::io::Error),
    /// A standard error with configurable message
    #[error("Generic error message (use sparigly): `{0}`")]
    Message(String),
    /// Reqwest error
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),
}
