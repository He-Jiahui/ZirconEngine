use std::path::PathBuf;

use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedProject {
    pub root: PathBuf,
    pub summary: ProjectManifestSummary,
}
