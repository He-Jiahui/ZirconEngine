use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn collect_files_with_extension(
    root: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read shader package {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read shader package {} entry: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files)?;
        } else if has_extension(&path, extension) {
            files.push(path);
        }
    }
    Ok(())
}

pub(super) fn meta_path_for_single_source(meta_path: &Path) -> PathBuf {
    let file_name = meta_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".zmeta"))
        .unwrap_or_default();
    meta_path.with_file_name(file_name)
}

pub(super) fn is_inside_compound_shader_source(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(parent_name) = parent.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    parent
        .parent()
        .map(|grandparent| grandparent.join(format!("{parent_name}.zmeta")).exists())
        .unwrap_or(false)
}

pub(super) fn is_zmeta(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.ends_with(".zmeta"))
}

pub(super) fn has_sidecar_zmeta(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    path.with_file_name(format!("{file_name}.zmeta")).exists()
}

pub(super) fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

pub(super) fn stable_label_for_path(asset_root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(asset_root).unwrap_or(path);
    let normalized = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    format!("asset-scan://{normalized}")
}
