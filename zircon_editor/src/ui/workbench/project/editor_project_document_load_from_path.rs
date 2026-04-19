use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::Scene;

use super::editor_project_document::EditorProjectDocument;
use super::editor_workspace_persistence::load_editor_workspace;
use super::project_root_path::project_root_path;

impl EditorProjectDocument {
    pub fn load_from_path(path: impl AsRef<std::path::Path>) -> Result<Self, SceneProjectError> {
        let root = project_root_path(path)?;
        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;

        Ok(Self {
            root_path: root.clone(),
            manifest: project.manifest().clone(),
            world: Scene::load_scene_from_uri(&project, &project.manifest().default_scene)?,
            editor_workspace: load_editor_workspace(&root)?,
        })
    }
}
