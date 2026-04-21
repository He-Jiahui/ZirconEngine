use std::path::Path;

use crate::ui::workbench::project::EditorProjectDocument;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        self.host.open_project(path)
    }

    pub fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<(), EditorError> {
        self.host.save_project(path, world)
    }

    pub fn create_runtime_level(
        &self,
        scene: zircon_runtime::scene::Scene,
    ) -> Result<zircon_runtime::scene::LevelSystem, EditorError> {
        self.host.create_runtime_level(scene)
    }
}
