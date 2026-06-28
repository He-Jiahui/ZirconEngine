use thiserror::Error;
use zircon_runtime_interface::ui::{
    dispatch::UiInputMethodSurroundingTextError, event_ui::UiNodeId, tree::UiTreeError,
};

pub type UiSurfaceInputEffectResult<T> = std::result::Result<T, UiSurfaceInputEffectError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiSurfaceInputEffectError {
    #[error("invalid input owner {node_id:?}")]
    InvalidInputOwner { node_id: UiNodeId },
    #[error("missing node {node_id:?}")]
    MissingNode { node_id: UiNodeId },
    #[error("missing dirty target {node_id:?}")]
    MissingDirtyTarget { node_id: UiNodeId },
    #[error("expected {expected} effect")]
    UnexpectedEffect { expected: &'static str },
    #[error("focus rejected: {source}")]
    FocusRejected { source: UiTreeError },
    #[error("focus owner mismatch")]
    FocusOwnerMismatch,
    #[error("pointer capture belongs to a different or unknown pointer")]
    PointerCaptureOwnerMismatch,
    #[error("pointer lock owner mismatch")]
    PointerLockOwnerMismatch,
    #[error("high precision requires pointer capture")]
    HighPrecisionRequiresPointerCapture,
    #[error("high precision owner mismatch")]
    HighPrecisionOwnerMismatch,
    #[error("drag session already active")]
    DragSessionAlreadyActive,
    #[error("drag session is not active")]
    DragSessionInactive,
    #[error("drag pointer owner mismatch")]
    DragPointerOwnerMismatch,
    #[error("drag session owner mismatch")]
    DragSessionOwnerMismatch,
    #[error("component state dirty rejected: {source}")]
    ComponentStateDirtyRejected { source: UiTreeError },
    #[error("navigation route rejected: {source}")]
    NavigationRouteRejected { source: UiTreeError },
    #[error("navigation target rejected: {source}")]
    NavigationTargetRejected { source: UiTreeError },
    #[error("navigation focus rejected: {source}")]
    NavigationFocusRejected { source: UiTreeError },
    #[error("invalid input method surrounding text: {validation_error}")]
    InvalidInputMethodSurroundingText {
        #[source]
        validation_error: UiInputMethodSurroundingTextError,
    },
    #[error("input method owner mismatch")]
    InputMethodOwnerMismatch,
    #[error("clipboard read request cannot carry text")]
    ClipboardReadRequestCarriesText,
    #[error("clipboard write request missing text")]
    ClipboardWriteRequestMissingText,
}
