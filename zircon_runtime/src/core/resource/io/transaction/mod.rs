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

pub(crate) use engine::{commit_prepared_files, DurableCommitDisposition, PreparedFileWrite};
pub(crate) use error::{DurableTransactionError, TransactionPhase};
pub(crate) use observation::{DurableCommitReport, DurableRecoveryReport};
pub(crate) use recovery::{
    detect_pending_transactions, recover_pending_transactions, RecoveryPolicy,
};
pub(crate) use schema::{JournalDocument, TransactionFault};
