use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::AssetImportError;

use super::is_meta_sidecar::is_meta_sidecar;

pub(super) fn collect_files_excluding_subtrees(
    root: &Path,
    files: &mut Vec<PathBuf>,
    excluded: &HashSet<PathBuf>,
) -> Result<(), AssetImportError> {
    collect_source_files(root, files, None, Some(excluded))
}

pub(super) fn collect_compound_files_with_limit(
    root: &Path,
    files: &mut Vec<PathBuf>,
    limit: usize,
) -> Result<(), AssetImportError> {
    collect_source_files(root, files, Some(limit), None)
}

fn collect_source_files(
    root: &Path,
    files: &mut Vec<PathBuf>,
    max_files: Option<usize>,
    excluded: Option<&HashSet<PathBuf>>,
) -> Result<(), AssetImportError> {
    collect_matching_files_with_limit(
        root,
        files,
        |path| {
            !is_meta_sidecar(path)
                && !crate::core::resource::io::is_atomic_write_transaction_path(path)
                && !crate::core::resource::io::transaction::is_project_transaction_sibling_path(
                    path,
                )
                && !is_auxiliary_source_file(path)
        },
        max_files,
        excluded,
    )
}

pub(super) fn visit_matching_files<F, V>(
    root: &Path,
    mut include: F,
    mut visit: V,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path) -> bool,
    V: FnMut(&Path) -> Result<(), AssetImportError>,
{
    let mut processing_error = None;
    let walk_result = walk_regular_files(root, None, &mut |_, path| {
        if processing_error.is_none() && include(path) {
            if let Err(error) = visit(path) {
                processing_error = Some(error);
            }
        }
        Ok(())
    });
    finish_matching_visit(walk_result, processing_error)
}

fn finish_matching_visit(
    walk_result: Result<(), AssetImportError>,
    processing_error: Option<AssetImportError>,
) -> Result<(), AssetImportError> {
    // The former collector finished the safe tree walk before processing any matches.
    // Keep its traversal diagnostics ahead of a metadata-processing failure.
    walk_result?;
    processing_error.map_or(Ok(()), Err)
}

fn collect_matching_files_with_limit<F>(
    root: &Path,
    files: &mut Vec<PathBuf>,
    mut include: F,
    max_files: Option<usize>,
    excluded: Option<&HashSet<PathBuf>>,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path) -> bool,
{
    walk_regular_files(root, excluded, &mut |directory, path| {
        if include(path) {
            if let Some(limit) = max_files {
                if files.len() >= limit {
                    return Err(AssetImportError::Parse(format!(
                        "compound source {} exceeds {limit}-file limit",
                        directory.display()
                    )));
                }
            }
            files.push(path.to_path_buf());
        }
        Ok(())
    })
}

fn walk_regular_files<F>(
    root: &Path,
    excluded: Option<&HashSet<PathBuf>>,
    visit: &mut F,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path, &Path) -> Result<(), AssetImportError>,
{
    if !root.exists() {
        return Ok(());
    }
    walk_regular_files_recursive(root, excluded, visit)
}

fn walk_regular_files_recursive<F>(
    directory: &Path,
    excluded: Option<&HashSet<PathBuf>>,
    visit: &mut F,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path, &Path) -> Result<(), AssetImportError>,
{
    let metadata = fs::symlink_metadata(directory)?;
    reject_link_or_reparse(directory, &metadata)?;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        reject_link_or_reparse(&path, &metadata)?;
        if metadata.is_dir() {
            if !excluded.is_some_and(|excluded| excluded.contains(&path)) {
                walk_regular_files_recursive(&path, excluded, visit)?;
            }
        } else if metadata.is_file() {
            visit(directory, &path)?;
        }
    }
    Ok(())
}

fn reject_link_or_reparse(path: &Path, metadata: &fs::Metadata) -> Result<(), AssetImportError> {
    if is_link_or_reparse(metadata) {
        return Err(AssetImportError::UnsafeProjectAssetLink {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn is_auxiliary_source_file(path: &Path) -> bool {
    // External glTF buffers and raw font binaries are source auxiliaries, not standalone assets.
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_auxiliary_source_extension)
}

fn is_auxiliary_source_extension(extension: &str) -> bool {
    match extension.len() {
        3 => ["bin", "ttf", "otf"]
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate)),
        4 => extension.eq_ignore_ascii_case("woff"),
        5 => extension.eq_ignore_ascii_case("woff2"),
        _ => false,
    }
}

#[cfg(test)]
#[path = "tests/collect_files.rs"]
mod tests;
