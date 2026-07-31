use std::io;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::Scene;

use super::editor_project_document::EditorProjectDocument;
use super::editor_workspace_persistence::{
    capture_editor_workspace, restore_editor_workspace, save_editor_workspace,
};
use super::project_editor_workspace::ProjectEditorWorkspace;

impl EditorProjectDocument {
    pub fn save_to_project(
        project: &ProjectManager,
        world: &Scene,
        editor_workspace: Option<&ProjectEditorWorkspace>,
    ) -> Result<(), SceneProjectError> {
        let root = project.paths().root();
        // The scene is the F3 authoring authority. Persist the auxiliary workspace first so a
        // workspace I/O failure cannot report a failed save after the scene has already changed.
        let previous_workspace = capture_editor_workspace(root)?;
        save_editor_workspace(root, editor_workspace)?;
        if let Err(scene_error) =
            world.save_scene_to_project(project, &project.manifest().default_scene)
        {
            // A scene write can still fail after the workspace has committed. Restore the exact
            // previous workspace so a failed project save never leaves a split persisted document.
            restore_editor_workspace(root, previous_workspace).map_err(|restore_error| {
                SceneProjectError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "scene save failed: {scene_error}; editor workspace rollback failed: {restore_error}"
                    ),
                ))
            })?;
            return Err(scene_error);
        }
        Ok(())
    }
}
