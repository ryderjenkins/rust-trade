use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Client not initialized")]
    NotInitialized,

    #[error("Substrate error: {0}")]
    SubstrateError(String),

    #[error("Block not found")]
    BlockNotFound,

    #[error("Invalid block number")]
    InvalidBlockNumber,
}

pub type Result<T> = std::result::Result<T, BlockchainError>;