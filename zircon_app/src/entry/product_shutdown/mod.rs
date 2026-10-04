mod coordinator;
mod failure;
mod failure_ledger;
mod phase;
pub(in crate::entry) mod retained_owner;
mod terminal;
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
