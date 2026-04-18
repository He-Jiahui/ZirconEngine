use std::path::Path;

use crate::{AssetImportError, ProjectManager};

use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub fn open_project(&self, root: impl AsRef<Path>) -> Result<(), AssetImportError> {
        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        self.sync_from_project(project)
    }
}
