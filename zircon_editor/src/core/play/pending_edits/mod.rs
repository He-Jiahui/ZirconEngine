mod intent;
mod queue;
mod resolution;

#[cfg(test)]
mod tests;

pub use intent::{PendingEditId, PendingEditIntent};
pub use queue::{
    PendingEditApplyBudget, PendingEditEnqueueReport, PendingEditPage, PendingEditPageCursor,
    PendingEditPageEntry, PendingEditQueue, PendingEditQueueError, PendingEditQueueLimits,
    PendingEditQueueSummary,
};
pub use resolution::{
    PendingEditApplyFailure, PendingEditApplyReport, PendingEditDecisionPrompt,
    PendingEditDiscardReport, PendingEditExitDecision,
};
