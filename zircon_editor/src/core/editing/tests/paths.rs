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
