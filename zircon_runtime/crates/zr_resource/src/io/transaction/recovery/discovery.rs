use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::error::DurableTransactionError;
use super::super::journal::{MAX_JOURNAL_BYTES, decode_journal_with_valid_len};
use super::super::schema::{FoldedTransactionJournal, JOURNAL_VERSION};
use super::RecoveryPolicy;
use super::validation::{
    ensure_regular_file, operation, validate_journals, validate_regular_directory,
};
use crate::io::is_atomic_write_transaction_path;

pub(super) struct PendingTransactions {
    pub(super) journals: Vec<(PathBuf, FoldedTransactionJournal, usize)>,
    pub(super) atomic_intent_orphans: Vec<PathBuf>,
}

pub(super) fn load_pending_transactions(
    directory: &Path,
    tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<PendingTransactions, DurableTransactionError> {
    match fs::symlink_metadata(directory) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(PendingTransactions {
                journals: Vec::new(),
                atomic_intent_orphans: Vec::new(),
            });
        }
        Err(source) => return Err(operation(directory, source)),
    }
    validate_regular_directory(directory)?;
    let mut paths = fs::read_dir(directory)
        .map_err(|source| operation(directory, source))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| operation(directory, source))?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();

    let mut journals = Vec::with_capacity(paths.len());
    let mut atomic_intent_orphans = Vec::new();
    for path in paths {
        let metadata = ensure_regular_file(&path)?;
        if path.extension().and_then(|value| value.to_str()) != Some("zrjournal") {
            if is_atomic_intent_orphan(&path) {
                atomic_intent_orphans.push(path);
                continue;
            }
            return Err(DurableTransactionError::invalid(
                &path,
                "unsupported file in durable transaction journal directory",
            ));
        }
        if metadata.len() > MAX_JOURNAL_BYTES as u64 {
            return Err(DurableTransactionError::invalid(
                &path,
                "durable transaction journal exceeds its bounded size",
            ));
        }
        let bytes = fs::read(&path).map_err(|source| operation(&path, source))?;
        let (parsed, valid_len) = decode_journal_with_valid_len(&path, &bytes)?;
        if parsed.version != JOURNAL_VERSION {
            return Err(DurableTransactionError::invalid(
                &path,
                "unsupported durable transaction journal version",
            ));
        }
        if parsed.documents.is_empty() {
            return Err(DurableTransactionError::invalid(
                &path,
                "empty transaction journal",
            ));
        }
        let folded = parsed
            .fold()
            .map_err(|reason| DurableTransactionError::invalid(&path, reason))?;
        journals.push((path, folded, valid_len));
    }
    validate_journals(&journals, tag, policy)?;
    Ok(PendingTransactions {
        journals,
        atomic_intent_orphans,
    })
}

fn is_atomic_intent_orphan(path: &Path) -> bool {
    if !is_atomic_write_transaction_path(path) {
        return false;
    }
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    [".zr-staging-", ".zr-backup-"]
        .into_iter()
        .find_map(|marker| file_name.rsplit_once(marker).map(|(target, _)| target))
        .and_then(|target| target.strip_prefix('.'))
        .is_some_and(|target| target.ends_with(".zrjournal"))
}
