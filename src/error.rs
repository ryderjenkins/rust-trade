use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradeError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("Environment error: {0}")]
    EnvError(#[from] std::env::VarError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TradeError>;