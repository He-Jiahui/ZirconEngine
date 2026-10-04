//! 回放已通过整批验证的日志；领域策略决定恢复旧代际还是只清理保留产物。
//! Durable cleanup phases preserve retry evidence and never authorize live-file mutation.

use std::io;
use std::path::Path;

use super::super::commit::{cleanup_documents, cleanup_documents_journal_first, restore_document};
use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::journal::{record_phase, record_state};
use super::super::schema::{FoldedTransactionJournal, JournalDocument, JournalPhase, JournalState};
use super::RecoveryMode;

pub(super) fn recover_journal(
    path: &Path,
    journal: &FoldedTransactionJournal,
    mode: RecoveryMode,
) -> Result<(), DurableTransactionError> {
    let cleanup_phase = match journal.phase {
        JournalPhase::Intent => JournalPhase::CleanupIntent,
        JournalPhase::Active if mode == RecoveryMode::RestoreOriginal => {
            return recover_active_journal(path, journal);
        }
        JournalPhase::Active => JournalPhase::CleanupActive,
        JournalPhase::RollbackCompleted => JournalPhase::CleanupRollback,
        JournalPhase::AllCommitted => JournalPhase::Cleanup,
        JournalPhase::CleanupActive if mode != RecoveryMode::CleanupArtifacts => {
            return Err(DurableTransactionError::invalid(
                path,
                "cleanup_active requires CleanupArtifacts recovery mode",
            ));
        }
        JournalPhase::CleanupIntent
        | JournalPhase::CleanupActive
        | JournalPhase::CleanupRollback
        | JournalPhase::Cleanup => {
            return cleanup_documents(path, &journal.documents, TransactionPhase::Recovery);
        }
    };
    recover_cleanup_journal_with(path, &journal.documents, cleanup_phase, mode, |phase| {
        record_phase(path, phase)
    })
}

pub(super) fn recover_cleanup_journal_with(
    path: &Path,
    documents: &[JournalDocument],
    cleanup_phase: JournalPhase,
    mode: RecoveryMode,
    mut record_phase: impl FnMut(JournalPhase) -> Result<(), DurableTransactionError>,
) -> Result<(), DurableTransactionError> {
    match record_phase(cleanup_phase) {
        Ok(()) => cleanup_documents(path, documents, TransactionPhase::Recovery),
        // A failed append may leave a torn frame. CleanupArtifacts must retain every artifact.
        Err(error) if mode == RecoveryMode::CleanupArtifacts => Err(error),
        Err(_) => cleanup_documents_journal_first(path, documents),
    }
}

// 逆序撤销可能已经发布的文档；状态追加失败时仍可幂等恢复 live 文件，不能据此继续追加不可信日志。
fn recover_active_journal(
    path: &Path,
    journal: &FoldedTransactionJournal,
) -> Result<(), DurableTransactionError> {
    recover_active_journal_with(
        path,
        journal,
        |index| record_state(path, index, JournalState::RollingBack),
        |phase| record_phase(path, phase),
        restore_document,
    )
}

pub(super) fn recover_active_journal_with(
    path: &Path,
    journal: &FoldedTransactionJournal,
    mut record_rolling_back: impl FnMut(usize) -> Result<(), DurableTransactionError>,
    mut record_recovery_phase: impl FnMut(JournalPhase) -> Result<(), DurableTransactionError>,
    mut restore: impl FnMut(&JournalDocument) -> io::Result<()>,
) -> Result<(), DurableTransactionError> {
    let mut journal_append_safe = true;
    let mut first_restore_error = None;
    for (index, document) in journal.documents.iter().enumerate().rev() {
        if !matches!(
            document.state,
            JournalState::Committing | JournalState::Committed | JournalState::RollingBack
        ) {
            continue;
        }
        if journal_append_safe
            && document.state != JournalState::RollingBack
            && record_rolling_back(index).is_err()
        {
            // The failed append can leave a torn tail. Restore remains idempotent, but no later
            // transition may be appended behind an uncertain frame.
            journal_append_safe = false;
        }
        if let Err(source) = restore(document) {
            first_restore_error.get_or_insert_with(|| {
                DurableTransactionError::operation(
                    TransactionPhase::Recovery,
                    &document.target,
                    source,
                )
            });
        }
    }
    if let Some(error) = first_restore_error {
        return Err(error);
    }
    if !journal_append_safe {
        return cleanup_documents_journal_first(path, &journal.documents);
    }
    if record_recovery_phase(JournalPhase::RollbackCompleted).is_err() {
        return cleanup_documents_journal_first(path, &journal.documents);
    }
    if record_recovery_phase(JournalPhase::CleanupRollback).is_err() {
        return cleanup_documents_journal_first(path, &journal.documents);
    }
    cleanup_documents(path, &journal.documents, TransactionPhase::Recovery)
}
