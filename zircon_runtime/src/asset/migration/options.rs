use std::path::PathBuf;

use super::AssetMigrationMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMigrationOptions {
    pub(super) project_root: PathBuf,
    pub(super) mode: AssetMigrationMode,
}

impl AssetMigrationOptions {
    pub fn new(project_root: impl Into<PathBuf>, mode: AssetMigrationMode) -> Self {
        Self {
            project_root: project_root.into(),
            mode,
        }
    }

    pub fn project_root(&self) -> &std::path::Path {
        &self.project_root
    }

    pub fn mode(&self) -> AssetMigrationMode {
        self.mode
    }
}
