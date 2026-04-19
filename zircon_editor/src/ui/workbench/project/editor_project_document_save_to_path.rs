use std::fs;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::Scene;

use super::editor_project_document::EditorProjectDocument;
use super::editor_workspace_persistence::save_editor_workspace;
use super::project_editor_workspace::ProjectEditorWorkspace;
use super::project_root_path::project_root_path;
use super::runtime_asset_helpers::invalid_data;

impl EditorProjectDocument {
    pub fn save_to_path(
        path: impl AsRef<std::path::Path>,
        world: &Scene,
        editor_workspace: Option<&ProjectEditorWorkspace>,
    ) -> Result<(), SceneProjectError> {
        let root = project_root_path(path)?;
        Self::ensure_runtime_assets(&root)?;

        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        let scene = world.to_scene_asset(&project)?;
        let scene_path = project.source_path_for_uri(&project.manifest().default_scene)?;
        if let Some(parent) = scene_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(
            scene_path,
            scene
                .to_toml_string()
                .map_err(|error| invalid_data(error.to_string()))?,
        )?;
        save_editor_workspace(&root, editor_workspace)?;
        Ok(())
    }
}
