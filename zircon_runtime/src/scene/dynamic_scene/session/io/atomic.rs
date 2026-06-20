use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::load_save::preview_save_to_path;
use super::support::{ensure_parent_dir, temporary_archive_path};

pub(in crate::scene::dynamic_scene::session) fn save_to_path_atomically(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    let path = path.as_ref();
    preview_save_to_path(archive, path)?;
    ensure_parent_dir(path)?;
    let temp_path = temporary_archive_path(path, "tmp");
    let payload = archive.to_versioned_json_pretty()?;
    if let Err(error) = fs::write(&temp_path, payload) {
        let _ = fs::remove_file(&temp_path);
        return Err(error.into());
    }

    let backup_path = prepare_existing_target_backup(path, &temp_path)?;

    match fs::rename(&temp_path, path) {
        Ok(()) => {
            if let Some(backup_path) = backup_path.as_ref() {
                let _ = fs::remove_file(backup_path);
            }
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            restore_existing_target_backup(path, backup_path.as_deref());
            Err(error.into())
        }
    }
}

fn prepare_existing_target_backup(
    path: &Path,
    temp_path: &Path,
) -> Result<Option<PathBuf>, RuntimeSessionArchiveError> {
    if !path.exists() {
        return Ok(None);
    }

    if !path.is_file() {
        let _ = fs::remove_file(temp_path);
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "runtime session archive target path is not a file",
        )
        .into());
    }

    let backup_path = temporary_archive_path(path, "bak");
    if let Err(error) = fs::rename(path, &backup_path) {
        let _ = fs::remove_file(temp_path);
        return Err(error.into());
    }
    Ok(Some(backup_path))
}

fn restore_existing_target_backup(path: &Path, backup_path: Option<&Path>) {
    if let Some(backup_path) = backup_path {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
        let _ = fs::rename(backup_path, path);
    }
}
