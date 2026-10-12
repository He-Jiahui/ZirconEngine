#[path = "adapter.rs"]
mod adapter;
#[path = "error.rs"]
mod error;
#[path = "model.rs"]
mod model;
#[path = "publish.rs"]
mod publish;
#[path = "resolve.rs"]
mod resolve;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

pub(super) use adapter::PlayPendingEditDecisionAdapter;
pub use error::{
    PlayPendingDecisionPublishError, PlayPendingDecisionReceiptDispatchError,
    PlayPendingDecisionReceiptError, PlayPendingDecisionReceiptRecoveryError,
};
pub(crate) use model::{
    PlayPendingDecisionSelection, PlayPendingEditApplyFailure, PlayPendingEditDecisionOutcome,
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
};
