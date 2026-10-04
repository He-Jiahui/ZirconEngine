//! Path validation for mesh import and project files.

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ModelSourcePathError {
    #[error("model import path is empty")]
    EmptyPath,
    #[error("unsupported model source extension for {path}; expected .obj, .gltf, or .glb")]
    UnsupportedExtension { path: PathBuf },
    #[error("cannot access model source {path}: {source}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("model source path is not a file: {path}")]
    NotAFile { path: PathBuf },
}

fn trimmed_path(value: &str) -> Result<PathBuf, ModelSourcePathError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ModelSourcePathError::EmptyPath);
    }
    Ok(PathBuf::from(trimmed))
}

pub(crate) fn canonical_model_source_path(value: &str) -> Result<PathBuf, ModelSourcePathError> {
    let path = trimmed_path(value)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "obj" | "gltf" | "glb") {
        return Err(ModelSourcePathError::UnsupportedExtension { path });
    }
    let canonical = path
        .canonicalize()
        .map_err(|source| ModelSourcePathError::Canonicalize { path, source })?;
    if !canonical.is_file() {
        return Err(ModelSourcePathError::NotAFile { path: canonical });
    }
    Ok(canonical)
}

#[cfg(test)]
#[path = "tests/paths.rs"]
mod tests;
