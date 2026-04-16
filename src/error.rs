use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Game error: {0}")]
    Game(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl From<crate::domain::HangarError> for AppError {
    fn from(e: crate::domain::HangarError) -> Self {
        AppError::BadRequest(e.to_string())
    }
}

impl From<crate::domain::GameInstanceError> for AppError {
    fn from(e: crate::domain::GameInstanceError) -> Self {
        AppError::Game(e.to_string())
    }
}
