use std::fs;
use std::io::{self, BufReader, Read};
use std::path::Path;

use super::super::super::archive::RuntimeSessionArchiveWirePayload;
use super::super::super::{
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

const ARCHIVE_READ_BUFFER_BYTES: usize = 64 * 1024;

pub(in crate::scene::dynamic_scene::session) fn load_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    load_from_path_with_limit(path, MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES)
}

pub(in crate::scene::dynamic_scene::session::io) fn load_from_path_with_limit(
    path: impl AsRef<Path>,
    max_archive_bytes: usize,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    load_file(fs::File::open(path)?, max_archive_bytes)
}

pub(in crate::scene::dynamic_scene::session) fn load_or_empty_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    match fs::File::open(path) {
        Ok(file) => load_file(file, MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(RuntimeSessionArchive::empty()),
        Err(error) => Err(error.into()),
    }
}

fn load_file(
    file: fs::File,
    max_archive_bytes: usize,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let max_archive_bytes = max_archive_bytes.min(MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES);
    let file_bytes = usize::try_from(file.metadata()?.len()).unwrap_or(usize::MAX);
    if file_bytes > max_archive_bytes {
        return Err(oversized_archive(file_bytes, max_archive_bytes));
    }

    let read_limit = max_archive_bytes.saturating_add(1) as u64;
    let reader = BufReader::with_capacity(ARCHIVE_READ_BUFFER_BYTES, file);
    let mut bounded = reader.take(read_limit);
    let decoded: Result<RuntimeSessionArchiveWirePayload, serde_json::Error> =
        serde_json::from_reader(&mut bounded);
    if bounded.limit() == 0 {
        return Err(oversized_archive(
            max_archive_bytes.saturating_add(1),
            max_archive_bytes,
        ));
    }

    let mut archive = RuntimeSessionArchive::from_deserialized_payload(decoded?.into());
    archive.normalize_slot_metadata();
    archive.record_normalized();
    archive.ensure_supported()?;
    archive.record_validated();
    Ok(archive)
}

fn oversized_archive(estimated_bytes: usize, limit_bytes: usize) -> RuntimeSessionArchiveError {
    RuntimeSessionArchiveError::ArtifactTooLarge {
        estimated_bytes,
        limit_bytes,
    }
}
