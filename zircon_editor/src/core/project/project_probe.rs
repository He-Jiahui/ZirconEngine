use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectProbe {
    root: PathBuf,
    summary: ProjectManifestSummary,
}

impl ProjectProbe {
    pub(super) fn new(root: PathBuf, summary: ProjectManifestSummary) -> Self {
        Self { root, summary }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn summary(&self) -> &ProjectManifestSummary {
        &self.summary
    }
}
