use crate::{AssetRegistry, ProjectManifest, ProjectPaths};

use super::ProjectManager;

impl ProjectManager {
    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }
}
