//! 原子写入 API 与错误类型直接转发自 zr_resource；暂存、目录同步、替换、恢复及事务日志原语仅在 Runtime crate 内可见，供配置、资产和项目持久化流程组合。
//! Curated Runtime projection of Resource I/O.

pub use zr_resource::io::{atomic_write, atomic_write_new, ArtifactIdentityExhausted};

pub(crate) use zr_resource::assembly::io::{
    atomic_write_with_fault, ensure_parent_directories, is_atomic_write_transaction_path,
    recover_missing_target_from_backup, replace_staged_file, stage_atomic_write,
    sync_parent_directory, AtomicWriteFault, PendingAtomicWrite,
};

pub(crate) mod transaction {
    pub(crate) use zr_resource::assembly::io::transaction::{
        commit_prepared_files, detect_pending_transactions, is_project_transaction_sibling_path,
        recover_pending_transactions, DurableCommitDisposition, DurableCommitReport,
        DurableRecoveryReport, DurableTransactionError, JournalDocument, PreparedFileWrite,
        RecoveryMode, RecoveryPolicy, TransactionFault, TransactionPhase,
    };
}
