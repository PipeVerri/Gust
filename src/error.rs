use thiserror::Error;

#[derive(Debug, Error)]
pub enum GustError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Project error: {0}")]
    Project(String)
}

pub type Result<T> = std::result::Result<T, GustError>;