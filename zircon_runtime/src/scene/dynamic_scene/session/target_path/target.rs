use std::fs;
use std::io;
use std::path::Path;

use super::super::RuntimeSessionArchiveError;

pub(in crate::scene::dynamic_scene::session) fn target_file_will_replace(
    target_path: &Path,
    target_label: &str,
) -> Result<bool, RuntimeSessionArchiveError> {
    reject_non_directory_parent(target_path, target_label)?;
    match fs::metadata(target_path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{target_label} path is not a file"),
        )
        .into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn reject_non_directory_parent(
    target_path: &Path,
    target_label: &str,
) -> Result<(), RuntimeSessionArchiveError> {
    let Some(parent) = target_path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    match fs::metadata(parent) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{target_label} parent path is not a directory"),
        )
        .into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
