//! 将已打开的运行时项目、清单摘要及选定物理身份作为同一结果交接；调用端可继续消费项目所有者，但不能从显示路径重建操作身份。
use std::path::Path;

use zircon_runtime::asset::project::{ProjectManager, ResolvedProjectPath};
use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug)]
pub struct OpenedProject {
    identity: ResolvedProjectPath,
    project: ProjectManager,
    summary: ProjectManifestSummary,
}

impl OpenedProject {
    pub(super) fn new(project: ProjectManager, identity: ResolvedProjectPath) -> Self {
        let summary = project.manifest().summary();
        Self {
            identity,
            project,
            summary,
        }
    }

    pub fn root(&self) -> &Path {
        self.identity.operation_path()
    }

    pub fn identity(&self) -> &ResolvedProjectPath {
        &self.identity
    }

    pub fn summary(&self) -> &ProjectManifestSummary {
        &self.summary
    }

    pub fn project(&self) -> &ProjectManager {
        &self.project
    }

    pub fn into_project(self) -> ProjectManager {
        self.project
    }
}
