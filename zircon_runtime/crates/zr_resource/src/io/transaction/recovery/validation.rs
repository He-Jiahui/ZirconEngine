use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::owner_lock::owner_lock_path;
use super::super::pathing::{
    PathIdentity, journal_path as expected_journal_path, transaction_sibling, valid_tag,
    valid_transaction_id,
};
use super::super::schema::{FoldedTransactionJournal, JournalDocument, JournalPhase, JournalState};
use super::super::stage::ensure_regular_or_missing;
use super::RecoveryPolicy;
use super::evidence::validate_document_evidence;

pub(super) fn validate_journals(
    journals: &[(PathBuf, FoldedTransactionJournal, usize)],
    expected_tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<(), DurableTransactionError> {
    let mut identities = BTreeSet::new();
    for (journal_path, journal, _) in journals {
        let journal_directory = journal_path.parent().unwrap_or_else(|| Path::new("."));
        let journal_directory = PathIdentity::resolve(journal_directory)
            .map_err(|source| operation(journal_path, source))?;
        let owner_lock = owner_lock_path(journal_path.parent().unwrap_or_else(|| Path::new(".")))
            .map_err(|source| operation(journal_path, source))?;
        let owner_lock =
            PathIdentity::resolve(&owner_lock).map_err(|source| operation(journal_path, source))?;
        if !valid_tag(&journal.tag) || journal.tag != expected_tag {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "journal transaction tag does not match its owner",
            ));
        }
        if !valid_transaction_id(&journal.transaction_id, &journal_directory) {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "invalid transaction id",
            ));
        }
        if journal.documents.is_empty() {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "empty transaction journal",
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
                &owner_lock,
                &journal_directory,
            )?;
        }
    }

    reject_namespace_overlaps(&identities)?;

    let mut evidence = Default::default();
    for (journal_path, journal, _) in journals {
        for document in &journal.documents {
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
        JournalPhase::Intent | JournalPhase::CleanupIntent => {
            journal.documents.iter().all(|document| {
                matches!(
                    document.state,
                    JournalState::Intent | JournalState::Prepared
                )
            })
        }
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

pub(super) fn validate_document_paths(
    journal_path: &Path,
    tag: &str,
    transaction_id: &str,
    document: &JournalDocument,
    identities: &mut BTreeSet<PathIdentity>,
    owner_lock: &PathIdentity,
    journal_directory: &PathIdentity,
) -> Result<(), DurableTransactionError> {
    let document_paths = [
        Some(document.target.as_path()),
        Some(document.staging.as_path()),
        Some(document.backup.as_path()),
        Some(document.rollback_staging.as_path()),
    ]
    .into_iter()
    .flatten();
    let retirement_paths = document.retirements.iter().flat_map(|retirement| {
        [
            retirement.path.as_path(),
            retirement.backup.as_path(),
            retirement.rollback_staging.as_path(),
        ]
    });
    let paths = document_paths.chain(retirement_paths).collect::<Vec<_>>();
    for path in paths {
        if !path.is_absolute() {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!("transaction path is not absolute: {}", path.display()),
            ));
        }
        let identity = resolve_journal_path_identity(journal_path, path)?;
        if !identity.has_exact_operation_path_encoding(path) {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!(
                    "transaction path is not a normalized physical path: {}",
                    path.display()
                ),
            ));
        }
        if identity.is_same_or_descendant_of(owner_lock)
            || owner_lock.is_same_or_descendant_of(&identity)
        {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!(
                    "transaction path overlaps the owner lock namespace: {}",
                    path.display()
                ),
            ));
        }
        if identity.is_same_or_descendant_of(journal_directory)
            || journal_directory.is_same_or_descendant_of(&identity)
        {
            return Err(DurableTransactionError::invalid(
                journal_path,
                format!(
                    "transaction path overlaps the journal owner namespace: {}",
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
    for retirement in &document.retirements {
        validate_role(
            journal_path,
            &retirement.path,
            &retirement.backup,
            tag,
            "retired-backup",
            transaction_id,
        )?;
        validate_role(
            journal_path,
            &retirement.path,
            &retirement.rollback_staging,
            tag,
            "retired-rollback-stage",
            transaction_id,
        )?;
    }
    Ok(())
}

fn reject_namespace_overlaps(
    identities: &BTreeSet<PathIdentity>,
) -> Result<(), DurableTransactionError> {
    for identity in identities {
        let ancestors = identity
            .strict_ancestor_identities()
            .map_err(|source| operation(identity.operation_path(), source))?;
        if ancestors
            .into_iter()
            .any(|ancestor| identities.contains(&ancestor))
        {
            return Err(DurableTransactionError::invalid(
                identity.operation_path(),
                "transaction paths overlap an ancestor or descendant",
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

pub(super) fn validate_regular_directory(path: &Path) -> Result<(), DurableTransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| operation(path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(DurableTransactionError::invalid(
            path,
            "transaction journal owner must be a real directory",
        ));
    }
    Ok(())
}

pub(super) fn ensure_regular_file(path: &Path) -> Result<fs::Metadata, DurableTransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| operation(path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DurableTransactionError::invalid(
            path,
            "journal entry must be a regular non-link file",
        ));
    }
    Ok(metadata)
}

pub(super) fn resolve_recovery_directory(path: &Path) -> Result<PathBuf, DurableTransactionError> {
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

pub(super) fn operation(path: &Path, source: io::Error) -> DurableTransactionError {
    DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
}
