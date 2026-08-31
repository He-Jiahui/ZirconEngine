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

pub use engine::{DurableCommitDisposition, PreparedFileWrite, commit_prepared_files};
pub use error::{DurableTransactionError, TransactionPhase};
pub use observation::{DurableCommitReport, DurableRecoveryReport};
pub use recovery::{RecoveryPolicy, detect_pending_transactions, recover_pending_transactions};
pub use schema::{JournalDocument, TransactionFault};
