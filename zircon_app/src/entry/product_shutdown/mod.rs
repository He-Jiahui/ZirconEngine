#[path = "coordinator.rs"]
mod coordinator;
#[path = "failure.rs"]
mod failure;
#[path = "failure_ledger.rs"]
mod failure_ledger;
#[path = "phase.rs"]
mod phase;
#[path = "retained_owner/mod.rs"]
pub(in crate::entry) mod retained_owner;
#[path = "terminal.rs"]
mod terminal;
#[path = "terminal_outcome.rs"]
mod terminal_outcome;

pub(crate) use coordinator::{
    ProductShutdownCoordinator, ProductShutdownPhaseDisposition, ProductShutdownSnapshot,
    ProductShutdownTransition, ProductShutdownTransitionError,
};
pub(crate) use failure::{ProductFailureRecord, ProductFailureReport, ProductFailureSeverity};
pub(crate) use failure_ledger::{
    ProductFailureLedger, PRODUCT_FAILURE_LEDGER_CAPACITY, PRODUCT_FAILURE_MESSAGE_BYTES,
};
pub(crate) use phase::ProductHostPhase;
pub use retained_owner::{
    retry_product_cleanup_until, ProductCloseError, ProductCompositionFailure,
};
pub(crate) use terminal::ProductTerminalReason;
pub use terminal::{ProductExitClass, ProductProcessExitCode};
pub use terminal_outcome::{
    ProductTerminalOutcome, ProductTerminalPrimary, ProductTerminalReceipt,
    ProductTerminalSecondary, ProductTerminalStatus, PRODUCT_TERMINAL_RECEIPT_SCHEMA_VERSION,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
