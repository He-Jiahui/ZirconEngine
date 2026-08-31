use std::path::PathBuf;

use zircon_runtime::asset::project::{ProjectManager, ResolvedProjectPath};
use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug)]
pub struct CreatedProject {
    pub root: PathBuf,
    pub summary: ProjectManifestSummary,
    identity: ResolvedProjectPath,
    // The creation transaction establishes this first project generation. Callers transfer it
    // into the host instead of reopening the just-committed manifest from disk.
    project: ProjectManager,
}

impl CreatedProject {
    pub(super) fn new(
        identity: ResolvedProjectPath,
        summary: ProjectManifestSummary,
        project: ProjectManager,
    ) -> Self {
        Self {
            root: identity.operation_path().to_path_buf(),
            summary,
            identity,
            project,
        }
    }

    pub fn identity(&self) -> &ResolvedProjectPath {
        &self.identity
    }

    pub fn project(&self) -> &ProjectManager {
        &self.project
    }

    pub fn into_project(self) -> ProjectManager {
        self.project
    }
}

impl PartialEq for CreatedProject {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity && self.summary == other.summary
    }
}

impl Eq for CreatedProject {}
