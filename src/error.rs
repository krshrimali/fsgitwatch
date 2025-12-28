use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsgitError {
    #[error("Invalid search pattern: {0}. Expected format: owner/repo")]
    InvalidPattern(String),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, FsgitError>;
