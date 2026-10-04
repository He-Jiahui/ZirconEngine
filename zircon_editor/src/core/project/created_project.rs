use std::path::PathBuf;

use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime_interface::project::ProjectManifestSummary;

use super::ProjectPreflightReceipt;

#[derive(Clone, Debug)]
pub struct CreatedProject {
    pub root: PathBuf,
    pub summary: ProjectManifestSummary,
    preflight: ProjectPreflightReceipt,
}

impl CreatedProject {
    pub(super) fn new(preflight: ProjectPreflightReceipt) -> Self {
        Self {
            root: preflight.root().to_path_buf(),
            summary: preflight.summary().clone(),
            preflight,
        }
    }

    pub fn identity(&self) -> &ResolvedProjectPath {
        self.preflight.resolved_project_path()
    }

    pub fn preflight(&self) -> &ProjectPreflightReceipt {
        &self.preflight
    }

    pub fn into_preflight(self) -> ProjectPreflightReceipt {
        self.preflight
    }
}

impl PartialEq for CreatedProject {
    fn eq(&self, other: &Self) -> bool {
        self.preflight == other.preflight
    }
}

impl Eq for CreatedProject {}
