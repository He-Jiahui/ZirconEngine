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
mod tests {
    use super::{canonical_model_source_path, ModelSourcePathError};
    use std::io::ErrorKind;
    use std::path::PathBuf;

    #[test]
    fn empty_model_source_path_is_rejected_with_a_typed_error() {
        assert!(matches!(
            canonical_model_source_path("   "),
            Err(ModelSourcePathError::EmptyPath)
        ));
    }

    #[test]
    fn unsupported_model_source_extension_preserves_the_input_path() {
        let input = "assets/models/character.fbx";

        assert!(matches!(
            canonical_model_source_path(input),
            Err(ModelSourcePathError::UnsupportedExtension { ref path })
                if path == &PathBuf::from(input)
        ));
    }

    #[test]
    fn inaccessible_model_source_preserves_the_io_error() {
        let input = "missing-zircon-editor-model-source.obj";

        assert!(matches!(
            canonical_model_source_path(input),
            Err(ModelSourcePathError::Canonicalize { ref path, ref source })
                if path == &PathBuf::from(input) && source.kind() == ErrorKind::NotFound
        ));
    }

    #[test]
    fn canonicalize_error_text_keeps_the_io_detail_for_the_host_boundary() {
        let error = ModelSourcePathError::Canonicalize {
            path: PathBuf::from("assets/models/character.obj"),
            source: std::io::Error::new(ErrorKind::PermissionDenied, "test access denied"),
        };

        assert_eq!(
            error.to_string(),
            "cannot access model source assets/models/character.obj: test access denied"
        );
    }
}
