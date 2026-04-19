use thiserror::Error;
use zircon_ui::binding::UiBindingParseError;

#[derive(Debug, Error)]
pub enum EditorUiBindingError {
    #[error(transparent)]
    Parse(#[from] UiBindingParseError),
    #[error("invalid editor ui payload: {0}")]
    InvalidPayload(String),
}
