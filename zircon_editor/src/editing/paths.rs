//! Path validation for mesh import and project files.

use std::path::PathBuf;

pub(crate) fn trimmed_path(value: &str, label: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} is empty"));
    }
    Ok(PathBuf::from(trimmed))
}

pub(crate) fn canonical_model_source_path(value: &str) -> Result<PathBuf, String> {
    let path = trimmed_path(value, "model import path")?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "obj" | "gltf" | "glb") {
        return Err("Only .obj, .gltf, and .glb imports are supported right now".to_string());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Cannot access model {}: {error}", path.display()))?;
    if !canonical.is_file() {
        return Err(format!("Model path is not a file: {}", canonical.display()));
    }
    Ok(canonical)
}
