mod coordinator;
mod failure;
mod failure_ledger;
mod phase;
mod terminal;

pub(crate) use coordinator::{
    ProductShutdownCoordinator, ProductShutdownPhaseDisposition, ProductShutdownSnapshot,
    ProductShutdownTransition, ProductShutdownTransitionError,
};
pub(crate) use failure::{ProductFailureRecord, ProductFailureReport, ProductFailureSeverity};
pub(crate) use failure_ledger::{
    ProductFailureLedger, PRODUCT_FAILURE_LEDGER_CAPACITY, PRODUCT_FAILURE_MESSAGE_BYTES,
};
pub(crate) use phase::ProductHostPhase;
pub(crate) use terminal::ProductTerminalReason;
pub use terminal::{ProductExitClass, ProductProcessExitCode};

#[cfg(test)]
mod tests;
