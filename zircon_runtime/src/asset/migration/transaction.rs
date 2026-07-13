use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::document::PendingDocument;
use super::{AssetMigrationError, AssetMigrationTransactionPhase};

static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(1);

mod commit;
mod journal;
mod journal_owner;
mod schema;
mod stage;
use commit::{cleanup_committed_artifacts, commit_document, rollback_and_cleanup, should_fail};
use journal::{create_intent_journal, sync_journal};
use schema::JournalPhase;
pub(super) use schema::{CommitFault, JOURNAL_DIRECTORY};
use stage::{cleanup_staging, stage_document, StagedDocument};

pub(super) fn apply_transaction(
    project_root: &Path,
    pending: Vec<PendingDocument>,
    fault: CommitFault,
) -> Result<(), AssetMigrationError> {
    if pending.is_empty() {
        return Ok(());
    }
    let transaction_id = next_transaction_id();
    let journal = create_intent_journal(project_root, &pending, &transaction_id)?;
    let mut staged = Vec::with_capacity(pending.len());
    for (index, document) in pending.into_iter().enumerate() {
        match stage_document(document, &transaction_id, fault, index) {
            Ok(document) => {
                staged.push(document);
                #[cfg(test)]
                if matches!(fault, CommitFault::CrashAfterStaging(fault_index) if fault_index == index)
                {
                    return Err(transaction_error(
                        AssetMigrationTransactionPhase::Stage,
                        journal,
                        io::Error::new(
                            io::ErrorKind::Interrupted,
                            "injected post-stage process interruption",
                        ),
                    ));
                }
            }
            Err(error) => {
                cleanup_staging(&staged);
                let _ = remove_if_exists(&journal);
                return Err(error);
            }
        }
    }
    sync_journal(&journal, &staged, JournalPhase::Active)?;

    for index in 0..staged.len() {
        if should_fail(fault, index) {
            let path = staged[index].target.clone();
            let source = io::Error::new(io::ErrorKind::Other, "injected migration commit failure");
            rollback_and_cleanup(&journal, &mut staged, index, fault)?;
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Commit,
                path,
                source,
            ));
        }
        staged[index].committing = true;
        sync_journal(&journal, &staged, JournalPhase::Active)?;
        if let Err(source) = commit_document(&mut staged[index], fault, index) {
            let path = staged[index].target.clone();
            #[cfg(test)]
            if matches!(
                fault,
                CommitFault::CrashAfterTargetReplace(fault_index)
                    | CommitFault::CrashAfterRetiredDelete(fault_index)
                    if fault_index == index
            ) {
                return Err(transaction_error(
                    AssetMigrationTransactionPhase::Commit,
                    path,
                    source,
                ));
            }
            rollback_and_cleanup(&journal, &mut staged, index + 1, fault)?;
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Commit,
                path,
                source,
            ));
        }
        staged[index].committing = false;
        if let Err(error) = sync_journal(&journal, &staged, JournalPhase::Active) {
            rollback_and_cleanup(&journal, &mut staged, index + 1, fault)?;
            return Err(error);
        }
        #[cfg(test)]
        if matches!(fault, CommitFault::CrashAfter(crash_index) if crash_index == index) {
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Commit,
                staged[index].target.clone(),
                io::Error::new(io::ErrorKind::Interrupted, "injected process interruption"),
            ));
        }
    }

    sync_journal(&journal, &staged, JournalPhase::AllCommitted)?;
    #[cfg(test)]
    if fault == CommitFault::CrashAfterAllCommitted {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Commit,
            journal,
            io::Error::new(
                io::ErrorKind::Interrupted,
                "injected all-committed interruption",
            ),
        ));
    }
    sync_journal(&journal, &staged, JournalPhase::Cleanup)?;
    #[cfg(test)]
    if fault == CommitFault::CrashAfterCleanup {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Commit,
            journal,
            io::Error::new(io::ErrorKind::Interrupted, "injected cleanup interruption"),
        ));
    }
    cleanup_committed_artifacts(&journal, &staged)?;
    Ok(())
}

mod recovery;

pub(in crate::asset::migration) use recovery::{
    detect_pending_transactions, recover_pending_transactions,
};

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn next_transaction_id() -> String {
    let id = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
    format!("{}-{id}", std::process::id())
}

pub(super) fn valid_transaction_id(value: &str) -> bool {
    let mut parts = value.split('-');
    parts.next().is_some_and(|part| part.parse::<u32>().is_ok())
        && parts.next().is_some_and(|part| part.parse::<u64>().is_ok())
        && parts.next().is_none()
}

fn transaction_sibling(parent: &Path, target: &Path, role: &str, transaction_id: &str) -> PathBuf {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    parent.join(format!(".{name}.zr-migrate-{role}-{transaction_id}"))
}

fn digest_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub(super) fn digest_file(path: &Path) -> io::Result<String> {
    fs::read(path).map(|bytes| digest_bytes(&bytes))
}

pub(super) fn transaction_error(
    phase: AssetMigrationTransactionPhase,
    path: PathBuf,
    source: io::Error,
) -> AssetMigrationError {
    AssetMigrationError::Transaction {
        phase,
        path,
        source,
    }
}
