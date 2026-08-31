use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::ProjectActivationOperationId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectCloseOperation {
    project_root: PathBuf,
    operation_id: ProjectActivationOperationId,
}

impl ProjectCloseOperation {
    pub(crate) fn new(project_root: PathBuf, operation_id: ProjectActivationOperationId) -> Self {
        Self {
            project_root,
            operation_id,
        }
    }

    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(crate) const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }
}
