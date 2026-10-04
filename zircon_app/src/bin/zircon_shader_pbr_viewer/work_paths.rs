//! 查看器项目、IBL 与诊断证据的运行时路径组织。
//! 命名与建目录分离；测试数据目录由各测试清理，不用于编译产物。

use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

const VIEWER_PROJECT_CACHE_VERSION: u32 = 4;
const VIEWER_IBL_CACHE_DIRECTORY: &str = "zircon_shader_pbr_viewer_ibl_cache";

#[cfg(test)]
static VIEWER_TEST_ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 启动与场景构造共享的运行时工件命名；路径构造本身不建立目录或执行准入。
pub(crate) struct ViewerWorkPaths {
    project_root: PathBuf,
    ibl_cache_root: PathBuf,
    renderdoc_capture_template: PathBuf,
    terminal_outcome_path: PathBuf,
}

impl ViewerWorkPaths {
    /// 在已准入的工作目录下派生工件路径；独立 IBL 缓存可由多个启动目录复用。
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
/// 为测试建立独立的数据工件目录；测试完成后必须清理，与编译缓存分配无关。
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
#[path = "tests/work_paths.rs"]
mod tests;
