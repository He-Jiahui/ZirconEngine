//! Transaction staging owns only reserved sibling artifacts; live targets are untouched here.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::schema::CommitFault;
use super::{digest_bytes, digest_file, remove_if_exists, transaction_error, transaction_sibling};
use crate::asset::migration::document::PendingDocument;
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};

pub(super) struct StagedDocument {
    pub(super) transaction_id: String,
    pub(super) target: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: Option<PathBuf>,
    pub(super) retired_path: Option<PathBuf>,
    pub(super) retired_backup: Option<PathBuf>,
    pub(super) target_existed: bool,
    pub(super) original_digest: Option<String>,
    pub(super) new_digest: String,
    pub(super) retired_digest: Option<String>,
    pub(super) committing: bool,
    pub(super) committed: bool,
}

#[derive(Default)]
struct StageArtifactGuard {
    artifacts: Vec<PathBuf>,
    armed: bool,
}

impl StageArtifactGuard {
    fn new() -> Self {
        Self {
            artifacts: Vec::new(),
            armed: true,
        }
    }
    fn track(&mut self, path: &Path) {
        self.artifacts.push(path.to_path_buf());
    }
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for StageArtifactGuard {
    fn drop(&mut self) {
        if self.armed {
            for artifact in self.artifacts.iter().rev() {
                let _ = remove_if_exists(artifact);
            }
        }
    }
}

pub(super) fn stage_document(
    document: PendingDocument,
    transaction_id: &str,
    fault: CommitFault,
    document_index: usize,
) -> Result<StagedDocument, AssetMigrationError> {
    #[cfg(not(test))]
    let _ = (fault, document_index);
    let mut guard = StageArtifactGuard::new();
    let parent = document.path.parent().unwrap_or_else(|| Path::new("."));
    let staging = transaction_sibling(parent, &document.path, "stage", transaction_id);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document.path.clone(),
                source,
            )
        })?;
    guard.track(&staging);
    #[cfg(test)]
    if matches!(fault, CommitFault::FailStageWrite(index) if index == document_index) {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Stage,
            document.path,
            io::Error::new(io::ErrorKind::Other, "injected staging write failure"),
        ));
    }
    if let Err(source) = write_and_sync(&mut file, &document.bytes) {
        drop(file);
        let _ = fs::remove_file(&staging);
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Stage,
            document.path,
            source,
        ));
    }
    drop(file);
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
    let new_digest = digest_bytes(&document.bytes);
    let backup = if target_existed {
        let backup = transaction_sibling(parent, &document.path, "backup", transaction_id);
        guard.track(&backup);
        #[cfg(test)]
        if matches!(fault, CommitFault::FailBackupCopy(index) if index == document_index) {
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document.path,
                io::Error::new(io::ErrorKind::Other, "injected backup copy failure"),
            ));
        }
        fs::copy(&document.path, &backup).map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document.path.clone(),
                source,
            )
        })?;
        sync_existing_file(&backup).map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document.path.clone(),
                source,
            )
        })?;
        Some(backup)
    } else {
        None
    };
    let retired_backup = if let Some(retired) = &document.retired_path {
        let backup = transaction_sibling(
            retired.parent().unwrap_or_else(|| Path::new(".")),
            retired,
            "retired-backup",
            transaction_id,
        );
        guard.track(&backup);
        fs::copy(retired, &backup).map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                retired.clone(),
                source,
            )
        })?;
        #[cfg(test)]
        if matches!(fault, CommitFault::FailRetiredBackupSync(index) if index == document_index) {
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Stage,
                retired.clone(),
                io::Error::new(io::ErrorKind::Other, "injected retired backup sync failure"),
            ));
        }
        sync_existing_file(&backup).map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                retired.clone(),
                source,
            )
        })?;
        Some(backup)
    } else {
        None
    };
    let retired_digest = document
        .retired_path
        .as_ref()
        .map(|path| digest_file(path))
        .transpose()
        .map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document
                    .retired_path
                    .clone()
                    .unwrap_or_else(|| document.path.clone()),
                source,
            )
        })?;
    let staged = StagedDocument {
        transaction_id: transaction_id.to_string(),
        target: document.path,
        staging,
        backup,
        retired_path: document.retired_path,
        retired_backup,
        target_existed,
        original_digest,
        new_digest,
        retired_digest,
        committing: false,
        committed: false,
    };
    guard.disarm();
    Ok(staged)
}

fn write_and_sync(file: &mut File, bytes: &[u8]) -> io::Result<()> {
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()
}

fn sync_existing_file(path: &Path) -> io::Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()
}

pub(super) fn cleanup_staging(staged: &[StagedDocument]) {
    for document in staged {
        if document.staging.exists() {
            let _ = fs::remove_file(&document.staging);
        }
        if !document.committed {
            if let Some(backup) = &document.backup {
                let _ = fs::remove_file(backup);
            }
            if let Some(backup) = &document.retired_backup {
                let _ = fs::remove_file(backup);
            }
        }
    }
}
