use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug)]
pub struct CreatedProject {
    pub root: PathBuf,
    pub summary: ProjectManifestSummary,
    // The creation transaction establishes this first project generation. Callers transfer it
    // into the host instead of reopening the just-committed manifest from disk.
    project: ProjectManager,
}

impl CreatedProject {
    pub fn project(&self) -> &ProjectManager {
        &self.project
    }

    pub fn into_project(self) -> ProjectManager {
        self.project
    }
}

impl PartialEq for CreatedProject {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.summary == other.summary
    }
}

impl Eq for CreatedProject {}
