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
    assert!(!error.to_string().contains(r"\\?\"));
}
