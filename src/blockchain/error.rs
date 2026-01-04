use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("NetworkFailure, Check internet connection")]
    NetworkFailure(String),
    #[error("No such UTXO, please check your entry")]
    NotFound(String),
    #[error("Invalid Input please check your entry")]
    InvalidInput(String),
    #[error("Rate Limited")]
    RateLimited,
    #[error("Data is inconsistent")]
    DataInconsistency(String),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;
