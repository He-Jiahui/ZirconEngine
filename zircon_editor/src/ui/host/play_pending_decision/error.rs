use thiserror::Error;

use crate::core::notifications::{DecisionNotificationError, NotificationIdentityError};
use crate::core::play::PlayEditResolutionError;
use crate::ui::host::EditorOperationDispatchError;

#[derive(Debug, Error)]
pub enum PlayPendingDecisionPublishError {
    #[error("play pending-edit decision identifier space is exhausted")]
    SequenceExhausted,
    #[error(transparent)]
    NotificationIdentity(#[from] NotificationIdentityError),
    #[error(transparent)]
    Decision(#[from] DecisionNotificationError),
}

#[derive(Debug, Error)]
pub enum PlayPendingDecisionReceiptRecoveryError {
    #[error(transparent)]
    Decision(#[from] DecisionNotificationError),
    #[error(transparent)]
    Publish(#[from] PlayPendingDecisionPublishError),
    #[error(
        "an owned Apply/Discard choice was lost and no pending prompt is available for explicit replacement"
    )]
    ReplacementPromptUnavailable,
    #[error("expired pending play-edit receipt recovery did not establish a replacement Decision")]
    ReplacementDecisionNotEstablished,
}

#[derive(Debug, Error)]
pub enum PlayPendingDecisionReceiptDispatchError {
    #[error(transparent)]
    Operation(#[from] EditorOperationDispatchError),
    #[error(transparent)]
    Resolution(#[from] PlayEditResolutionError),
    #[error("unsupported pending play-edit decision option `{option}`")]
    UnsupportedOption { option: String },
}

#[derive(Debug, Error)]
pub enum PlayPendingDecisionReceiptError {
    #[error(transparent)]
    Decision(#[from] DecisionNotificationError),
    #[error(transparent)]
    Dispatch(#[from] PlayPendingDecisionReceiptDispatchError),
    #[error(
        "pending play-edit receipt effect committed but prompt reconciliation failed: {source}"
    )]
    Reconcile {
        #[source]
        source: PlayPendingDecisionPublishError,
    },
    #[error(transparent)]
    Recovery(#[from] PlayPendingDecisionReceiptRecoveryError),
    #[error(
        "pending play-edit receipt replay lost an unconsumed Apply/Discard choice; a new Decision was published and must be selected explicitly"
    )]
    ExplicitReplacementRequired,
}
