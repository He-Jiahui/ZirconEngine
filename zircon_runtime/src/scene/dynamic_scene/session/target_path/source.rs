use std::fs;
use std::io;
use std::path::Path;

use super::super::RuntimeSessionArchiveError;

pub(in crate::scene::dynamic_scene::session) fn reject_same_archive_paths(
    source_path: &Path,
    target_path: &Path,
    operation_label: &str,
) -> Result<(), RuntimeSessionArchiveError> {
    if archive_paths_match(source_path, target_path)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{operation_label} target archive path must differ from source archive path"),
        )
        .into());
    }
    Ok(())
}

fn archive_paths_match(
    source_path: &Path,
    target_path: &Path,
) -> Result<bool, RuntimeSessionArchiveError> {
    if source_path == target_path {
        return Ok(true);
    }

    match (fs::canonicalize(source_path), fs::canonicalize(target_path)) {
        (Ok(source_path), Ok(target_path)) => Ok(source_path == target_path),
        (Err(error), _) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
        (_, Err(error)) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(false),
    }
}
