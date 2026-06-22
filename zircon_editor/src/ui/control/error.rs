use thiserror::Error;
use zircon_runtime_interface::ui::event_ui::UiInvocationError;

#[derive(Debug, Error)]
pub enum EditorUiError {
    #[error("activity view {0} already registered")]
    DuplicateActivityView(String),
    #[error("activity window {0} already registered")]
    DuplicateActivityWindow(String),
    #[error(transparent)]
    Invocation(#[from] UiInvocationError),
}
