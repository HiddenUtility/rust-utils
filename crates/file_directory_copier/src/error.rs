use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum CopierError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Failed to create temporary path: {0}")]
    TempPathCreationFailed(PathBuf),

    #[error("Failed to rename existing output to backup: {0}. Original error: {1}")]
    BackupRenameFailed(PathBuf, std::io::Error),

    #[error("Failed to rename temporary output to final destination: {0}. Original error: {1}")]
    FinalRenameFailed(PathBuf, std::io::Error),

    #[error("Failed to restore backup from {0}: {1}")]
    BackupRestoreFailed(PathBuf, std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
}

pub type Result<T> = std::result::Result<T, CopierError>;
