use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::paths::has_extension;
use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};

/// Only a strict child of the scanned root is eligible for exclusion. This
/// keeps a shared snapshot root from silently reusing an inventory with a
/// different scan shape.
pub(super) fn nested_excluded_root_identity(
    root: &Path,
    excluded_root: Option<&Path>,
) -> Option<String> {
    let canonical_root = fs::canonicalize(root).ok()?;
    excluded_root
        .and_then(|path| fs::canonicalize(path).ok())
        .filter(|path| path.starts_with(&canonical_root) && path != &canonical_root)
        .map(|path| path.to_string_lossy().to_string())
}

pub(super) fn is_inventory_text_path(path: &Path) -> bool {
    has_extension(path, "wgsl")
        || has_extension(path, "zshader")
        || has_extension(path, "zmaterial")
}

pub(super) fn inventory_text_read_error(
    path: &Path,
    source: std::io::Error,
) -> ShaderPrewarmAssetScanError {
    if has_extension(path, "zshader") {
        ShaderPrewarmAssetScanError::ReadZShader {
            path: path.to_path_buf(),
            source,
        }
    } else if has_extension(path, "zmaterial") {
        ShaderPrewarmAssetScanError::ReadZMaterial {
            path: path.to_path_buf(),
            source,
        }
    } else {
        ShaderPrewarmAssetScanError::ReadWgsl {
            path: path.to_path_buf(),
            source,
        }
    }
}

pub(super) fn collect_file_paths(
    directory: &Path,
    canonical_root: &Path,
    canonical_excluded_root: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
    directories: &mut Vec<PathBuf>,
) -> ShaderPrewarmAssetScanResult<()> {
    let canonical_directory = fs::canonicalize(directory).map_err(|source| {
        ShaderPrewarmAssetScanError::ReadAssetRoot {
            path: directory.to_path_buf(),
            source,
        }
    })?;
    ensure_below_root(canonical_root, &canonical_directory)?;
    if canonical_excluded_root
        .is_some_and(|excluded_root| canonical_directory.starts_with(excluded_root))
    {
        return Ok(());
    }
    if !visited.insert(canonical_directory.clone()) {
        return Err(ShaderPrewarmAssetScanError::AssetInventoryDirectoryCycle {
            root: canonical_root.to_path_buf(),
            path: canonical_directory,
        });
    }
    directories.push(directory.to_path_buf());
    let mut entries = fs::read_dir(directory)
        .map_err(|source| ShaderPrewarmAssetScanError::ReadAssetRoot {
            path: directory.to_path_buf(),
            source,
        })?
        .map(|entry| {
            entry.map_err(|source| ShaderPrewarmAssetScanError::ReadAssetRootEntry {
                path: directory.to_path_buf(),
                source,
            })
        })
        .collect::<ShaderPrewarmAssetScanResult<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| {
            ShaderPrewarmAssetScanError::ReadAssetRoot {
                path: path.clone(),
                source,
            }
        })?;
        reject_link_or_reparse(canonical_root, &path, &metadata)?;
        let canonical_path = fs::canonicalize(&path).map_err(|source| {
            ShaderPrewarmAssetScanError::ReadAssetRoot {
                path: path.clone(),
                source,
            }
        })?;
        ensure_below_root(canonical_root, &canonical_path)?;
        if metadata.is_dir() {
            collect_file_paths(
                &path,
                canonical_root,
                canonical_excluded_root,
                visited,
                paths,
                directories,
            )?;
        } else {
            paths.push(path);
        }
    }
    Ok(())
}

fn ensure_below_root(root: &Path, path: &Path) -> ShaderPrewarmAssetScanResult<()> {
    if path.starts_with(root) {
        Ok(())
    } else {
        Err(ShaderPrewarmAssetScanError::AssetInventoryPathEscapesRoot {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    }
}

pub(super) fn reject_link_or_reparse(
    root: &Path,
    path: &Path,
    metadata: &fs::Metadata,
) -> ShaderPrewarmAssetScanResult<()> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) {
        Err(ShaderPrewarmAssetScanError::UnsafeAssetInventoryLink {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    } else {
        Ok(())
    }
}

#[cfg(windows)]
pub(super) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
pub(super) fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
