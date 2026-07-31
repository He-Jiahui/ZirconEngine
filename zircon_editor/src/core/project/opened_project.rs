use std::path::Path;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug)]
pub struct OpenedProject {
    project: ProjectManager,
    summary: ProjectManifestSummary,
}

impl OpenedProject {
    pub(super) fn new(project: ProjectManager) -> Self {
        let summary = project.manifest().summary();
        Self { project, summary }
    }

    pub fn root(&self) -> &Path {
        self.project.paths().root()
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
