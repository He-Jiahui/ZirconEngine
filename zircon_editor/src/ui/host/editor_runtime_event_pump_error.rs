use thiserror::Error;

use crate::core::play::PlaySessionError;
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerError;
use crate::ui::host::play_pending_decision::{
    PlayPendingDecisionPublishError, PlayPendingDecisionReceiptError,
};
use crate::ui::workbench::state::EditorStateOperationError;

use super::editor_host_event_controller::EditorTerminalPlayDetachError;

#[derive(Debug, Error)]
pub enum EditorRuntimeEventPumpError {
    #[error(transparent)]
    PendingDecisionReceipt(#[from] PlayPendingDecisionReceiptError),
    #[error("failed to poll play backend: {source}")]
    BackendPoll {
        #[source]
        source: PlaySessionError,
    },
    #[error("runtime preview stopped but its play gateway could not be detached: {source}")]
    PlayGatewayDetach {
        #[source]
        source: EditorTerminalPlayDetachError,
    },
    #[error(
        "runtime preview stopped but its App-owned session could not enter retirement: {source}"
    )]
    PlayBackendRetirement {
        #[source]
        source: PlaySessionError,
    },
    #[error(transparent)]
    RuntimeConsumer(#[from] EditorRuntimeEventConsumerError),
    #[error(transparent)]
    PendingDecisionPublish(#[from] PlayPendingDecisionPublishError),
    #[error("runtime preview stopped but failed to restore editor state: {source}")]
    StateRestore {
        #[source]
        source: EditorStateOperationError,
    },
    #[error(
        "runtime preview stopped but failed to publish the pending play-edit decision: {decision}; editor-state restore also failed: {restore}"
    )]
    PendingDecisionPublishAndStateRestore {
        #[source]
        decision: PlayPendingDecisionPublishError,
        restore: EditorStateOperationError,
    },
}
