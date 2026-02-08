use thiserror::Error;

#[derive(Debug, Error)]
pub enum GustError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Project parsing error: {0}")]
    ProjectParsing(String),
    #[error("Error: {0}")]
    User(String),
}

pub type Result<T> = std::result::Result<T, GustError>;