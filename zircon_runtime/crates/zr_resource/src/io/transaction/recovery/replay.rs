use std::io;
use std::path::Path;

use super::super::commit::{cleanup_documents, cleanup_documents_journal_first, restore_document};
use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::journal::{record_phase, record_state};
use super::super::schema::{FoldedTransactionJournal, JournalDocument, JournalPhase, JournalState};

pub(super) fn recover_journal(
    path: &Path,
    journal: &FoldedTransactionJournal,
) -> Result<(), DurableTransactionError> {
    match journal.phase {
        JournalPhase::Intent => recover_intent_journal(path, &journal.documents),
        JournalPhase::CleanupIntent => {
            cleanup_documents(path, &journal.documents, TransactionPhase::Recovery)
        }
        JournalPhase::Active => recover_active_journal(path, journal),
        JournalPhase::RollbackCompleted => {
            match record_phase(path, JournalPhase::CleanupRollback) {
                Ok(()) => cleanup_documents(path, &journal.documents, TransactionPhase::Recovery),
                Err(_) => cleanup_documents_journal_first(path, &journal.documents),
            }
        }
        JournalPhase::CleanupRollback | JournalPhase::Cleanup => {
            cleanup_documents(path, &journal.documents, TransactionPhase::Recovery)
        }
        JournalPhase::AllCommitted => match record_phase(path, JournalPhase::Cleanup) {
            Ok(()) => cleanup_documents(path, &journal.documents, TransactionPhase::Recovery),
            Err(_) => cleanup_documents_journal_first(path, &journal.documents),
        },
    }
}

fn recover_intent_journal(
    path: &Path,
    documents: &[JournalDocument],
) -> Result<(), DurableTransactionError> {
    recover_intent_journal_with(path, documents, |phase| record_phase(path, phase))
}

pub(super) fn recover_intent_journal_with(
    path: &Path,
    documents: &[JournalDocument],
    mut record_phase: impl FnMut(JournalPhase) -> Result<(), DurableTransactionError>,
) -> Result<(), DurableTransactionError> {
    if record_phase(JournalPhase::CleanupIntent).is_err() {
        return cleanup_documents_journal_first(path, documents);
    }
    cleanup_documents(path, documents, TransactionPhase::Recovery)
}

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
