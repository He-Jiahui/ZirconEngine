#[cfg(windows)]
use std::path::PathBuf;

#[cfg(windows)]
use crate::asset::AssetUri;

#[cfg(windows)]
use super::AssetImportError;

#[cfg(windows)]
#[test]
fn ambiguous_project_asset_uri_keeps_operation_paths_but_displays_virtual_paths() {
    let operation_paths = vec![
        PathBuf::from(r"\\?\C:\projects\forest\assets\cube.obj"),
        PathBuf::from(r"\\?\C:\projects\forest\shared-assets\cube.obj"),
    ];
    let error = AssetImportError::ambiguous_project_asset_uri(
        AssetUri::parse("res://models/cube.obj").unwrap(),
        operation_paths.clone(),
    );

    assert!(matches!(
        &error,
        AssetImportError::AmbiguousProjectAssetUri {
            paths,
            display_paths,
            ..
        } if paths == &operation_paths
            && display_paths == &vec![
                PathBuf::from(r"C:\projects\forest\assets\cube.obj"),
                PathBuf::from(r"C:\projects\forest\shared-assets\cube.obj"),
            ]
    ));
    let diagnostic = error.to_string();
    assert!(!diagnostic.contains(r"\\?\"));
    assert!(diagnostic.contains(r"C:\projects\forest\assets\cube.obj"));
    assert!(diagnostic.contains(r"C:\projects\forest\shared-assets\cube.obj"));
}

#[cfg(windows)]
#[test]
fn duplicate_project_asset_uri_displays_windows_virtual_paths() {
    let error = AssetImportError::DuplicateProjectAssetUri {
        uri: AssetUri::parse("res://models/cube.obj").unwrap(),
        first: PathBuf::from(r"\\?\C:\projects\forest\assets\cube.obj"),
        second: PathBuf::from(r"\\?\C:\projects\forest\shared-assets\cube.obj"),
    };

    let diagnostic = error.to_string();
    assert!(!diagnostic.contains(r"\\?\"));
    assert!(diagnostic.contains(r"C:\projects\forest\assets\cube.obj"));
    assert!(diagnostic.contains(r"C:\projects\forest\shared-assets\cube.obj"));
}

#[cfg(windows)]
#[test]
fn ambiguous_project_source_path_keeps_operation_roots_but_displays_virtual_paths() {
    let operation_path = PathBuf::from(r"\\?\C:\projects\forest\assets\cube.obj");
    let operation_roots = vec![
        PathBuf::from(r"\\?\C:\projects\forest\assets"),
        PathBuf::from(r"\\?\C:\projects\forest\shared-assets"),
    ];
    let error = AssetImportError::ambiguous_project_source_path(
        operation_path.clone(),
        operation_roots.clone(),
    );

    assert!(matches!(
        &error,
        AssetImportError::AmbiguousProjectSourcePath {
            path,
            roots,
            display_path,
            display_roots,
        } if path == &operation_path
            && roots == &operation_roots
            && display_path == &PathBuf::from(r"C:\projects\forest\assets\cube.obj")
            && display_roots == &vec![
                PathBuf::from(r"C:\projects\forest\assets"),
                PathBuf::from(r"C:\projects\forest\shared-assets"),
            ]
    ));
    let diagnostic = error.to_string();
    assert!(!diagnostic.contains(r"\\?\"));
    assert!(diagnostic.contains(r"C:\projects\forest\assets"));
    assert!(diagnostic.contains(r"C:\projects\forest\shared-assets"));
}

#[test]
fn registry_import_source_preserves_typed_registry_and_io_chain() {
    let source = crate::asset::AssetRegistryError::Io {
        path: std::path::PathBuf::from("typed-registry"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "registry denied"),
    };
    let display = format!("asset registry index failed: {source}");
    let error = crate::asset::AssetImportError::from(source);
    assert_eq!(error.to_string(), display);
    let source = std::error::Error::source(&error).expect("typed registry source");
    assert!(source
        .downcast_ref::<crate::asset::AssetRegistryError>()
        .is_some());
    let io = source.source().expect("registry I/O source");
    assert_eq!(
        io.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::PermissionDenied
    );
}

#[test]
fn ui_v2_import_source_preserves_typed_document_identity() {
    let source = crate::asset::assets::UiV2AssetDocumentError::ComponentRequiresZui;
    let display = format!(".zui asset document failed while typed source: {source}");
    let error = crate::asset::AssetImportError::UiV2Document {
        context: "typed source",
        source: source.into(),
    };
    assert_eq!(error.to_string(), display);
    assert!(matches!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<crate::asset::assets::UiV2AssetDocumentError>(),
        Some(crate::asset::assets::UiV2AssetDocumentError::ComponentRequiresZui)
    ));
}
