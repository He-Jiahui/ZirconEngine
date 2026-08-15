//! Staging creates only journal-owned siblings and leaves live targets unchanged.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use super::error::{DurableTransactionError, TransactionPhase};
use super::schema::{JournalIntent, TransactionFault};
use super::PreparedFileWrite;
use crate::core::resource::io::{ensure_parent_directories, sync_parent_directory};

const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub(super) struct StagedFile {
    pub(super) intent: JournalIntent,
    pub(super) target_existed: bool,
    pub(super) original_digest: Option<String>,
    pub(super) new_digest: String,
    pub(super) retired_digest: Option<String>,
    pub(super) committed: bool,
}

pub(super) fn stage_file(
    write: PreparedFileWrite,
    intent: JournalIntent,
    fault: TransactionFault,
    index: usize,
) -> Result<StagedFile, DurableTransactionError> {
    #[cfg(not(test))]
    let _ = (fault, index);
    ensure_parent_directories(&write.path).map_err(|source| operation(&write.path, source))?;
    ensure_regular_or_missing(&write.path).map_err(|source| operation(&write.path, source))?;
    remove_reserved_if_exists(&intent.staging)
        .map_err(|source| operation(&intent.staging, source))?;
    let mut staging = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&intent.staging)
        .map_err(|source| operation(&write.path, source))?;
    #[cfg(test)]
    if matches!(fault, TransactionFault::FailStageWrite(fault_index) if fault_index == index) {
        return Err(operation(
            &write.path,
            io::Error::other("injected staging write failure"),
        ));
    }
    let new_digest = write_and_sync_hash(&mut staging, &write.bytes)
        .map_err(|source| operation(&write.path, source))?;
    drop(staging);
    #[cfg(test)]
    if matches!(
        fault,
        TransactionFault::FailStagingDirectorySync(fault_index) if fault_index == index
    ) {
        return Err(operation(
            &intent.staging,
            io::Error::other("injected staging directory sync failure"),
        ));
    }
    sync_parent_directory(&intent.staging).map_err(|source| operation(&intent.staging, source))?;

    let target_existed = write.path.exists();
    let original_digest = if target_existed {
        #[cfg(test)]
        if matches!(fault, TransactionFault::FailBackupCopy(fault_index) if fault_index == index) {
            return Err(operation(
                &write.path,
                io::Error::other("injected backup copy failure"),
            ));
        }
        Some(
            copy_and_sync_hash(&write.path, &intent.backup)
                .map_err(|source| operation(&write.path, source))?,
        )
    } else {
        None
    };

    let retired_digest = match (&write.retired_path, &intent.retired_backup) {
        (Some(retired), Some(backup)) => {
            ensure_regular_or_missing(retired).map_err(|source| operation(retired, source))?;
            if !retired.exists() {
                return Err(operation(
                    retired,
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "retired transaction file is missing",
                    ),
                ));
            }
            #[cfg(test)]
            if matches!(fault, TransactionFault::FailRetiredBackupSync(fault_index) if fault_index == index)
            {
                return Err(operation(
                    retired,
                    io::Error::other("injected retired backup sync failure"),
                ));
            }
            Some(copy_and_sync_hash(retired, backup).map_err(|source| operation(retired, source))?)
        }
        (None, None) => None,
        _ => unreachable!("journal intent follows the prepared write"),
    };

    let staged = StagedFile {
        intent,
        target_existed,
        original_digest,
        new_digest,
        retired_digest,
        committed: false,
    };
    verify_originals(&staged).map_err(|source| operation(&staged.intent.target, source))?;
    Ok(staged)
}

pub(super) fn verify_originals(staged: &StagedFile) -> io::Result<()> {
    match staged.original_digest.as_deref() {
        Some(expected) => require_digest(&staged.intent.target, expected),
        None if staged.intent.target.exists() => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "transaction target appeared after intent preparation",
        )),
        None => Ok(()),
    }?;
    if let (Some(path), Some(expected)) = (
        staged.intent.retired_path.as_deref(),
        staged.retired_digest.as_deref(),
    ) {
        require_digest(path, expected)?;
    }
    Ok(())
}

pub(super) fn cleanup_intents(intents: &[JournalIntent]) -> io::Result<()> {
    let mut first_error = None;
    for intent in intents {
        for artifact in intent_artifacts(intent) {
            if let Err(error) = remove_reserved_if_exists(artifact) {
                first_error.get_or_insert(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

pub(super) fn intent_artifacts(intent: &JournalIntent) -> impl Iterator<Item = &Path> {
    [
        Some(intent.staging.as_path()),
        Some(intent.backup.as_path()),
        Some(intent.rollback_staging.as_path()),
        intent.retired_backup.as_deref(),
        intent.retired_rollback_staging.as_deref(),
    ]
    .into_iter()
    .flatten()
}

pub(super) fn remove_reserved_if_exists(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "transaction artifact is not a regular file",
            ))
        }
        Ok(_) => fs::remove_file(path).and_then(|()| sync_parent_directory(path)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn ensure_regular_or_missing(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "transaction path is not a regular file",
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn digest_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    let mut hasher = blake3::Hasher::new();
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub(super) fn copy_and_sync_hash(source: &Path, target: &Path) -> io::Result<String> {
    remove_reserved_if_exists(target)?;
    let mut source = File::open(source)?;
    let mut target_file = OpenOptions::new()
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
        target_file.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
    }
    target_file.flush()?;
    target_file.sync_all()?;
    sync_parent_directory(target)?;
    Ok(hasher.finalize().to_hex().to_string())
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

fn require_digest(path: &Path, expected: &str) -> io::Result<()> {
    let actual = digest_file(path)?;
    if actual == expected {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "transaction source changed during staging",
        ))
    }
}

fn operation(path: &Path, source: io::Error) -> DurableTransactionError {
    DurableTransactionError::operation(TransactionPhase::Stage, path, source)
}
