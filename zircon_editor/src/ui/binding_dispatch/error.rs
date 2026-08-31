use crate::core::editing::authoring_world::AuthoringWorldAccessError;
use crate::ui::workbench::state::{EditorStateOperationError, EditorViewportStateError};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EditorBindingDispatchError {
    #[error("unsupported editor binding payload")]
    UnsupportedPayload,
    #[error("invalid animation track path {0}")]
    InvalidAnimationTrackPath(String),
    #[error("invalid subject path {0}")]
    InvalidSubjectPath(String),
    #[error("unsupported inspector field {0}")]
    UnsupportedInspectorField(String),
    #[error("invalid inspector field value for {field_id}")]
    InvalidInspectorFieldValue { field_id: String },
    #[error("unknown drawer slot {0}")]
    UnknownDrawerSlot(String),
    #[error("unknown drawer mode {0}")]
    UnknownDrawerMode(String),
    #[error("unknown asset surface {0}")]
    UnknownAssetSurface(String),
    #[error("unknown asset utility tab {0}")]
    UnknownAssetUtilityTab(String),
    #[error("unknown asset view mode {0}")]
    UnknownAssetViewMode(String),
    #[error(transparent)]
    State(#[from] EditorStateOperationError),
    #[error(transparent)]
    AuthoringWorld(#[from] AuthoringWorldAccessError),
    #[error(
        "inspector binding failed with {cause}; checkpoint restore also failed with {rollback}"
    )]
    InspectorBindingRollback {
        #[source]
        cause: Box<EditorBindingDispatchError>,
        rollback: EditorStateOperationError,
    },
    #[error(transparent)]
    ViewportState(#[from] EditorViewportStateError),
}
