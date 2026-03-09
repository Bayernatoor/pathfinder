use crate::blockchain::BlockchainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TracerError {
    #[error("Invalid input provided")]
    InvalidInput(String),
    #[error("Invalid trace configuration")]
    InvalidConfig(String),
    #[error("Trace exceeds size limits")]
    TracingTooLarge(String),
    #[error("Internal tracing error, try again")]
    TraceLogic(String),
    #[error("Blockchain source failed")]
    Source(#[from] BlockchainError),
}

pub type Result<T> = std::result::Result<T, TracerError>;
