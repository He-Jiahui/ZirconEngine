use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

const VIEWER_PROJECT_CACHE_VERSION: u32 = 4;
const VIEWER_IBL_CACHE_DIRECTORY: &str = "zircon_shader_pbr_viewer_ibl_cache";

#[cfg(test)]
static VIEWER_TEST_ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct ViewerWorkPaths {
    project_root: PathBuf,
    ibl_cache_root: PathBuf,
    renderdoc_capture_template: PathBuf,
    terminal_outcome_path: PathBuf,
}

impl ViewerWorkPaths {
    pub(crate) fn new(work_dir: &Path, ibl_cache_override: Option<&Path>) -> Self {
        Self {
            project_root: work_dir.join(format!(
                "zircon_shader_pbr_viewer_project_v{VIEWER_PROJECT_CACHE_VERSION}"
            )),
            ibl_cache_root: ibl_cache_override
                .map(Path::to_path_buf)
                .unwrap_or_else(|| work_dir.join(VIEWER_IBL_CACHE_DIRECTORY)),
            renderdoc_capture_template: work_dir.join("renderdoc").join("zircon_shader_pbr_viewer"),
            terminal_outcome_path: work_dir.join("zircon_shader_pbr_viewer_terminal_outcome.json"),
        }
    }

    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(crate) fn ibl_cache_root(&self) -> &Path {
        &self.ibl_cache_root
    }

    pub(crate) fn renderdoc_capture_template(&self) -> &Path {
        &self.renderdoc_capture_template
    }

    pub(crate) fn terminal_outcome_path(&self) -> &Path {
        &self.terminal_outcome_path
    }
}

#[cfg(test)]
pub(crate) fn viewer_test_artifact_root(test_name: &str) -> PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("viewer crate must live below the workspace root");
    let workspace_is_on_c_drive = workspace_root
        .to_string_lossy()
        .to_ascii_lowercase()
        .starts_with("c:");
    let artifact_parent = if workspace_is_on_c_drive {
        PathBuf::from("D:/ZirconEngineTestArtifacts/zircon_shader_pbr_viewer")
    } else {
        workspace_root.join("docs/tests/runtime/shader/.viewer-test-artifacts")
    };
    let sequence = VIEWER_TEST_ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = artifact_parent.join(format!("{test_name}-{}-{sequence}", std::process::id()));
    std::fs::create_dir_all(&root).expect("viewer test artifact root should be created");
    root
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{viewer_test_artifact_root, ViewerWorkPaths};

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
    fn viewer_test_artifacts_do_not_use_the_system_temp_directory_or_c_drive() {
        let root = viewer_test_artifact_root("work-paths");

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
}
