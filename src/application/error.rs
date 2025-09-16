use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self { Self::Validation(msg.into()) }
    pub fn conflict(msg: impl Into<String>) -> Self { Self::Conflict(msg.into()) }
    pub fn not_found(msg: impl Into<String>) -> Self { Self::NotFound(msg.into()) }
}
