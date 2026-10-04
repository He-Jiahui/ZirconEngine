//! Crash-recoverable publication of one file generation.
//!
//! Callers own semantic path policy. This module owns the immutable intent, append-only state
//! transitions, staged evidence, platform publication, rollback, and restart decision.

mod commit;
mod engine;
mod error;
mod journal;
mod observation;
mod owner_lock;
mod pathing;
mod recovery;
mod schema;
mod stage;

/// Identifies canonical project transaction stage, backup, and rollback-stage siblings.
pub fn is_project_transaction_sibling_path(path: &std::path::Path) -> bool {
    pathing::is_project_transaction_sibling_path(path)
}

pub use engine::{commit_prepared_files, DurableCommitDisposition, PreparedFileWrite};
pub use error::{DurableTransactionError, TransactionPhase};
pub use observation::{DurableCommitReport, DurableRecoveryReport};
pub use recovery::{
    detect_pending_transactions, recover_pending_transactions, RecoveryMode, RecoveryPolicy,
};
pub use schema::{JournalDocument, TransactionFault};
