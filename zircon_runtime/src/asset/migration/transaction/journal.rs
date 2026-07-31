//! Immutable migration intent plus fsync'd append-only transition records.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::journal_owner;
use super::schema::{
    JOURNAL_VERSION, JournalIntent, JournalPhase, JournalState, JournalTransition,
    TransactionJournal,
};
use super::stage::StagedDocument;
use super::{transaction_error, transaction_sibling};
use crate::asset::migration::document::PendingDocument;
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};
use crate::foundation::persistence::atomic_file::atomic_write;

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
            JournalIntent {
                target: document.path.clone(),
                staging: transaction_sibling(
                    document_parent,
                    &document.path,
                    "stage",
                    transaction_id,
                ),
                backup: document.path.exists().then(|| {
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
            }
        })
        .collect();
    persist_intent(
        &path,
        &TransactionJournal {
            version: JOURNAL_VERSION,
            transaction_id: transaction_id.to_owned(),
            documents,
            transitions: Vec::new(),
        },
    )?;
    Ok(path)
}

pub(super) fn record_document_prepared(
    path: &Path,
    document_index: usize,
    document: &StagedDocument,
) -> Result<(), AssetMigrationError> {
    append_transition(
        path,
        JournalTransition {
            phase: JournalPhase::Intent,
            document_index: Some(document_index),
            state: Some(JournalState::Prepared),
            target_existed: Some(document.target_existed),
            original_digest: document.original_digest.clone(),
            new_digest: Some(document.new_digest.clone()),
            retired_digest: document.retired_digest.clone(),
        },
        AssetMigrationTransactionPhase::Stage,
    )
}

pub(super) fn activate_journal(path: &Path) -> Result<(), AssetMigrationError> {
    append_transition(
        path,
        JournalTransition {
            phase: JournalPhase::Active,
            document_index: None,
            state: None,
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digest: None,
        },
        AssetMigrationTransactionPhase::Stage,
    )
}

pub(super) fn record_document_state(
    path: &Path,
    document_index: usize,
    state: JournalState,
) -> Result<(), AssetMigrationError> {
    append_transition(
        path,
        JournalTransition {
            phase: JournalPhase::Active,
            document_index: Some(document_index),
            state: Some(state),
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digest: None,
        },
        AssetMigrationTransactionPhase::Commit,
    )
}

pub(super) fn record_phase(path: &Path, phase: JournalPhase) -> Result<(), AssetMigrationError> {
    let transaction_phase = match phase {
        JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback => {
            AssetMigrationTransactionPhase::Rollback
        }
        JournalPhase::Intent
        | JournalPhase::Active
        | JournalPhase::AllCommitted
        | JournalPhase::Cleanup => AssetMigrationTransactionPhase::Commit,
    };
    append_transition(
        path,
        JournalTransition {
            phase,
            document_index: None,
            state: None,
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digest: None,
        },
        transaction_phase,
    )
}

fn persist_intent(path: &Path, journal: &TransactionJournal) -> Result<(), AssetMigrationError> {
    let bytes = toml::to_string_pretty(journal).map_err(|error| {
        transaction_error(
            AssetMigrationTransactionPhase::Stage,
            path.to_path_buf(),
            io::Error::new(io::ErrorKind::InvalidData, error.to_string()),
        )
    })?;
    atomic_write(path, bytes.as_bytes())
        .and_then(|()| sync_committed_journal(path))
        .map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                path.to_path_buf(),
                source,
            )
        })
}

fn sync_committed_journal(path: &Path) -> io::Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    sync_parent_directory(path)
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn append_transition(
    path: &Path,
    transition: JournalTransition,
    phase: AssetMigrationTransactionPhase,
) -> Result<(), AssetMigrationError> {
    let bytes = toml::to_string_pretty(&TransitionAppend {
        transitions: vec![transition],
    })
    .map_err(|error| {
        transaction_error(
            phase,
            path.to_path_buf(),
            io::Error::new(io::ErrorKind::InvalidData, error.to_string()),
        )
    })?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|source| transaction_error(phase, path.to_path_buf(), source))?;
    file.write_all(bytes.as_bytes())
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|source| transaction_error(phase, path.to_path_buf(), source))
}

#[derive(Serialize)]
struct TransitionAppend {
    transitions: Vec<JournalTransition>,
}
