use thiserror::Error;

use crate::core::asset::AssetTypeId;
use crate::core::commands::{EditorCommandDispatchError, EditorCommandRegistryError};
use crate::core::editing::engine::EditCommandError;
use crate::core::editor_extension::EditorExtensionRegistryError;
use crate::core::play::{PlaySceneSourceError, PlaySessionError};
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerError;
use crate::ui::binding_dispatch::EditorBindingDispatchError;
use crate::ui::host::play_pending_decision::PlayPendingDecisionPublishError;
use crate::ui::host::EditorError;
use crate::ui::workbench::state::{EditorStateOperationError, EditorViewportStateError};

use super::super::editor_host_event_controller::EditorTerminalPlayDetachError;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssetKindFilterError {
    #[error("unknown asset kind filter `{value}`")]
    Unknown { value: String },
}

#[derive(Debug, Error)]
pub enum AssetEventExecutionError {
    #[error(transparent)]
    Extension(#[from] EditorExtensionRegistryError),
    #[error("asset type `{asset_type}` is not registered")]
    UnregisteredAssetType { asset_type: AssetTypeId },
    #[error(transparent)]
    CommandRegistry(#[from] EditorCommandRegistryError),
    #[error(transparent)]
    Command(#[from] EditorCommandDispatchError),
    #[error(transparent)]
    KindFilter(#[from] AssetKindFilterError),
    #[error("asset toolkit route serialization failed: {source}")]
    RouteSerialization {
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid asset UUID `{asset_uuid}`: {source}")]
    InvalidAssetUuid { asset_uuid: String, source: String },
    #[error("invalid asset relocation target `{target_locator}`: {source}")]
    InvalidRelocationTarget {
        target_locator: String,
        #[source]
        source: zircon_runtime::core::resource::ResourceLocatorError,
    },
    #[error(transparent)]
    Editor(#[from] EditorError),
}

#[derive(Debug, Error)]
pub enum MenuActionExecutionError {
    #[error("project save {phase} failed: {source}")]
    Transaction {
        phase: &'static str,
        #[source]
        source: EditCommandError,
    },
    #[error("No project open")]
    NoProjectOpen,
    #[error(transparent)]
    Editor(#[from] EditorError),
    #[error(transparent)]
    PlaySceneSource(#[from] PlaySceneSourceError),
    #[error(transparent)]
    State(#[from] EditorStateOperationError),
    #[error("failed to enter play session: {source}")]
    PlayStart {
        #[source]
        source: PlaySessionError,
    },
    #[error("failed to enter play session: {source}; editor state restore also failed: {restore}")]
    PlayStartRestoreStateFailed {
        #[source]
        source: PlaySessionError,
        restore: EditorStateOperationError,
    },
    #[error("failed to bind runtime plugin event consumers: {source}")]
    RuntimeConsumerStart {
        #[source]
        source: EditorRuntimeEventConsumerError,
    },
    #[error(
        "failed to bind runtime plugin event consumers: {source}; failed to stop play session: {stop}"
    )]
    RuntimeConsumerStartStopFailed {
        #[source]
        source: EditorRuntimeEventConsumerError,
        stop: PlaySessionError,
    },
    #[error(
        "failed to bind runtime plugin event consumers: {source}; play session stopped but its runtime gateway could not be detached: {detach}"
    )]
    RuntimeConsumerStartGatewayDetachFailed {
        #[source]
        source: EditorRuntimeEventConsumerError,
        detach: EditorTerminalPlayDetachError,
    },
    #[error(
        "failed to bind runtime plugin event consumers: {source}; play session stopped but failed to restore editor state: {restore}"
    )]
    RuntimeConsumerStartRestoreStateFailed {
        #[source]
        source: EditorRuntimeEventConsumerError,
        restore: EditorStateOperationError,
    },
    #[error("failed to clean up runtime event consumers: {source}")]
    RuntimeConsumerStop {
        #[source]
        source: EditorRuntimeEventConsumerError,
    },
    #[error("failed to stop play session: {source}")]
    PlayStop {
        #[source]
        source: PlaySessionError,
    },
    #[error("play session stopped but its runtime gateway could not be detached: {source}")]
    PlayGatewayDetach {
        #[source]
        source: EditorTerminalPlayDetachError,
    },
    #[error("play session stopped but failed to restore editor state: {source}")]
    PlayStopRestoreStateFailed {
        #[source]
        source: EditorStateOperationError,
    },
    #[error(transparent)]
    PendingEditDecision(#[from] PlayPendingDecisionPublishError),
}

#[derive(Debug, Error)]
pub enum EditorEventExecutionError {
    #[error(transparent)]
    Menu(#[from] MenuActionExecutionError),
    #[error("layout event execution failed: {source}")]
    Layout {
        #[source]
        source: EditorError,
    },
    #[error("selection event execution failed: {source}")]
    Selection {
        #[source]
        source: EditorStateOperationError,
    },
    #[error("hierarchy event execution failed: {source}")]
    Hierarchy {
        #[source]
        source: EditorStateOperationError,
    },
    #[error(transparent)]
    Asset(#[from] AssetEventExecutionError),
    #[error("draft event execution failed: {source}")]
    Draft {
        #[source]
        source: EditorBindingDispatchError,
    },
    #[error("animation event execution failed: {source}")]
    Animation {
        #[source]
        source: EditorError,
    },
    #[error("inspector event execution failed: {source}")]
    Inspector {
        #[source]
        source: EditorBindingDispatchError,
    },
    #[error("viewport event execution failed: {source}")]
    Viewport {
        #[source]
        source: EditorViewportStateError,
    },
    #[error("{message}")]
    RecordedOperationControlFailure {
        operation_id: String,
        message: String,
    },
}
