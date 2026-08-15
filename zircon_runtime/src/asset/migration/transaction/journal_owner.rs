use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::migration::AssetMigrationError;
use crate::asset::project::ProjectPaths;
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::core::resource::io::ensure_parent_directories;

use super::JOURNAL_DIRECTORY;

pub(super) fn ensure_journal_directory(
    project_root: &Path,
) -> Result<PathBuf, AssetMigrationError> {
    validate_directory(project_root, project_root, "project root")?;
    let owner = project_root.join(".zircon");
    let directory = owner.join(JOURNAL_DIRECTORY);
    if !directory.exists() {
        ensure_parent_directories(&directory.join(".journal-owner"))
            .map_err(|error| invalid(&directory, error.to_string()))?;
    }
    validate_directory(project_root, &owner, "journal owner")?;
    validate_directory(project_root, &directory, "journal directory")?;
    Ok(directory)
}

pub(super) fn existing_journal_directory(
    project_root: &Path,
) -> Result<Option<PathBuf>, AssetMigrationError> {
    validate_directory(project_root, project_root, "project root")?;
    let owner = project_root.join(".zircon");
    if !owner.exists() {
        return Ok(None);
    }
    validate_directory(project_root, &owner, "journal owner")?;
    let directory = owner.join(JOURNAL_DIRECTORY);
    if !directory.exists() {
        return Ok(None);
    }
    validate_directory(project_root, &directory, "journal directory")?;
    Ok(Some(directory))
}

fn validate_directory(
    project_root: &Path,
    path: &Path,
    label: &str,
) -> Result<(), AssetMigrationError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| invalid(path, error.to_string()))?;
    if is_link_or_reparse(&metadata) || !metadata.is_dir() {
        return Err(invalid(path, format!("{label} must be a real directory")));
    }
    let canonical_root = ProjectPaths::resolve_existing_path(project_root)
        .map_err(|error| invalid(project_root, error.to_string()))?;
    let canonical = ProjectPaths::resolve_existing_path(path)
        .map_err(|error| invalid(path, error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(invalid(path, format!("{label} escapes project root")));
    }
    Ok(())
}

fn invalid(path: &Path, reason: impl Into<String>) -> AssetMigrationError {
    AssetMigrationError::InvalidJournal {
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}
