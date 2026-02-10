use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum GustError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Project parsing error: {0}")]
    ProjectParsing(String),
    #[error("Error: {0}")]
    User(String),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error)
}

pub type Result<T> = std::result::Result<T, GustError>;