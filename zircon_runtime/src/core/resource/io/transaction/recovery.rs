use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::commit::{cleanup_documents, cleanup_documents_journal_first, restore_document};
use super::error::{DurableTransactionError, TransactionPhase};
use super::journal::{
    decode_journal_with_valid_len, record_phase, record_state, truncate_torn_tail,
    MAX_JOURNAL_BYTES,
};
use super::observation::DurableRecoveryReport;
use super::owner_lock::TransactionOwnerLock;
use super::pathing::{
    journal_path as expected_journal_path, transaction_sibling, valid_tag, valid_transaction_id,
    PathIdentity,
};
use super::schema::{
    FoldedTransactionJournal, JournalDocument, JournalPhase, JournalState, JOURNAL_VERSION,
};
use super::stage::{digest_file, ensure_regular_or_missing, remove_reserved_if_exists};
use crate::core::resource::io::is_atomic_write_transaction_path;

pub(crate) trait RecoveryPolicy {
    fn validate_document(
        &self,
        journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String>;

    fn digest_file(&mut self, path: &Path) -> io::Result<String> {
        digest_file(path)
    }
}

pub(crate) fn detect_pending_transactions(
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

pub(crate) fn recover_pending_transactions(
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
        let rolls_back = journal.phase == JournalPhase::Active;
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

struct PendingTransactions {
    journals: Vec<(PathBuf, FoldedTransactionJournal, usize)>,
    atomic_intent_orphans: Vec<PathBuf>,
}

fn load_pending_transactions(
    directory: &Path,
    tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<PendingTransactions, DurableTransactionError> {
    if !directory.exists() {
        return Ok(PendingTransactions {
            journals: Vec::new(),
            atomic_intent_orphans: Vec::new(),
        });
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

fn validate_journals(
    journals: &[(PathBuf, FoldedTransactionJournal, usize)],
    expected_tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<(), DurableTransactionError> {
    let mut identities = BTreeSet::new();
    let mut evidence = EvidenceCache::default();
    for (journal_path, journal, _) in journals {
        if !valid_tag(&journal.tag) || journal.tag != expected_tag {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "journal transaction tag does not match its owner",
            ));
        }
        if !valid_transaction_id(&journal.transaction_id) {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "invalid transaction id",
            ));
        }
        let expected_name = expected_journal_path(
            journal_path.parent().unwrap_or_else(|| Path::new(".")),
            &journal.documents[0].target,
            &journal.tag,
            &journal.transaction_id,
        );
        if resolve_journal_path_identity(journal_path, &expected_name)?
            != resolve_journal_path_identity(journal_path, journal_path)?
        {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "journal filename does not match immutable intent",
            ));
        }
        validate_phase(journal_path, journal)?;
        for document in &journal.documents {
            policy
                .validate_document(journal_path, document)
                .map_err(|reason| DurableTransactionError::invalid(journal_path, reason))?;
            validate_document_paths(
                journal_path,
                &journal.tag,
                &journal.transaction_id,
                document,
                &mut identities,
            )?;
            validate_document_evidence(
                journal_path,
                journal.phase,
                document,
                policy,
                &mut evidence,
            )?;
        }
    }
    Ok(())
}

fn validate_phase(
    path: &Path,
    journal: &FoldedTransactionJournal,
) -> Result<(), DurableTransactionError> {
    let valid = match journal.phase {
        JournalPhase::Intent => journal.documents.iter().all(|document| {
            matches!(
                document.state,
                JournalState::Intent | JournalState::Prepared
            )
        }),
        JournalPhase::Active => journal.documents.iter().all(|document| {
            matches!(
                document.state,
                JournalState::Prepared
                    | JournalState::Committing
                    | JournalState::Committed
                    | JournalState::RollingBack
            )
        }),
        JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback => journal
            .documents
            .iter()
            .all(|document| document.state == JournalState::Prepared),
        JournalPhase::AllCommitted | JournalPhase::Cleanup => journal
            .documents
            .iter()
            .all(|document| document.state == JournalState::Committed),
    };
    if valid {
        Ok(())
    } else {
        Err(DurableTransactionError::invalid(
            path,
            "journal phase and document states disagree",
        ))
    }
}

fn validate_document_paths(
    journal_path: &Path,
    tag: &str,
    transaction_id: &str,
    document: &JournalDocument,
    identities: &mut BTreeSet<PathIdentity>,
) -> Result<(), DurableTransactionError> {
    let paths = [
        Some(document.target.as_path()),
        Some(document.staging.as_path()),
        Some(document.backup.as_path()),
        Some(document.rollback_staging.as_path()),
        document.retired_path.as_deref(),
        document.retired_backup.as_deref(),
        document.retired_rollback_staging.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for path in paths {
        if !path.is_absolute() {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!("transaction path is not absolute: {}", path.display()),
            ));
        }
        let identity = resolve_journal_path_identity(journal_path, path)?;
        if identity.operation_path() != path {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!(
                    "transaction path is not a normalized physical path: {}",
                    path.display()
                ),
            ));
        }
        if !identities.insert(identity) {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!("transaction path aliases another path: {}", path.display()),
            ));
        }
        ensure_regular_or_missing(path).map_err(|error| {
            DurableTransactionError::invalid(
                journal_path,
                format!("invalid transaction path {}: {error}", path.display()),
            )
        })?;
    }

    validate_role(
        journal_path,
        &document.target,
        &document.staging,
        tag,
        "stage",
        transaction_id,
    )?;
    validate_role(
        journal_path,
        &document.target,
        &document.backup,
        tag,
        "backup",
        transaction_id,
    )?;
    validate_role(
        journal_path,
        &document.target,
        &document.rollback_staging,
        tag,
        "rollback-stage",
        transaction_id,
    )?;
    match (
        document.retired_path.as_deref(),
        document.retired_backup.as_deref(),
        document.retired_rollback_staging.as_deref(),
    ) {
        (Some(retired), Some(backup), Some(rollback)) => {
            validate_role(
                journal_path,
                retired,
                backup,
                tag,
                "retired-backup",
                transaction_id,
            )?;
            validate_role(
                journal_path,
                retired,
                rollback,
                tag,
                "retired-rollback-stage",
                transaction_id,
            )?;
        }
        (None, None, None) => {}
        _ => {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "retired file recovery paths are incomplete",
            ));
        }
    }
    Ok(())
}

fn validate_role(
    journal_path: &Path,
    owner: &Path,
    artifact: &Path,
    tag: &str,
    role: &str,
    transaction_id: &str,
) -> Result<(), DurableTransactionError> {
    let expected = transaction_sibling(owner, tag, role, transaction_id);
    if resolve_journal_path_identity(journal_path, &expected)?
        == resolve_journal_path_identity(journal_path, artifact)?
    {
        Ok(())
    } else {
        Err(DurableTransactionError::invalid(
            journal_path,
            format!("{role} artifact does not match its reserved sibling path"),
        ))
    }
}

fn validate_document_evidence(
    journal_path: &Path,
    phase: JournalPhase,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if document.state == JournalState::Intent {
        return Ok(());
    }
    let target_existed = document.target_existed.ok_or_else(|| {
        DurableTransactionError::invalid(journal_path, "missing target origin evidence")
    })?;
    let new_digest = document.new_digest.as_deref().ok_or_else(|| {
        DurableTransactionError::invalid(journal_path, "missing staged digest evidence")
    })?;
    if target_existed != document.original_digest.is_some() {
        return Err(DurableTransactionError::invalid(
            journal_path,
            "target origin and original digest disagree",
        ));
    }
    if document.retired_path.is_some() != document.retired_digest.is_some() {
        return Err(DurableTransactionError::invalid(
            journal_path,
            "retired file and digest evidence disagree",
        ));
    }

    let cleanup = matches!(phase, JournalPhase::Cleanup | JournalPhase::CleanupRollback);
    match phase {
        JournalPhase::Intent | JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback => {
            require_original_target(journal_path, document, policy, evidence)?;
            require_retired_original(journal_path, document, policy, evidence)?;
        }
        JournalPhase::Active => match document.state {
            JournalState::Prepared => {
                require_original_target(journal_path, document, policy, evidence)?;
                require_retired_original(journal_path, document, policy, evidence)?;
            }
            JournalState::Committing | JournalState::Committed | JournalState::RollingBack => {
                require_original_or_new_target(
                    journal_path,
                    document,
                    new_digest,
                    policy,
                    evidence,
                )?;
                require_retired_original_or_missing(journal_path, document, policy, evidence)?;
            }
            JournalState::Intent => unreachable!("handled before evidence validation"),
        },
        JournalPhase::AllCommitted | JournalPhase::Cleanup => {
            require_digest(
                journal_path,
                &document.target,
                new_digest,
                false,
                "committed target",
                policy,
                evidence,
            )?;
            require_retired_missing(journal_path, document)?;
        }
    }

    let staging_may_be_missing = cleanup
        || phase == JournalPhase::RollbackCompleted
        || matches!(
            document.state,
            JournalState::Committing | JournalState::Committed | JournalState::RollingBack
        );
    require_digest(
        journal_path,
        &document.staging,
        new_digest,
        staging_may_be_missing,
        "staging artifact",
        policy,
        evidence,
    )?;
    if let Some(original) = document.original_digest.as_deref() {
        require_digest(
            journal_path,
            &document.backup,
            original,
            cleanup,
            "backup artifact",
            policy,
            evidence,
        )?;
    } else if document.backup.exists() {
        return Err(DurableTransactionError::invalid(
            journal_path,
            "backup exists for a newly-created target",
        ));
    }
    if let (Some(backup), Some(digest)) = (
        document.retired_backup.as_deref(),
        document.retired_digest.as_deref(),
    ) {
        require_digest(
            journal_path,
            backup,
            digest,
            cleanup,
            "retired backup",
            policy,
            evidence,
        )?;
    }
    Ok(())
}

fn require_original_target(
    journal_path: &Path,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    match document.original_digest.as_deref() {
        Some(digest) => require_digest(
            journal_path,
            &document.target,
            digest,
            false,
            "original target",
            policy,
            evidence,
        ),
        None if document.target.exists() => Err(DurableTransactionError::invalid(
            journal_path,
            "new target exists before transaction publication",
        )),
        None => Ok(()),
    }
}

fn require_original_or_new_target(
    journal_path: &Path,
    document: &JournalDocument,
    new_digest: &str,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if !document.target.exists() {
        return if document.target_existed == Some(false) {
            Ok(())
        } else {
            Err(DurableTransactionError::invalid(
                journal_path,
                "existing active target is missing",
            ))
        };
    }
    let actual = file_evidence(journal_path, &document.target, policy, evidence)?;
    if actual == new_digest
        || document
            .original_digest
            .as_deref()
            .is_some_and(|original| actual == original)
    {
        Ok(())
    } else {
        Err(DurableTransactionError::invalid(
            journal_path,
            "active target matches neither old nor new generation",
        ))
    }
}

fn require_retired_original(
    journal_path: &Path,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if let (Some(path), Some(digest)) = (
        document.retired_path.as_deref(),
        document.retired_digest.as_deref(),
    ) {
        require_digest(
            journal_path,
            path,
            digest,
            false,
            "retired live file",
            policy,
            evidence,
        )?;
    }
    Ok(())
}

fn require_retired_original_or_missing(
    journal_path: &Path,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if let (Some(path), Some(digest)) = (
        document.retired_path.as_deref(),
        document.retired_digest.as_deref(),
    ) {
        require_digest(
            journal_path,
            path,
            digest,
            true,
            "retired live file",
            policy,
            evidence,
        )?;
    }
    Ok(())
}

fn require_retired_missing(
    journal_path: &Path,
    document: &JournalDocument,
) -> Result<(), DurableTransactionError> {
    if document.retired_path.as_deref().is_some_and(Path::exists) {
        Err(DurableTransactionError::invalid(
            journal_path,
            "retired live file remains after committed generation",
        ))
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn require_digest(
    journal_path: &Path,
    path: &Path,
    expected: &str,
    allow_missing: bool,
    label: &str,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if !path.exists() {
        return if allow_missing {
            Ok(())
        } else {
            Err(DurableTransactionError::invalid(
                journal_path,
                format!("{label} is missing"),
            ))
        };
    }
    if file_evidence(journal_path, path, policy, evidence)? == expected {
        Ok(())
    } else {
        Err(DurableTransactionError::invalid(
            journal_path,
            format!("{label} digest does not match journal evidence"),
        ))
    }
}

fn file_evidence(
    journal_path: &Path,
    path: &Path,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<String, DurableTransactionError> {
    if let Some(digest) = evidence.digests.get(path) {
        return Ok(digest.clone());
    }
    let digest = policy.digest_file(path).map_err(|source| {
        DurableTransactionError::invalid(
            journal_path,
            format!("invalid transaction evidence {}: {source}", path.display()),
        )
    })?;
    evidence.digests.insert(path.to_path_buf(), digest.clone());
    Ok(digest)
}

fn recover_journal(
    path: &Path,
    journal: &FoldedTransactionJournal,
) -> Result<(), DurableTransactionError> {
    match journal.phase {
        JournalPhase::Intent => cleanup_documents_journal_first(path, &journal.documents),
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

fn recover_active_journal(
    path: &Path,
    journal: &FoldedTransactionJournal,
) -> Result<(), DurableTransactionError> {
    recover_active_journal_with(
        path,
        journal,
        |index| record_state(path, index, JournalState::RollingBack),
        |phase| record_phase(path, phase),
    )
}

fn recover_active_journal_with(
    path: &Path,
    journal: &FoldedTransactionJournal,
    mut record_rolling_back: impl FnMut(usize) -> Result<(), DurableTransactionError>,
    mut record_recovery_phase: impl FnMut(JournalPhase) -> Result<(), DurableTransactionError>,
) -> Result<(), DurableTransactionError> {
    let mut journal_append_safe = true;
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
        restore_document(document).map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, &document.target, source)
        })?;
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

fn validate_regular_directory(path: &Path) -> Result<(), DurableTransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| operation(path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(DurableTransactionError::invalid(
            path,
            "transaction journal owner must be a real directory",
        ));
    }
    Ok(())
}

fn ensure_regular_file(path: &Path) -> Result<fs::Metadata, DurableTransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| operation(path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DurableTransactionError::invalid(
            path,
            "journal entry must be a regular non-link file",
        ));
    }
    Ok(metadata)
}

fn resolve_recovery_directory(path: &Path) -> Result<PathBuf, DurableTransactionError> {
    if !path.is_absolute() {
        return Err(DurableTransactionError::invalid(
            path,
            "transaction journal directory must be absolute",
        ));
    }
    PathIdentity::resolve(path)
        .map(PathIdentity::into_operation_path)
        .map_err(|source| operation(path, source))
}

fn resolve_journal_path_identity(
    journal_path: &Path,
    path: &Path,
) -> Result<PathIdentity, DurableTransactionError> {
    PathIdentity::resolve(path).map_err(|error| {
        DurableTransactionError::invalid(
            journal_path,
            format!(
                "cannot resolve transaction path {} to a physical identity: {error}",
                path.display()
            ),
        )
    })
}

fn operation(path: &Path, source: io::Error) -> DurableTransactionError {
    DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
}

#[derive(Default)]
struct EvidenceCache {
    digests: HashMap<PathBuf, String>,
}

#[cfg(test)]
mod tests;
