#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML decode error: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("TOML encode error: {0}")]
    TomlEncode(#[from] toml::ser::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Slint platform error: {0}")]
    Slint(#[from] slint::PlatformError),
    #[error("{0}")]
    Message(String),
}

impl HubError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
