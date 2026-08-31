//! Staging creates only journal-owned siblings and leaves live targets unchanged.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use super::PreparedFileWrite;
use super::error::{DurableTransactionError, TransactionPhase};
use super::schema::{JournalIntent, TransactionFault};
use crate::io::{ensure_parent_directories, sync_parent_directory};

const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub(super) struct StagedFile {
    pub(super) intent: JournalIntent,
    pub(super) target_existed: bool,
    pub(super) original_digest: Option<String>,
    pub(super) new_digest: String,
    pub(super) retired_digests: Vec<String>,
    pub(super) committed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FilePresence {
    Missing,
    Present,
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
    #[cfg(any(test, feature = "test-support"))]
    if matches!(fault, TransactionFault::FailStageWrite(fault_index) if fault_index == index) {
        return Err(operation(
            &write.path,
            io::Error::other("injected staging write failure"),
        ));
    }
    let new_digest = write_and_sync_hash(&mut staging, &write.bytes)
        .map_err(|source| operation(&write.path, source))?;
    drop(staging);
    #[cfg(any(test, feature = "test-support"))]
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

    let target_existed = file_presence(&write.path)
        .map_err(|source| operation(&write.path, source))?
        == FilePresence::Present;
    let original_digest = if target_existed {
        #[cfg(any(test, feature = "test-support"))]
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

    assert_eq!(write.retirements.len(), intent.retirements.len());
    let mut retired_digests = Vec::with_capacity(write.retirements.len());
    for (retirement, retired_intent) in write.retirements.iter().zip(&intent.retirements) {
        if file_presence(&retirement.path).map_err(|source| operation(&retirement.path, source))?
            == FilePresence::Missing
        {
            return Err(operation(
                &retirement.path,
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "retired transaction file is missing",
                ),
            ));
        }
        #[cfg(any(test, feature = "test-support"))]
        if matches!(fault, TransactionFault::FailRetiredBackupSync(fault_index) if fault_index == index)
        {
            return Err(operation(
                &retirement.path,
                io::Error::other("injected retired backup sync failure"),
            ));
        }
        let digest = copy_and_sync_hash(&retirement.path, &retired_intent.backup)
            .map_err(|source| operation(&retirement.path, source))?;
        if retirement
            .expected_digest
            .as_deref()
            .is_some_and(|expected| expected != digest)
        {
            return Err(operation(
                &retirement.path,
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "retired transaction file changed since preparation",
                ),
            ));
        }
        retired_digests.push(digest);
    }

    let staged = StagedFile {
        intent,
        target_existed,
        original_digest,
        new_digest,
        retired_digests,
        committed: false,
    };
    verify_originals(&staged).map_err(|source| operation(&staged.intent.target, source))?;
    Ok(staged)
}

pub(super) fn verify_originals(staged: &StagedFile) -> io::Result<()> {
    match staged.original_digest.as_deref() {
        Some(expected) => require_digest(&staged.intent.target, expected),
        None => match file_presence(&staged.intent.target)? {
            FilePresence::Missing => Ok(()),
            FilePresence::Present => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "transaction target appeared after intent preparation",
            )),
        },
    }?;
    for (retirement, expected) in staged
        .intent
        .retirements
        .iter()
        .zip(&staged.retired_digests)
    {
        require_digest(&retirement.path, expected)?;
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

pub(super) fn cleanup_intents_journal_first(
    journal: &Path,
    intents: &[JournalIntent],
) -> io::Result<()> {
    remove_reserved_if_exists(journal)?;
    cleanup_intents(intents)
}

pub(super) fn intent_artifacts(intent: &JournalIntent) -> impl Iterator<Item = &Path> {
    let document_artifacts = [
        Some(intent.staging.as_path()),
        Some(intent.backup.as_path()),
        Some(intent.rollback_staging.as_path()),
    ]
    .into_iter()
    .flatten();
    document_artifacts.chain(intent.retirements.iter().flat_map(|retirement| {
        [
            retirement.backup.as_path(),
            retirement.rollback_staging.as_path(),
        ]
    }))
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
    file_presence(path).map(|_| ())
}

pub(super) fn file_presence(path: &Path) -> io::Result<FilePresence> {
    classify_file_metadata(fs::symlink_metadata(path))
}

fn classify_file_metadata(metadata: io::Result<fs::Metadata>) -> io::Result<FilePresence> {
    match metadata {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "transaction path is not a regular file",
            ))
        }
        Ok(_) => Ok(FilePresence::Present),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(FilePresence::Missing),
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

#[cfg(test)]
mod tests {
    use std::io;

    use super::{FilePresence, classify_file_metadata};

    #[test]
    fn file_presence_treats_only_not_found_as_missing() {
        let missing = classify_file_metadata(Err(io::Error::from(io::ErrorKind::NotFound)))
            .expect("NotFound is the only missing-file classification");
        assert_eq!(missing, FilePresence::Missing);

        let error = classify_file_metadata(Err(io::Error::from(io::ErrorKind::PermissionDenied)))
            .expect_err("metadata access failures must not become missing-file evidence");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }
}
