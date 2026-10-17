//! Crash-recoverable publication of one file generation.
//!
//! Callers own semantic path policy. This module owns the immutable intent, append-only state
//! transitions, staged evidence, platform publication, rollback, and restart decision.

#[path = "commit.rs"]
mod commit;
#[path = "engine.rs"]
mod engine;
#[path = "error.rs"]
mod error;
#[path = "journal/mod.rs"]
mod journal;
#[path = "observation.rs"]
mod observation;
#[path = "owner_lock.rs"]
mod owner_lock;
#[path = "pathing.rs"]
mod pathing;
#[path = "recovery/mod.rs"]
mod recovery;
#[path = "schema.rs"]
mod schema;
#[path = "stage.rs"]
mod stage;

/// Identifies canonical project transaction stage, backup, and rollback-stage siblings.
pub fn is_project_transaction_sibling_path(path: &std::path::Path) -> bool {
    pathing::is_project_transaction_sibling_path(path)
}

pub use engine::{
    commit_prepared_files, commit_prepared_files_with_revisions, DurableCommitDisposition,
    DurableCommitReceipt, PreparedFileWrite,
};
pub use error::{DurableTransactionError, TransactionPhase};
pub use observation::{DurableCommitReport, DurableRecoveryReport};
pub use recovery::{
    detect_pending_transactions, recover_pending_transactions, RecoveryMode, RecoveryPolicy,
};
pub use schema::{JournalDocument, TransactionFault};
