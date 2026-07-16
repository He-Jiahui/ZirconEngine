//! Durable intent creation and synchronized journal state transitions.

use std::io;
use std::path::{Path, PathBuf};

use super::journal_owner;
use super::schema::{
    JournalDocument, JournalPhase, JournalState, TransactionJournal, JOURNAL_VERSION,
};
use super::stage::StagedDocument;
use super::{digest_bytes, digest_file, transaction_error, transaction_sibling};
use crate::asset::migration::document::PendingDocument;
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};
use crate::asset::project::meta_io::atomic_write;

pub(super) fn create_intent_journal(
    project_root: &Path,
    pending: &[PendingDocument],
    transaction_id: &str,
) -> Result<PathBuf, AssetMigrationError> {
    let first = pending.first().ok_or_else(|| {
        transaction_error(
            AssetMigrationTransactionPhase::Stage,
            PathBuf::from("<empty>"),
            io::Error::new(io::ErrorKind::InvalidInput, "empty migration transaction"),
        )
    })?;
    let parent = journal_owner::ensure_journal_directory(project_root)?;
    let mut path = transaction_sibling(&parent, &first.path, "journal", transaction_id);
    path.as_mut_os_string().push(".toml");
    let documents = pending
        .iter()
        .map(|document| {
            let document_parent = document.path.parent().unwrap_or_else(|| Path::new("."));
            let target_existed = document.path.exists();
            let original_digest = target_existed
                .then(|| digest_file(&document.path))
                .transpose()
                .map_err(|source| {
                    transaction_error(
                        AssetMigrationTransactionPhase::Stage,
                        document.path.clone(),
                        source,
                    )
                })?;
            let retired_digest = document
                .retired_path
                .as_ref()
                .map(|retired| digest_file(retired))
                .transpose()
                .map_err(|source| {
                    transaction_error(
                        AssetMigrationTransactionPhase::Stage,
                        document.path.clone(),
                        source,
                    )
                })?;
            Ok(JournalDocument {
                state: JournalState::Prepared,
                target_existed,
                original_digest,
                new_digest: digest_bytes(&document.bytes),
                retired_digest,
                target: document.path.clone(),
                staging: transaction_sibling(
                    document_parent,
                    &document.path,
                    "stage",
                    transaction_id,
                ),
                backup: target_existed.then(|| {
                    transaction_sibling(document_parent, &document.path, "backup", transaction_id)
                }),
                retired_path: document.retired_path.clone(),
                retired_backup: document.retired_path.as_ref().map(|retired| {
                    transaction_sibling(
                        retired.parent().unwrap_or_else(|| Path::new(".")),
                        retired,
                        "retired-backup",
                        transaction_id,
                    )
                }),
            })
        })
        .collect::<Result<Vec<_>, AssetMigrationError>>()?;
    persist_journal(
        &path,
        &TransactionJournal {
            version: JOURNAL_VERSION,
            transaction_id: transaction_id.to_owned(),
            phase: JournalPhase::Intent,
            documents,
        },
    )?;
    Ok(path)
}

pub(super) fn sync_journal(
    path: &Path,
    staged: &[StagedDocument],
    phase: JournalPhase,
) -> Result<(), AssetMigrationError> {
    let journal = TransactionJournal {
        version: JOURNAL_VERSION,
        transaction_id: staged
            .first()
            .map(|document| document.transaction_id.clone())
            .unwrap_or_default(),
        phase,
        documents: staged
            .iter()
            .map(|document| JournalDocument {
                state: if document.committed {
                    JournalState::Committed
                } else if document.committing {
                    JournalState::Committing
                } else {
                    JournalState::Prepared
                },
                target_existed: document.target_existed,
                original_digest: document.original_digest.clone(),
                new_digest: document.new_digest.clone(),
                retired_digest: document.retired_digest.clone(),
                target: document.target.clone(),
                staging: document.staging.clone(),
                backup: document.backup.clone(),
                retired_path: document.retired_path.clone(),
                retired_backup: document.retired_backup.clone(),
            })
            .collect(),
    };
    persist_journal(path, &journal)
}

pub(super) fn persist_journal(
    path: &Path,
    journal: &TransactionJournal,
) -> Result<(), AssetMigrationError> {
    let bytes = toml::to_string_pretty(journal).map_err(|error| {
        transaction_error(
            AssetMigrationTransactionPhase::Stage,
            path.to_path_buf(),
            io::Error::new(io::ErrorKind::InvalidData, error.to_string()),
        )
    })?;
    atomic_write(path, bytes.as_bytes()).map_err(|source| {
        transaction_error(
            AssetMigrationTransactionPhase::Stage,
            path.to_path_buf(),
            source,
        )
    })
}
