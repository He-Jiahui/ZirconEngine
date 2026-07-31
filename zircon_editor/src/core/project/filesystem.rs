use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use super::ProjectAuthorityError;

const PROJECT_MANIFEST_FILE: &str = "zircon-project.toml";

pub(super) fn canonical_project_root(path: &Path) -> Result<PathBuf, ProjectAuthorityError> {
    reject_blank_project_path(path)?;
    let root = if path
        .file_name()
        .is_some_and(|name| name == OsStr::new(PROJECT_MANIFEST_FILE))
    {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    let root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| ProjectAuthorityError::CurrentDirectory { source })?
            .join(root)
    };
    reject_linked_components(&root)?;
    if root.exists() {
        fs::canonicalize(&root)
            .map_err(|source| ProjectAuthorityError::io("canonicalize project root", &root, source))
    } else {
        Ok(root)
    }
}

pub(super) fn validate_creation_target(root: &Path) -> Result<(), ProjectAuthorityError> {
    reject_blank_project_path(root)?;
    reject_linked_components(root)?;
    if root.is_file() {
        return Err(ProjectAuthorityError::TargetIsFile {
            path: root.to_path_buf(),
        });
    }
    if root.is_dir() {
        let mut entries = fs::read_dir(root)
            .map_err(|source| ProjectAuthorityError::io("read target directory", root, source))?;
        if entries
            .next()
            .transpose()
            .map_err(|source| {
                ProjectAuthorityError::io("read target directory entry", root, source)
            })?
            .is_some()
        {
            return Err(ProjectAuthorityError::TargetNotEmpty {
                path: root.to_path_buf(),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_canonical_existing_project_root(
    root: &Path,
) -> Result<(), ProjectAuthorityError> {
    if !root.is_dir() {
        return Err(ProjectAuthorityError::ProjectMissing {
            path: root.to_path_buf(),
        });
    }
    let manifest = root.join(PROJECT_MANIFEST_FILE);
    if !manifest.is_file() {
        return Err(ProjectAuthorityError::ManifestMissing { path: manifest });
    }
    Ok(())
}

fn reject_blank_project_path(path: &Path) -> Result<(), ProjectAuthorityError> {
    if path.as_os_str().is_empty() || path.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(ProjectAuthorityError::EmptyProjectPath);
    }
    Ok(())
}

fn reject_linked_components(path: &Path) -> Result<(), ProjectAuthorityError> {
    let mut existing = path;
    while !existing.exists() {
        let Some(parent) = existing.parent() else {
            break;
        };
        existing = parent;
    }
    for ancestor in existing.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|source| {
            ProjectAuthorityError::io("inspect project path", ancestor, source)
        })?;
        if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
            return Err(ProjectAuthorityError::LinkedPath {
                path: ancestor.to_path_buf(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::ProjectAuthorityError;
    use super::{canonical_project_root, validate_creation_target};

    #[test]
    fn project_root_validation_rejects_empty_and_blank_paths_before_filesystem_access() {
        for path in [Path::new(""), Path::new(" "), Path::new("\u{2003}")] {
            assert!(matches!(
                canonical_project_root(path),
                Err(ProjectAuthorityError::EmptyProjectPath)
            ));
            assert!(matches!(
                validate_creation_target(path),
                Err(ProjectAuthorityError::EmptyProjectPath)
            ));
        }
    }
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
