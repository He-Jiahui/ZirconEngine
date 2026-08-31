//! Curated Runtime projection of Resource I/O.

pub use zr_resource::io::{ArtifactIdentityExhausted, atomic_write, atomic_write_new};

pub(crate) use zr_resource::assembly::io::{
    AtomicWriteFault, PendingAtomicWrite, atomic_write_with_fault, ensure_parent_directories,
    is_atomic_write_transaction_path, recover_missing_target_from_backup, replace_staged_file,
    stage_atomic_write, sync_parent_directory,
};

pub(crate) mod transaction {
    pub(crate) use zr_resource::assembly::io::transaction::{
        DurableCommitDisposition, DurableCommitReport, DurableRecoveryReport,
        DurableTransactionError, JournalDocument, PreparedFileWrite, RecoveryPolicy,
        TransactionFault, TransactionPhase, commit_prepared_files, detect_pending_transactions,
        recover_pending_transactions,
    };
}
