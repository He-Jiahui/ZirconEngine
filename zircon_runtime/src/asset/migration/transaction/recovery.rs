use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::schema::{
    FoldedTransactionJournal, JOURNAL_VERSION, JournalDocument, JournalPhase, JournalState,
    TransactionJournal,
};
use super::toml_evidence::TomlEvidenceReader;
use super::{transaction_error, valid_transaction_id};
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};
use crate::asset::safe_project_path::is_link_or_reparse;

const EVIDENCE_READ_BUFFER_BYTES: usize = 64 * 1024;

pub(in crate::asset::migration) fn recover_pending_transactions(
    project_root: &Path,
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<(), AssetMigrationError> {
    let journals = load_pending_transactions(project_root, roots, allowed_targets)?;
    for (path, journal) in journals {
        match journal.phase {
            JournalPhase::Intent
            | JournalPhase::Active
            | JournalPhase::RollbackCompleted
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
) -> Result<Vec<(PathBuf, FoldedTransactionJournal)>, AssetMigrationError> {
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
        let value = toml::from_str::<toml::Value>(&source).map_err(|source| {
            AssetMigrationError::JournalDeserialize {
                path: path.clone(),
                source,
            }
        })?;
        if value.get("version").and_then(toml::Value::as_integer)
            != Some(i64::from(JOURNAL_VERSION))
        {
            return Err(invalid_journal(
                &path,
                "unsupported migration journal version",
            ));
        }
        let parsed = toml::from_str::<TransactionJournal>(&source).map_err(|source| {
            AssetMigrationError::JournalDeserialize {
                path: path.clone(),
                source,
            }
        })?;
        if parsed.documents.is_empty() {
            return Err(invalid_journal(&path, "empty migration journal"));
        }
        let journal = parsed
            .fold()
            .map_err(|reason| invalid_journal(&path, reason))?;
        journals.push((path, journal));
    }
    validate_journals(&journals, &canonical_roots, allowed_targets)?;
    Ok(journals)
}

fn validate_journals(
    journals: &[(PathBuf, FoldedTransactionJournal)],
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<(), AssetMigrationError> {
    let mut identities = HashSet::new();
    let allowed_identities = allowed_targets
        .iter()
        .filter_map(|path| path_identity(path))
        .collect::<HashSet<_>>();
    let mut evidence = RecoveryEvidence::new();
    for (journal_path, journal) in journals {
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
        validate_phase(journal_path, journal)?;
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
            validate_document_evidence(journal_path, journal, document, &mut evidence)?;
        }
    }
    Ok(())
}

fn validate_phase(
    journal_path: &Path,
    journal: &FoldedTransactionJournal,
) -> Result<(), AssetMigrationError> {
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
        Err(invalid_journal(
            journal_path,
            "journal phase and folded document states disagree",
        ))
    }
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
            ));
        }
    }
    Ok(())
}

fn validate_document_evidence(
    journal_path: &Path,
    journal: &FoldedTransactionJournal,
    document: &JournalDocument,
    evidence: &mut RecoveryEvidence,
) -> Result<(), AssetMigrationError> {
    if document.state == JournalState::Intent {
        return validate_existing_toml_evidence(journal_path, document, evidence);
    }

    let target_existed = document.target_existed.ok_or_else(|| {
        invalid_journal(
            journal_path,
            "document state is missing target origin evidence",
        )
    })?;
    let new_digest = document.new_digest.as_deref().ok_or_else(|| {
        invalid_journal(
            journal_path,
            "document state is missing staged digest evidence",
        )
    })?;
    if target_existed != document.original_digest.is_some()
        || target_existed != document.backup.is_some()
    {
        return Err(invalid_journal(
            journal_path,
            "target origin, backup role, and original digest disagree",
        ));
    }
    if document.retired_path.is_some() != document.retired_digest.is_some() {
        return Err(invalid_journal(
            journal_path,
            "retired sidecar and retired digest disagree",
        ));
    }
    let artifacts_may_be_missing = matches!(
        journal.phase,
        JournalPhase::RollbackCompleted | JournalPhase::Cleanup | JournalPhase::CleanupRollback
    );
    let staging_may_be_missing = artifacts_may_be_missing
        || document.state == JournalState::Committed
        || document.state == JournalState::RollingBack
        || (document.state == JournalState::Committing
            && target_matches_digest(journal_path, &document.target, new_digest, evidence)?);

    match document.state {
        JournalState::Prepared => validate_original_target(
            journal_path,
            document,
            target_existed,
            artifacts_may_be_missing,
            evidence,
        )?,
        JournalState::Committing => validate_active_target_observation(
            journal_path,
            document,
            target_existed,
            new_digest,
            evidence,
        )?,
        JournalState::RollingBack => validate_active_target_observation(
            journal_path,
            document,
            target_existed,
            new_digest,
            evidence,
        )?,
        JournalState::Committed => require_file_digest(
            journal_path,
            &document.target,
            new_digest,
            false,
            "committed target",
            evidence,
        )?,
        JournalState::Intent => unreachable!("handled before evidence validation"),
    }

    require_file_digest(
        journal_path,
        &document.staging,
        new_digest,
        staging_may_be_missing,
        "staging artifact",
        evidence,
    )?;
    if let (Some(backup), Some(digest)) = (&document.backup, &document.original_digest) {
        require_file_digest(
            journal_path,
            backup,
            digest,
            artifacts_may_be_missing,
            "backup artifact",
            evidence,
        )?;
    }
    match (
        &document.retired_path,
        &document.retired_backup,
        &document.retired_digest,
    ) {
        (Some(retired), Some(backup), Some(digest)) => {
            match document.state {
                JournalState::Prepared => require_file_digest(
                    journal_path,
                    retired,
                    digest,
                    false,
                    "retired sidecar",
                    evidence,
                )?,
                JournalState::Committing | JournalState::RollingBack if retired.exists() => {
                    require_file_digest(
                        journal_path,
                        retired,
                        digest,
                        false,
                        "retired sidecar",
                        evidence,
                    )?
                }
                JournalState::Committing => require_file_digest(
                    journal_path,
                    &document.target,
                    new_digest,
                    false,
                    "committing target after retired-sidecar deletion",
                    evidence,
                )?,
                JournalState::RollingBack => {}
                JournalState::Committed if retired.exists() => {
                    return Err(invalid_journal(
                        journal_path,
                        "committed retired sidecar still exists",
                    ));
                }
                JournalState::Committed => {}
                JournalState::Intent => unreachable!("handled before evidence validation"),
            }
            require_file_digest(
                journal_path,
                backup,
                digest,
                artifacts_may_be_missing,
                "retired backup",
                evidence,
            )?;
        }
        (None, None, None) => {}
        _ => {
            return Err(invalid_journal(
                journal_path,
                "retired sidecar evidence is incomplete",
            ));
        }
    }
    validate_existing_toml_evidence(journal_path, document, evidence)
}

fn validate_original_target(
    journal_path: &Path,
    document: &JournalDocument,
    target_existed: bool,
    artifacts_may_be_missing: bool,
    evidence: &mut RecoveryEvidence,
) -> Result<(), AssetMigrationError> {
    if target_existed {
        require_file_digest(
            journal_path,
            &document.target,
            document.original_digest.as_deref().unwrap_or_default(),
            false,
            "original target",
            evidence,
        )
    } else if document.target.exists() && !artifacts_may_be_missing {
        Err(invalid_journal(
            journal_path,
            "new target exists before transaction commit",
        ))
    } else {
        Ok(())
    }
}

fn validate_existing_toml_evidence(
    journal_path: &Path,
    document: &JournalDocument,
    evidence: &mut RecoveryEvidence,
) -> Result<(), AssetMigrationError> {
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
        if artifact.exists() {
            let _ = file_evidence(journal_path, artifact, label, evidence)?;
        }
    }
    Ok(())
}

fn require_file_digest(
    journal_path: &Path,
    path: &Path,
    expected: &str,
    allow_missing: bool,
    label: &str,
    evidence: &mut RecoveryEvidence,
) -> Result<(), AssetMigrationError> {
    if !path.exists() {
        return if allow_missing {
            Ok(())
        } else {
            Err(invalid_journal(journal_path, format!("{label} is missing")))
        };
    }
    if file_evidence(journal_path, path, label, evidence)?.digest != expected {
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
    target_existed: bool,
    new_digest: &str,
    evidence: &mut RecoveryEvidence,
) -> Result<(), AssetMigrationError> {
    if !document.target.exists() {
        return if target_existed {
            Err(invalid_journal(
                journal_path,
                "existing active target is missing",
            ))
        } else {
            Ok(())
        };
    }
    let actual = &file_evidence(journal_path, &document.target, "active target", evidence)?.digest;
    let matches_new = actual == new_digest;
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

fn target_matches_digest(
    journal_path: &Path,
    path: &Path,
    expected: &str,
    evidence: &mut RecoveryEvidence,
) -> Result<bool, AssetMigrationError> {
    if !path.exists() {
        return Ok(false);
    }
    Ok(file_evidence(journal_path, path, "transaction target", evidence)?.digest == expected)
}

fn file_evidence<'a>(
    journal_path: &Path,
    path: &Path,
    label: &str,
    evidence: &'a mut RecoveryEvidence,
) -> Result<&'a FileEvidence, AssetMigrationError> {
    if !evidence.artifacts.contains_key(path) {
        let value = stream_file_evidence(path, &mut evidence.reader).map_err(|source| {
            invalid_journal(
                journal_path,
                format!(
                    "{label} {} does not satisfy bounded TOML structure evidence: {source}",
                    path.display()
                ),
            )
        })?;
        evidence.artifacts.insert(path.to_path_buf(), value);
    }
    evidence.artifacts.get(path).ok_or_else(|| {
        invalid_journal(
            journal_path,
            format!(
                "{label} {} was not retained after evidence insertion",
                path.display()
            ),
        )
    })
}

fn stream_file_evidence(path: &Path, reader: &mut TomlEvidenceReader) -> io::Result<FileEvidence> {
    Ok(FileEvidence {
        digest: reader.stream_file_digest(path)?,
    })
}

struct RecoveryEvidence {
    artifacts: HashMap<PathBuf, FileEvidence>,
    reader: TomlEvidenceReader,
}

impl RecoveryEvidence {
    fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            reader: TomlEvidenceReader::new(EVIDENCE_READ_BUFFER_BYTES),
        }
    }
}

struct FileEvidence {
    digest: String,
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
    journal: &FoldedTransactionJournal,
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

fn invalid_journal(path: &Path, message: impl Into<String>) -> AssetMigrationError {
    AssetMigrationError::InvalidJournal {
        path: path.to_path_buf(),
        reason: message.into(),
    }
}

fn recovery_error(path: PathBuf, source: io::Error) -> AssetMigrationError {
    transaction_error(AssetMigrationTransactionPhase::Recovery, path, source)
}
