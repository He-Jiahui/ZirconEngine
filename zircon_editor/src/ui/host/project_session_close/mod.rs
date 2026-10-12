#[path = "commit.rs"]
mod commit;
#[path = "coordinator.rs"]
mod coordinator;
#[path = "effect_receipt.rs"]
mod effect_receipt;
#[path = "error.rs"]
mod error;
#[path = "operation.rs"]
mod operation;
#[path = "persistence.rs"]
mod persistence;
#[path = "phase.rs"]
mod phase;
#[path = "receipt.rs"]
mod receipt;
#[path = "transition_error.rs"]
mod transition_error;

pub(crate) use commit::ProjectCloseCommit;
pub(crate) use coordinator::ProjectCloseCoordinator;
pub(crate) use effect_receipt::ProjectCloseEffectReceipt;
pub(crate) use error::ProjectCloseError;
pub(crate) use operation::ProjectCloseOperation;
pub(crate) use phase::ProjectCloseCoordinatorPhase;
pub(crate) use receipt::ProjectCloseReceipt;
pub(crate) use transition_error::ProjectCloseTransitionError;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
