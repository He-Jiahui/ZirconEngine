use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::schema::{
    JournalDocument, JournalPhase, JournalState, TransactionJournal, JOURNAL_DIRECTORY,
    JOURNAL_VERSION,
};
use super::{digest_file, transaction_error, valid_transaction_id};
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};
use crate::asset::safe_project_path::is_link_or_reparse;

pub(in crate::asset::migration) fn recover_pending_transactions(
    project_root: &Path,
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<(), AssetMigrationError> {
    let journals = load_pending_transactions(project_root, roots, allowed_targets)?;
    for (path, journal) in journals {
        match journal.phase {
            JournalPhase::Intent | JournalPhase::Active => {
                cleanup_completed_journal(&path, &journal)?
            }
            JournalPhase::RollbackCompleted
            | JournalPhase::CleanupRollback
            | JournalPhase::AllCommitted
            | JournalPhase::Cleanup => cleanup_completed_journal(&path, &journal)?,
        }
    }
    Ok(())
}

pub(in crate::asset::migration) fn detect_pending_transactions(
    project_root: &Path,
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<Vec<PathBuf>, AssetMigrationError> {
    Ok(
        load_pending_transactions(project_root, roots, allowed_targets)?
            .into_iter()
            .map(|(path, _)| path)
            .collect(),
    )
}

fn load_pending_transactions(
    project_root: &Path,
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<Vec<(PathBuf, TransactionJournal)>, AssetMigrationError> {
    let Some(directory) = super::journal_owner::existing_journal_directory(project_root)? else {
        return Ok(Vec::new());
    };
    let entries = fs::read_dir(&directory)
        .map_err(|source| recovery_error(directory.clone(), source))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| recovery_error(directory.clone(), source))?;
    let mut journal_paths = Vec::new();
    for entry in entries {
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|source| recovery_error(path.clone(), source))?;
        if is_link_or_reparse(&metadata) || !metadata.is_file() {
            return Err(invalid_journal(
                &path,
                "journal directory entries must be regular non-link files",
            ));
        }
        if path.extension().and_then(|value| value.to_str()) == Some("toml") {
            journal_paths.push(path);
        }
    }
    journal_paths.sort();

    let canonical_roots = roots
        .iter()
        .map(|root| {
            root.canonicalize()
                .map_err(|source| recovery_error(root.to_path_buf(), source))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut journals = Vec::with_capacity(journal_paths.len());
    for path in journal_paths {
        let source =
            fs::read_to_string(&path).map_err(|error| recovery_error(path.clone(), error))?;
        let journal = toml::from_str::<TransactionJournal>(&source).map_err(|source| {
            AssetMigrationError::JournalDeserialize {
                path: path.clone(),
                source,
            }
        })?;
        journals.push((path, journal));
    }
    validate_journals(&journals, &canonical_roots, allowed_targets)?;
    Ok(journals)
}

fn validate_journals(
    journals: &[(PathBuf, TransactionJournal)],
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<(), AssetMigrationError> {
    let mut identities = HashSet::new();
    let allowed_identities = allowed_targets
        .iter()
        .filter_map(|path| path_identity(path))
        .collect::<HashSet<_>>();
    for (journal_path, journal) in journals {
        if journal.version != JOURNAL_VERSION || journal.documents.is_empty() {
            return Err(invalid_journal(
                journal_path,
                "unsupported or empty migration journal",
            ));
        }
        if !valid_transaction_id(&journal.transaction_id) {
            return Err(invalid_journal(
                journal_path,
                "invalid migration transaction id",
            ));
        }
        let journal_name = journal_path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".toml"))
            .unwrap_or_default();
        if !journal_name.ends_with(&format!("-{}", journal.transaction_id)) {
            return Err(invalid_journal(
                journal_path,
                "journal filename does not match transaction id",
            ));
        }
        match journal.phase {
            JournalPhase::Intent
                if journal
                    .documents
                    .iter()
                    .any(|document| document.state != JournalState::Prepared) =>
            {
                return Err(invalid_journal(
                    journal_path,
                    "transaction intent requires every document to be prepared",
                ));
            }
            JournalPhase::AllCommitted | JournalPhase::Cleanup
                if journal
                    .documents
                    .iter()
                    .any(|document| document.state != JournalState::Committed) =>
            {
                return Err(invalid_journal(
                    journal_path,
                    "commit cleanup phase requires every document to be committed",
                ));
            }
            JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback
                if journal
                    .documents
                    .iter()
                    .any(|document| document.state != JournalState::Prepared) =>
            {
                return Err(invalid_journal(
                    journal_path,
                    "rollback cleanup phase requires every document to be prepared",
                ));
            }
            _ => {}
        }
        for document in &journal.documents {
            let target_identity = path_identity(&document.target).ok_or_else(|| {
                invalid_journal(journal_path, "journal target has no canonical identity")
            })?;
            if !allowed_identities.contains(&target_identity) {
                return Err(invalid_journal(
                    journal_path,
                    format!(
                        "journal target {} was not produced by the migration scanner",
                        document.target.display()
                    ),
                ));
            }
            if let Some(retired) = &document.retired_path {
                let retired_identity = path_identity(retired).ok_or_else(|| {
                    invalid_journal(journal_path, "retired target has no canonical identity")
                })?;
                if !allowed_identities.contains(&retired_identity) {
                    return Err(invalid_journal(
                        journal_path,
                        "retired target was not produced by the migration scanner",
                    ));
                }
            }
            validate_document_paths(
                journal_path,
                &journal.transaction_id,
                document,
                roots,
                &mut identities,
            )?;
            if journal.phase == JournalPhase::Active
                && (document.backup.as_ref().is_some_and(|path| !path.is_file())
                    || document
                        .retired_backup
                        .as_ref()
                        .is_some_and(|path| !path.is_file()))
            {
                return Err(invalid_journal(journal_path, "journal backup is missing"));
            }
            validate_document_evidence(journal_path, journal, document)?;
        }
    }
    Ok(())
}

fn validate_document_paths(
    journal_path: &Path,
    transaction_id: &str,
    document: &JournalDocument,
    roots: &[PathBuf],
    identities: &mut HashSet<String>,
) -> Result<(), AssetMigrationError> {
    let paths = [
        Some(&document.target),
        Some(&document.staging),
        document.backup.as_ref(),
        document.retired_path.as_ref(),
        document.retired_backup.as_ref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for path in &paths {
        if !path_is_within_roots(path, roots) {
            return Err(invalid_journal(
                journal_path,
                format!(
                    "journal path {} is outside project asset roots",
                    path.display()
                ),
            ));
        }
        let identity = path_identity(path).ok_or_else(|| {
            invalid_journal(
                journal_path,
                format!("journal path {} has no canonical parent", path.display()),
            )
        })?;
        if !identities.insert(identity) {
            return Err(invalid_journal(
                journal_path,
                format!(
                    "journal path {} aliases another transaction path",
                    path.display()
                ),
            ));
        }
    }

    validate_sibling_role(
        journal_path,
        &document.target,
        &document.staging,
        "stage",
        transaction_id,
    )?;
    if let Some(backup) = &document.backup {
        validate_sibling_role(
            journal_path,
            &document.target,
            backup,
            "backup",
            transaction_id,
        )?;
    }
    match (&document.retired_path, &document.retired_backup) {
        (Some(retired), Some(backup)) => {
            if retired.parent() != document.target.parent() {
                return Err(invalid_journal(
                    journal_path,
                    "retired path must share the target directory",
                ));
            }
            validate_retired_pair(journal_path, &document.target, retired)?;
            validate_sibling_role(
                journal_path,
                retired,
                backup,
                "retired-backup",
                transaction_id,
            )?;
        }
        (None, None) => {}
        _ => {
            return Err(invalid_journal(
                journal_path,
                "retired path and retired backup must be present together",
            ))
        }
    }
    Ok(())
}

fn validate_document_evidence(
    journal_path: &Path,
    journal: &TransactionJournal,
    document: &JournalDocument,
) -> Result<(), AssetMigrationError> {
    if document.target_existed != document.original_digest.is_some() {
        return Err(invalid_journal(
            journal_path,
            "target origin and original digest disagree",
        ));
    }
    let artifacts_may_be_missing = matches!(
        journal.phase,
        JournalPhase::Intent | JournalPhase::Cleanup | JournalPhase::CleanupRollback
    );
    let rollback_complete = matches!(
        journal.phase,
        JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback
    );
    if journal.phase == JournalPhase::Active {
        validate_active_target_observation(journal_path, document)?;
    } else {
        match document.state {
            JournalState::Committed => require_file_digest(
                journal_path,
                &document.target,
                &document.new_digest,
                false,
                "committed target",
            )?,
            JournalState::Prepared if document.target_existed => require_file_digest(
                journal_path,
                &document.target,
                document.original_digest.as_deref().unwrap_or_default(),
                false,
                "original target",
            )?,
            JournalState::Prepared => {
                if document.target.exists() {
                    return Err(invalid_journal(
                        journal_path,
                        "new target exists before transaction commit",
                    ));
                }
            }
            JournalState::Committing => {
                return Err(invalid_journal(
                    journal_path,
                    "committing state is valid only in an active transaction",
                ));
            }
        }
    }
    require_file_digest(
        journal_path,
        &document.staging,
        &document.new_digest,
        artifacts_may_be_missing,
        "staging artifact",
    )?;
    match (&document.backup, &document.original_digest) {
        (Some(backup), Some(digest)) => require_file_digest(
            journal_path,
            backup,
            digest,
            artifacts_may_be_missing,
            "backup artifact",
        )?,
        (None, None) => {}
        _ => {
            return Err(invalid_journal(
                journal_path,
                "backup evidence does not match target origin",
            ))
        }
    }
    match (
        &document.retired_path,
        &document.retired_backup,
        &document.retired_digest,
    ) {
        (Some(retired), Some(backup), Some(digest)) => {
            if journal.phase == JournalPhase::Active {
                if retired.exists() {
                    require_file_digest(journal_path, retired, digest, false, "retired sidecar")?;
                }
            } else if document.state == JournalState::Committed {
                if retired.exists() {
                    return Err(invalid_journal(
                        journal_path,
                        "committed retired sidecar still exists",
                    ));
                }
            } else {
                require_file_digest(journal_path, retired, digest, false, "retired sidecar")?;
            }
            require_file_digest(
                journal_path,
                backup,
                digest,
                artifacts_may_be_missing,
                "retired backup",
            )?;
        }
        (None, None, None) => {}
        _ => {
            return Err(invalid_journal(
                journal_path,
                "retired sidecar evidence is incomplete",
            ))
        }
    }
    if rollback_complete && document.state != JournalState::Prepared {
        return Err(invalid_journal(
            journal_path,
            "rollback completion must retain prepared state",
        ));
    }
    for (artifact, label) in [
        (Some(&document.target), "target"),
        (Some(&document.staging), "staging artifact"),
        (document.backup.as_ref(), "backup artifact"),
        (document.retired_path.as_ref(), "retired sidecar"),
        (document.retired_backup.as_ref(), "retired backup"),
    ]
    .into_iter()
    .filter_map(|(path, label)| path.map(|path| (path, label)))
    {
        validate_toml_evidence(journal_path, artifact, label)?;
    }
    Ok(())
}

fn validate_toml_evidence(
    journal_path: &Path,
    path: &Path,
    label: &str,
) -> Result<(), AssetMigrationError> {
    if !path.exists() {
        return Ok(());
    }
    let source = fs::read_to_string(path).map_err(|error| {
        invalid_journal(
            journal_path,
            format!("{label} {} is not UTF-8 TOML: {error}", path.display()),
        )
    })?;
    toml::from_str::<toml::Table>(&source).map_err(|error| {
        invalid_journal(
            journal_path,
            format!("{label} {} is not a TOML table: {error}", path.display()),
        )
    })?;
    Ok(())
}

fn require_file_digest(
    journal_path: &Path,
    path: &Path,
    expected: &str,
    allow_missing: bool,
    label: &str,
) -> Result<(), AssetMigrationError> {
    if !path.exists() {
        return if allow_missing {
            Ok(())
        } else {
            Err(invalid_journal(journal_path, format!("{label} is missing")))
        };
    }
    let actual = digest_file(path).map_err(|source| recovery_error(path.to_path_buf(), source))?;
    if actual != expected {
        return Err(invalid_journal(
            journal_path,
            format!("{label} digest does not match journal evidence"),
        ));
    }
    Ok(())
}

fn validate_active_target_observation(
    journal_path: &Path,
    document: &JournalDocument,
) -> Result<(), AssetMigrationError> {
    if !document.target.exists() {
        return if document.target_existed {
            Err(invalid_journal(
                journal_path,
                "existing active target is missing",
            ))
        } else {
            Ok(())
        };
    }
    let actual = digest_file(&document.target)
        .map_err(|source| recovery_error(document.target.clone(), source))?;
    let matches_new = actual == document.new_digest;
    let matches_original = document
        .original_digest
        .as_deref()
        .is_some_and(|digest| digest == actual);
    if matches_new || matches_original {
        Ok(())
    } else {
        Err(invalid_journal(
            journal_path,
            "active target matches neither original nor migrated digest",
        ))
    }
}

fn validate_sibling_role(
    journal_path: &Path,
    owner: &Path,
    artifact: &Path,
    role: &str,
    transaction_id: &str,
) -> Result<(), AssetMigrationError> {
    if owner.parent() != artifact.parent() {
        return Err(invalid_journal(
            journal_path,
            format!("{role} artifact must share its owner directory"),
        ));
    }
    let owner_name = owner
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_journal(journal_path, "journal owner name is not UTF-8"))?;
    let artifact_name = artifact
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_journal(journal_path, "journal artifact name is not UTF-8"))?;
    let prefix = format!(".{owner_name}.zr-migrate-{role}-");
    let suffix = artifact_name.strip_prefix(&prefix).ok_or_else(|| {
        invalid_journal(
            journal_path,
            format!("{role} artifact does not use the required sibling name"),
        )
    })?;
    if suffix != transaction_id {
        return Err(invalid_journal(
            journal_path,
            format!("{role} artifact has an invalid transaction suffix"),
        ));
    }
    Ok(())
}

fn validate_retired_pair(
    journal_path: &Path,
    target: &Path,
    retired: &Path,
) -> Result<(), AssetMigrationError> {
    let retired_name = retired
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".meta.toml"))
        .ok_or_else(|| invalid_journal(journal_path, "retired path must end in .meta.toml"))?;
    let expected_target = format!("{retired_name}.zmeta");
    if target.file_name().and_then(|name| name.to_str()) != Some(expected_target.as_str()) {
        return Err(invalid_journal(
            journal_path,
            "retired path and target do not describe the same sidecar",
        ));
    }
    Ok(())
}

fn path_identity(path: &Path) -> Option<String> {
    let identity = if path.exists() {
        path.canonicalize().ok()?
    } else {
        let parent = path.parent()?.canonicalize().ok()?;
        parent.join(path.file_name()?)
    };
    #[cfg(windows)]
    return Some(identity.to_string_lossy().to_ascii_lowercase());
    #[cfg(not(windows))]
    Some(identity.to_string_lossy().into_owned())
}

fn invalid_journal(path: &Path, message: impl Into<String>) -> AssetMigrationError {
    AssetMigrationError::InvalidJournal {
        path: path.to_path_buf(),
        reason: message.into(),
    }
}

fn path_is_within_roots(path: &Path, roots: &[PathBuf]) -> bool {
    if !path.is_absolute() {
        return false;
    }
    let Some(parent) = path.parent() else {
        return false;
    };
    let resolved = if path.exists() {
        path.canonicalize().ok()
    } else {
        parent
            .canonicalize()
            .ok()
            .map(|parent| parent.join(path.file_name().unwrap_or_default()))
    };
    resolved.is_some_and(|path| roots.iter().any(|root| path.starts_with(root)))
}

fn cleanup_completed_journal(
    path: &Path,
    journal: &TransactionJournal,
) -> Result<(), AssetMigrationError> {
    for document in &journal.documents {
        for artifact in [
            Some(&document.staging),
            document.backup.as_ref(),
            document.retired_backup.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            remove_if_exists(artifact)
                .map_err(|source| recovery_error(artifact.to_path_buf(), source))?;
        }
    }
    fs::remove_file(path).map_err(|source| recovery_error(path.to_path_buf(), source))
}

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn recovery_error(path: PathBuf, source: io::Error) -> AssetMigrationError {
    transaction_error(AssetMigrationTransactionPhase::Recovery, path, source)
}
