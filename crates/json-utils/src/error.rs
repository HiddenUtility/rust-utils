use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonError {
    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}
