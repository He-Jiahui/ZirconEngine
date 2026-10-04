use std::path::Path;

use super::super::ProjectAuthorityError;
use super::{canonical_project_root, protect_scene_path, validate_creation_target};

#[test]
fn project_root_validation_rejects_empty_and_blank_paths_before_filesystem_access() {
    for path in [Path::new(""), Path::new(" "), Path::new("\u{2003}")] {
        assert!(matches!(
            canonical_project_root(path),
            Err(ProjectAuthorityError::EmptyProjectPath)
        ));
        assert!(matches!(
            validate_creation_target(path),
            Err(ProjectAuthorityError::EmptyProjectPath)
        ));
    }
}

#[cfg(not(windows))]
#[test]
fn scene_path_guard_rejects_unsupported_platforms_before_loading_or_publishing() {
    let root =
        std::env::temp_dir().join(format!("zircon-editor-scene-guard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();

    let error =
        protect_scene_path(&root, &root.join("assets/scenes/main.scene.toml"), false).unwrap_err();
    assert!(matches!(
        error,
        ProjectAuthorityError::SceneTarget { reason, .. }
            if reason.contains("Windows no-follow lease support")
    ));

    std::fs::remove_dir_all(root).unwrap();
}
