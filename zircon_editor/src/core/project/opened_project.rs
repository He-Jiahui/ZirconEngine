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
