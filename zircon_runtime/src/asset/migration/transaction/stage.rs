//! Transaction staging owns only reserved sibling artifacts; live targets are untouched here.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super::schema::CommitFault;
use super::{remove_if_exists, transaction_error, transaction_sibling};
use crate::asset::migration::document::PendingDocument;
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};

const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub(super) struct StagedDocument {
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
    let target_existed = document.path.exists();
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
    let new_digest = match write_and_sync_hash(&mut file, &document.bytes) {
        Ok(digest) => digest,
        Err(source) => {
            drop(file);
            let _ = fs::remove_file(&staging);
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document.path,
                source,
            ));
        }
    };
    drop(file);

    let (backup, original_digest) = if target_existed {
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
        let digest = copy_and_sync_hash(&document.path, &backup).map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                document.path.clone(),
                source,
            )
        })?;
        (Some(backup), Some(digest))
    } else {
        (None, None)
    };

    let (retired_backup, retired_digest) = if let Some(retired) = &document.retired_path {
        let backup = transaction_sibling(
            retired.parent().unwrap_or_else(|| Path::new(".")),
            retired,
            "retired-backup",
            transaction_id,
        );
        guard.track(&backup);
        #[cfg(test)]
        if matches!(fault, CommitFault::FailRetiredBackupSync(index) if index == document_index) {
            return Err(transaction_error(
                AssetMigrationTransactionPhase::Stage,
                retired.clone(),
                io::Error::new(io::ErrorKind::Other, "injected retired backup sync failure"),
            ));
        }
        let digest = copy_and_sync_hash(retired, &backup).map_err(|source| {
            transaction_error(
                AssetMigrationTransactionPhase::Stage,
                retired.clone(),
                source,
            )
        })?;
        (Some(backup), Some(digest))
    } else {
        (None, None)
    };

    let staged = StagedDocument {
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

pub(super) fn cleanup_staging(staged: &[StagedDocument]) {
    for document in staged {
        for artifact in [
            Some(&document.staging),
            document.backup.as_ref(),
            document.retired_backup.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            let _ = remove_if_exists(artifact);
        }
    }
}

fn write_and_sync_hash(file: &mut File, bytes: &[u8]) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    for chunk in bytes.chunks(COPY_BUFFER_BYTES) {
        file.write_all(chunk)?;
        hasher.update(chunk);
    }
    file.flush()?;
    file.sync_all()?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn copy_and_sync_hash(source: &Path, target: &Path) -> io::Result<String> {
    let mut source = File::open(source)?;
    let mut target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    let mut hasher = blake3::Hasher::new();
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        target.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
    }
    target.flush()?;
    target.sync_all()?;
    Ok(hasher.finalize().to_hex().to_string())
}
