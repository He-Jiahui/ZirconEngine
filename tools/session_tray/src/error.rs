use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrayError {
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("JSON contract is invalid")]
    Json(#[from] serde_json::Error),
    #[error("runtime descriptor is invalid: {0}")]
    InvalidDescriptor(&'static str),
    #[error("coordinator identity mismatch: {0}")]
    IdentityMismatch(&'static str),
    #[error("automatic recovery is suppressed: {0}")]
    RecoverySuppressed(&'static str),
    #[error("coordinator request failed: {0}")]
    Http(String),
    #[error("coordinator returned an error: {code}")]
    Coordinator { code: String, message: String },
    #[error("Windows operation failed")]
    Windows(#[from] windows::core::Error),
    #[error("Tauri operation failed")]
    Tauri(#[from] tauri::Error),
}
