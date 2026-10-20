use std::path::{Path, PathBuf};

use super::{viewer_test_artifact_root, ViewerWorkPaths};

struct TestArtifactDirectory(PathBuf);

impl Drop for TestArtifactDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn viewer_work_paths_keep_project_and_default_ibl_cache_under_the_work_root() {
    let paths = ViewerWorkPaths::new(Path::new("E:/viewer-work"), None);

    assert_eq!(
        paths.project_root(),
        Path::new("E:/viewer-work/zircon_shader_pbr_viewer_project_v4")
    );
    assert_eq!(
        paths.ibl_cache_root(),
        Path::new("E:/viewer-work/zircon_shader_pbr_viewer_ibl_cache")
    );
    assert_eq!(
        paths.renderdoc_capture_template(),
        Path::new("E:/viewer-work/renderdoc/zircon_shader_pbr_viewer")
    );
    assert_eq!(
        paths.terminal_outcome_path(),
        Path::new("E:/viewer-work/zircon_shader_pbr_viewer_terminal_outcome.json")
    );
}

#[test]
fn explicit_ibl_cache_directory_overrides_the_work_root_default() {
    let paths = ViewerWorkPaths::new(
        Path::new("E:/viewer-work"),
        Some(Path::new("E:/dedicated-ibl-cache")),
    );

    assert_eq!(
        paths.ibl_cache_root(),
        PathBuf::from("E:/dedicated-ibl-cache")
    );
}

#[test]
fn renderdoc_capture_parent_is_prepared_for_first_run() {
    let root = viewer_test_artifact_root("viewer-renderdoc-first-run");
    let _cleanup = TestArtifactDirectory(root.clone());
    let work_dir = root.join("work");
    std::fs::create_dir_all(&work_dir).expect("registered first-run work directory is created");
    let paths = ViewerWorkPaths::new(&work_dir, None);
    let capture_parent = paths
        .renderdoc_capture_template()
        .parent()
        .expect("capture template has a parent");
    assert!(!capture_parent.exists());

    paths
        .prepare_renderdoc_capture_template()
        .expect("first-run capture parent should be created and validated");

    assert!(capture_parent.is_dir());
}

#[test]
fn viewer_test_artifacts_do_not_use_the_system_temp_directory_or_c_drive() {
    let root = viewer_test_artifact_root("work-paths");
    let configured_parent = PathBuf::from(
        std::env::var_os("ZIRCON_VIEWER_TEST_ARTIFACT_ROOT")
            .expect("viewer test artifact parent must be explicitly configured"),
    );
    let admitted_parent = crate::args::resolve_non_c_artifact_path(
        "ZIRCON_VIEWER_TEST_ARTIFACT_ROOT",
        &configured_parent,
    )
    .expect("configured viewer test artifact parent must pass physical admission");
    assert_eq!(root.parent(), Some(admitted_parent.as_path()));

    assert!(
        !root
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with("c:"),
        "viewer test artifacts must remain outside C:"
    );
    assert!(
        !root.starts_with(std::env::temp_dir()),
        "viewer test artifacts must not use the system temporary directory"
    );
    std::fs::remove_dir_all(root).expect("viewer test artifact root should be removed");
}
