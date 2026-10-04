use zircon_runtime::asset::{
    project::{EditorDocumentScope, ProjectManager},
    AssetUri,
};
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::Scene;

use super::editor_project_document::EditorProjectDocument;
use super::editor_workspace_document::encode_editor_workspace_document;
use super::project_editor_workspace::ProjectEditorWorkspace;

impl EditorProjectDocument {
    pub fn save_scene_to_project(
        project: &ProjectManager,
        scene_uri: &AssetUri,
        world: &Scene,
        editor_workspace: Option<&ProjectEditorWorkspace>,
    ) -> Result<(), SceneProjectError> {
        let root = project.paths().root();
        let scope = EditorDocumentScope::new(
            root.to_path_buf(),
            project.catalog_input_generation().sequence(),
        );
        let scoped = project
            .scoped_editor_document(&scope)
            .map_err(SceneProjectError::Asset)?;
        let outcome = match editor_workspace {
            Some(workspace) => {
                let workspace_document =
                    encode_editor_workspace_document(workspace).map_err(std::io::Error::other)?;
                scoped.commit_scene_workspace(scene_uri, world, workspace_document.as_bytes())?
            }
            None => scoped.commit_scene_without_workspace(scene_uri, world)?,
        };
        outcome.ensure_durable()
    }
}
