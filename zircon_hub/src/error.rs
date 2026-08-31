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
    #[error("background action queue is full (capacity {capacity})")]
    BackgroundActionQueueFull { capacity: usize },
    #[error("project manifest summary error: {0}")]
    ProjectManifestSummary(#[from] zircon_runtime_interface::project::ProjectManifestSummaryError),
    #[error("project template pack error: {0}")]
    ProjectTemplatePack(#[from] zircon_runtime_interface::project::ProjectTemplatePackError),
    #[error("project launch intent error: {0}")]
    ProjectLaunchIntent(#[from] zircon_runtime_interface::project::ProjectLaunchIntentError),
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("{}", detail.render(crate::settings::HubLanguage::English))]
    Status {
        detail: Box<crate::state::HubMessage>,
        recovery: Option<Box<crate::state::HubMessage>>,
    },
    #[error("{0}")]
    Message(String),
}

impl HubError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn status(
        detail: crate::state::HubMessage,
        recovery: Option<crate::state::HubMessage>,
    ) -> Self {
        Self::Status {
            detail: Box::new(detail),
            recovery: recovery.map(Box::new),
        }
    }

    pub fn into_status_messages(
        self,
    ) -> (crate::state::HubMessage, Option<crate::state::HubMessage>) {
        match self {
            Self::Status { detail, recovery } => (*detail, recovery.map(|message| *message)),
            other => (crate::state::HubMessage::raw_text(other.to_string()), None),
        }
    }
}
