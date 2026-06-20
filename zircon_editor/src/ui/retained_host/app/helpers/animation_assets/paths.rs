use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn sibling_relative_path(model_relative_path: &Path, file_name: &Path) -> PathBuf {
    model_relative_path
        .parent()
        .map(|parent| parent.join(file_name))
        .unwrap_or_else(|| file_name.to_path_buf())
}

pub(super) fn sanitize_animation_asset_segment(
    name: Option<&str>,
    animation_index: usize,
) -> String {
    let sanitized = name
        .unwrap_or_default()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        format!("clip_{animation_index:03}")
    } else {
        sanitized
    }
}

pub(super) fn write_animation_asset_bytes(path: &Path, bytes: Vec<u8>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, bytes).map_err(|error| error.to_string())
}
