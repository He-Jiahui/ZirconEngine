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

    /// Prepare and validate the directory RenderDoc needs before configuring its template.
    pub(crate) fn prepare_renderdoc_capture_template(&self) -> Result<(), String> {
        let parent = self
            .renderdoc_capture_template
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| {
                format!(
                    "RenderDoc capture template has no parent directory: {}",
                    self.renderdoc_capture_template.display()
                )
            })?;
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create RenderDoc capture template directory {}: {error}",
                parent.display()
            )
        })?;
        if !parent.is_dir() {
            return Err(format!(
                "RenderDoc capture template parent is not a directory: {}",
                parent.display()
            ));
        }
        Ok(())
    }

    pub(crate) fn terminal_outcome_path(&self) -> &Path {
        &self.terminal_outcome_path
    }
}

#[cfg(test)]
/// 为测试建立独立的数据工件目录；测试完成后必须清理，与编译缓存分配无关。
pub(crate) fn viewer_test_artifact_root(test_name: &str) -> PathBuf {
    const CONFIGURATION: &str = "ZIRCON_VIEWER_TEST_ARTIFACT_ROOT";
    let configured_parent = PathBuf::from(
        std::env::var_os(CONFIGURATION)
            .expect("ZIRCON_VIEWER_TEST_ARTIFACT_ROOT must name a registered test artifact parent"),
    );
    assert!(
        configured_parent.is_absolute() && configured_parent.is_dir(),
        "viewer test artifact parent must be an absolute existing directory"
    );
    let physical_root = ["D:/cargo-targets", "E:/cargo-targets", "F:/cargo-targets"]
        .into_iter()
        .map(Path::new)
        .find(|root| configured_parent.starts_with(root))
        .expect("viewer test artifact parent must be below a literal drive-root cargo-targets");
    let admitted_root = crate::args::resolve_non_c_artifact_path(CONFIGURATION, physical_root)
        .expect("viewer test artifact drive root must pass physical path admission");
    assert_eq!(
        admitted_root, physical_root,
        "viewer test artifact drive root must not resolve through an alias"
    );
    let artifact_parent =
        crate::args::resolve_non_c_artifact_path(CONFIGURATION, &configured_parent)
            .expect("viewer test artifact parent must pass physical path admission");
    assert!(
        artifact_parent.is_dir() && artifact_parent.starts_with(&admitted_root),
        "viewer test artifact parent must physically remain below the admitted cargo-targets root"
    );
    let sequence = VIEWER_TEST_ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = artifact_parent.join(format!("{test_name}-{}-{sequence}", std::process::id()));
    assert_eq!(
        root.parent(),
        Some(artifact_parent.as_path()),
        "viewer test name must identify a direct child of the registered parent"
    );
    std::fs::create_dir(&root).expect("unique viewer test artifact root should be created");
    root
}

#[cfg(test)]
#[path = "tests/work_paths.rs"]
mod tests;
