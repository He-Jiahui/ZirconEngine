//! Restart recovery for durable transaction journals.

mod discovery;
mod evidence;
mod replay;
mod validation;

use std::io;
use std::path::{Path, PathBuf};

use discovery::load_pending_transactions;
use replay::recover_journal;
use validation::{operation, resolve_recovery_directory};

use super::error::{DurableTransactionError, TransactionPhase};
use super::journal::truncate_torn_tail;
use super::observation::DurableRecoveryReport;
use super::owner_lock::TransactionOwnerLock;
use super::schema::JournalDocument;
use super::stage::{digest_file, remove_reserved_if_exists};

/// Domain-owned validation and digest policy for a recovered journal document.
pub trait RecoveryPolicy {
    fn validate_document(
        &self,
        journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String>;

    fn digest_file(&mut self, path: &Path) -> io::Result<String> {
        digest_file(path)
    }
}

pub fn detect_pending_transactions(
    directory: &Path,
    tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<Vec<PathBuf>, DurableTransactionError> {
    let directory = resolve_recovery_directory(directory)?;
    let _owner = TransactionOwnerLock::acquire(&directory, TransactionPhase::Recovery)?;
    let pending = load_pending_transactions(&directory, tag, policy)?;
    let mut paths = pending
        .journals
        .into_iter()
        .map(|(path, _, _)| path)
        .chain(pending.atomic_intent_orphans)
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

pub fn recover_pending_transactions(
    directory: &Path,
    tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<DurableRecoveryReport, DurableTransactionError> {
    let directory = resolve_recovery_directory(directory)?;
    let _owner = TransactionOwnerLock::acquire(&directory, TransactionPhase::Recovery)?;
    let pending = load_pending_transactions(&directory, tag, policy)?;
    let orphan_count = pending.atomic_intent_orphans.len();
    for orphan in pending.atomic_intent_orphans {
        remove_reserved_if_exists(&orphan).map_err(|source| operation(&orphan, source))?;
    }
    let mut rollback_count = 0;
    let mut cleanup_count = 0;
    for (path, journal, valid_len) in pending.journals {
        truncate_torn_tail(&path, valid_len)?;
        let rolls_back = journal.phase == super::schema::JournalPhase::Active;
        recover_journal(&path, &journal)?;
        rollback_count += usize::from(rolls_back);
        cleanup_count += 1;
    }
    Ok(DurableRecoveryReport::new(
        rollback_count,
        cleanup_count,
        orphan_count,
    ))
}

#[cfg(test)]
mod tests;
