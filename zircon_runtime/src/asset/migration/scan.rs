use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::{AssetImporter, AssetImporterDescriptor};

pub(super) struct RecognizedSource {
    pub(super) path: PathBuf,
    pub(super) descriptor: AssetImporterDescriptor,
}

pub(super) fn prospective_sidecar_targets(recognized: &[RecognizedSource]) -> Vec<PathBuf> {
    recognized
        .iter()
        .filter_map(|source| {
            let name = source.path.file_name()?.to_str()?;
            Some(source.path.with_file_name(format!("{name}.zmeta")))
        })
        .collect()
}

pub(super) fn recognized_sources(
    roots: &[PathBuf],
) -> Result<Vec<RecognizedSource>, std::io::Error> {
    let importer = AssetImporter::default();
    let mut paths = Vec::new();
    for root in roots {
        collect_recognized(root, &mut paths)?;
    }
    paths.sort();
    paths.dedup();
    Ok(paths
        .into_iter()
        .filter_map(|path| {
            importer
                .descriptor_for_source(&path)
                .ok()
                .map(|descriptor| RecognizedSource { path, descriptor })
        })
        .collect())
}

pub(super) fn supported_authoring_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for root in roots {
        collect(root, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub(super) fn supported_transaction_targets(
    roots: &[PathBuf],
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for root in roots {
        collect_transaction_targets(root, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_transaction_targets(
    path: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    let metadata = fs::symlink_metadata(path)?;
    if is_link_or_reparse(&metadata) {
        return Ok(());
    }
    if metadata.is_file() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if is_supported(path) || name.ends_with(".zmeta") || name.ends_with(".meta.toml") {
            files.push(path.to_path_buf());
            if let Some(stem) = name.strip_suffix(".meta.toml") {
                files.push(path.with_file_name(format!("{stem}.zmeta")));
            } else if let Some(stem) = name.strip_suffix(".zmeta") {
                files.push(path.with_file_name(format!("{stem}.meta.toml")));
            }
        }
        return Ok(());
    }
    if metadata.is_dir() {
        let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            if child.file_name() != ".zircon" {
                collect_transaction_targets(&child.path(), files)?;
            }
        }
    }
    Ok(())
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    let metadata = fs::symlink_metadata(path)?;
    if is_link_or_reparse(&metadata) {
        return Ok(());
    }
    if metadata.is_file() {
        if is_supported(path) {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        if child.file_name() == ".zircon" {
            continue;
        }
        collect(&child.path(), files)?;
    }
    Ok(())
}

fn collect_recognized(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    let metadata = fs::symlink_metadata(path)?;
    if is_link_or_reparse(&metadata) {
        return Ok(());
    }
    if metadata.is_file() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if !name.ends_with(".zmeta") && !name.ends_with(".meta.toml") && !is_auxiliary_source(path)
        {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    if metadata.is_dir() {
        let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            if child.file_name() != ".zircon" {
                collect_recognized(&child.path(), files)?;
            }
        }
    }
    Ok(())
}

fn is_auxiliary_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bin")
                || extension.eq_ignore_ascii_case("ttf")
                || extension.eq_ignore_ascii_case("otf")
                || extension.eq_ignore_ascii_case("woff")
                || extension.eq_ignore_ascii_case("woff2")
        })
}

fn is_supported(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.ends_with(".scene.toml") || name.ends_with(".model.toml") || name.ends_with(".zmaterial")
}
