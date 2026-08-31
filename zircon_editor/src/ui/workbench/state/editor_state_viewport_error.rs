use std::fmt;

use thiserror::Error;

use super::editor_state_keep_play_changes::KeepPlayChangesError;
use crate::core::editing::authoring_world::AuthoringWorldAccessError;
use crate::core::editing::engine::EditCommandError;
use crate::core::editing::interactive_transform::InteractiveTransformError;
use crate::core::play::WorldDomain;
use crate::scene::viewport::SceneViewportControllerError;
use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::reflect::ReflectError;
use zircon_runtime_interface::ui::tree::UiTreeError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoTransactionPhase {
    MutationPreflight,
    ContextBinding,
    CommandCapture,
    CommandExecution,
}

#[derive(Debug, Error)]
pub enum GizmoTransactionError {
    #[error("gizmo transaction requires an open project")]
    NoProjectOpen,
    #[error("gizmo transaction requires an active scene document")]
    SceneDocumentNotActive,
    #[error("gizmo transaction requires CoreEditContext")]
    TransactionContextMissing,
    #[error("interactive transform failed: {message}")]
    InteractiveTransform { message: String },
    #[error(transparent)]
    AuthoringWorld(#[from] AuthoringWorldAccessError),
    #[error("gizmo transaction {phase:?} failed")]
    EditCommand {
        phase: GizmoTransactionPhase,
        #[source]
        source: EditCommandError,
    },
    #[error("gizmo transaction failed with {cause}; rollback also failed with {rollback}")]
    RollbackFailed {
        #[source]
        cause: Box<GizmoTransactionError>,
        rollback: Box<GizmoTransactionError>,
    },
}

impl PartialEq for GizmoTransactionError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NoProjectOpen, Self::NoProjectOpen)
            | (Self::SceneDocumentNotActive, Self::SceneDocumentNotActive)
            | (Self::TransactionContextMissing, Self::TransactionContextMissing) => true,
            (
                Self::InteractiveTransform {
                    message: left_message,
                },
                Self::InteractiveTransform {
                    message: right_message,
                },
            ) => left_message == right_message,
            (Self::AuthoringWorld(left), Self::AuthoringWorld(right)) => left == right,
            (
                Self::EditCommand {
                    phase: left_phase,
                    source: left_source,
                },
                Self::EditCommand {
                    phase: right_phase,
                    source: right_source,
                },
            ) => left_phase == right_phase && left_source.to_string() == right_source.to_string(),
            (
                Self::RollbackFailed {
                    cause: left_cause,
                    rollback: left_rollback,
                },
                Self::RollbackFailed {
                    cause: right_cause,
                    rollback: right_rollback,
                },
            ) => left_cause == right_cause && left_rollback == right_rollback,
            _ => false,
        }
    }
}

impl From<InteractiveTransformError> for GizmoTransactionError {
    fn from(error: InteractiveTransformError) -> Self {
        Self::InteractiveTransform {
            message: error.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorTransformField {
    Translation,
    Scale,
}

impl fmt::Display for InspectorTransformField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Translation => formatter.write_str("translation"),
            Self::Scale => formatter.write_str("scale"),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum InspectorEditError {
    #[error("Nothing selected")]
    NoSelection,
    #[error("node name cannot be empty")]
    EmptyNodeName,
    #[error("Parent field must be a valid node id: {value}")]
    InvalidParentField { value: String },
    #[error("Transform {field} fields must be finite numbers")]
    InvalidTransformFields { field: InspectorTransformField },
    #[error("unsupported inspector field {field_id}")]
    UnsupportedFieldId { field_id: String },
    #[error("failed to read inspector field `{field_id}`: {source}")]
    ReflectionRead {
        field_id: String,
        #[source]
        source: ReflectError,
    },
    #[error("Inspector property value `{value}` must be a bool")]
    InvalidBool { value: String },
    #[error("Inspector property value `{value}` must be a signed integer")]
    InvalidSignedInteger { value: String },
    #[error("Inspector property value `{value}` must be an unsigned integer")]
    InvalidUnsignedInteger { value: String },
    #[error("Inspector property value `{value}` must be a number")]
    InvalidNumber { value: String },
    #[error(
        "Inspector property value `{value}` must be a {type_name} with {component_count} finite numbers"
    )]
    InvalidVector {
        value: String,
        type_name: &'static str,
        component_count: usize,
    },
    #[error("Inspector property value `{value}` must be an entity id or none")]
    InvalidEntity { value: String },
    #[error(
        "Inspector customization only supports scalar, bool, string, enum, resource, vector, quaternion, and entity fields"
    )]
    UnsupportedValueKind,
}

#[derive(Debug, Error)]
pub enum EditorStateOperationError {
    #[error("No project open")]
    NoProjectOpen,
    #[error("scene editing requires an active scene document")]
    SceneDocumentNotActive,
    #[error("scene editing is disabled during play mode")]
    SceneEditingDisabledDuringPlay,
    #[error("play history is unavailable until the play world becomes active")]
    PlayWorldNotActive,
    #[error("scene mutation requires the active gizmo preview to be canceled first")]
    SceneActionBlockedByActiveGizmo,
    #[error("save or discard the current scene before opening or creating another scene")]
    SceneTransitionDirty,
    #[error("editor transaction context is not CoreEditContext")]
    TransactionContextMissing,
    #[error("created scene node did not become selected")]
    CreatedNodeNotSelected,
    #[error("Cannot select missing node {node_id}")]
    SelectedNodeMissing { node_id: NodeId },
    #[error("selection event targets {requested:?} while {active:?} is active")]
    SelectionWorldMismatch {
        requested: WorldDomain,
        active: WorldDomain,
    },
    #[error("imported mesh node did not become selected")]
    ImportedMeshNodeNotSelected,
    #[error("play session disappeared during exclusive exit")]
    PlaySessionMissing,
    #[error("cannot apply inspector changes while a gizmo interaction is active")]
    InspectorBindingActiveGizmo,
    #[error("forced inspector checkpoint restore failure")]
    InspectorCheckpointRestoreFailed,
    #[error(transparent)]
    AuthoringWorld(#[from] AuthoringWorldAccessError),
    #[error(transparent)]
    GizmoTransaction(#[from] GizmoTransactionError),
    #[error(transparent)]
    EditCommand(#[from] EditCommandError),
    #[error(transparent)]
    Inspector(#[from] InspectorEditError),
    #[error(transparent)]
    KeepPlayChanges(#[from] KeepPlayChangesError),
    #[error(transparent)]
    SettingsMutation(#[from] crate::core::settings::SettingsMutationError),
}

impl PartialEq for EditorStateOperationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NoProjectOpen, Self::NoProjectOpen)
            | (Self::SceneDocumentNotActive, Self::SceneDocumentNotActive)
            | (Self::SceneEditingDisabledDuringPlay, Self::SceneEditingDisabledDuringPlay)
            | (Self::PlayWorldNotActive, Self::PlayWorldNotActive)
            | (Self::SceneActionBlockedByActiveGizmo, Self::SceneActionBlockedByActiveGizmo)
            | (Self::SceneTransitionDirty, Self::SceneTransitionDirty)
            | (Self::TransactionContextMissing, Self::TransactionContextMissing)
            | (Self::CreatedNodeNotSelected, Self::CreatedNodeNotSelected)
            | (Self::ImportedMeshNodeNotSelected, Self::ImportedMeshNodeNotSelected)
            | (Self::PlaySessionMissing, Self::PlaySessionMissing)
            | (Self::InspectorBindingActiveGizmo, Self::InspectorBindingActiveGizmo)
            | (Self::InspectorCheckpointRestoreFailed, Self::InspectorCheckpointRestoreFailed) => {
                true
            }
            (Self::AuthoringWorld(left), Self::AuthoringWorld(right)) => left == right,
            (
                Self::SelectedNodeMissing {
                    node_id: left_node_id,
                },
                Self::SelectedNodeMissing {
                    node_id: right_node_id,
                },
            ) => left_node_id == right_node_id,
            (
                Self::SelectionWorldMismatch {
                    requested: left_requested,
                    active: left_active,
                },
                Self::SelectionWorldMismatch {
                    requested: right_requested,
                    active: right_active,
                },
            ) => left_requested == right_requested && left_active == right_active,
            (Self::GizmoTransaction(left), Self::GizmoTransaction(right)) => left == right,
            (Self::EditCommand(left), Self::EditCommand(right)) => {
                left.to_string() == right.to_string()
            }
            (Self::Inspector(left), Self::Inspector(right)) => left == right,
            (Self::KeepPlayChanges(left), Self::KeepPlayChanges(right)) => left == right,
            (Self::SettingsMutation(left), Self::SettingsMutation(right)) => left == right,
            _ => false,
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum EditorViewportStateError {
    #[error(transparent)]
    AuthoringWorld(#[from] AuthoringWorldAccessError),
    #[error(transparent)]
    PointerRoute(#[from] UiTreeError),
    #[error(transparent)]
    ViewportController(#[from] SceneViewportControllerError),
    #[error(transparent)]
    StateMutation(#[from] GizmoTransactionError),
}
