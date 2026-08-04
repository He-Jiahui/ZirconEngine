use zircon_runtime::asset::{project::ProjectManager, ProjectInfo};
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::Scene;

use crate::core::settings::{SettingsAuthority, SettingsStore};

use super::editor_project_document::{EditorProjectDocument, ProjectSettingsLoadState};
use super::editor_workspace_persistence::load_editor_workspace_with_diagnostics;

impl EditorProjectDocument {
    pub fn load_from_project(project: &ProjectManager) -> Result<Self, SceneProjectError> {
        let root = project.paths().root();
        let authority = SettingsAuthority::with_defaults();
        let store = SettingsStore::from_roots(root, Some(root));
        let project_settings = ProjectSettingsLoadState::from_authority_load(
            authority.load_project_layer_from_store(&store),
        );
        Self::assemble(
            project,
            ProjectInfo::from_project(project),
            project_settings,
        )
    }

    pub(crate) fn load_from_activated_project(
        project: &ProjectManager,
        project_info: ProjectInfo,
        settings: &SettingsAuthority,
    ) -> Result<Self, SceneProjectError> {
        let project_settings = ProjectSettingsLoadState::from_authority_load(
            settings.load_project_layer_from_environment(project.paths().root()),
        );
        Self::assemble(project, project_info, project_settings)
    }

    fn assemble(
        project: &ProjectManager,
        project_info: ProjectInfo,
        project_settings: ProjectSettingsLoadState,
    ) -> Result<Self, SceneProjectError> {
        let root = project.paths().root().to_path_buf();
        let (editor_workspace, workspace_restore_diagnostics) =
            load_editor_workspace_with_diagnostics(&root);

        Ok(Self {
            root_path: root,
            manifest: project.manifest().clone(),
            project_info,
            project_settings,
            world: Scene::load_scene_from_uri(project, &project.manifest().default_scene)?,
            editor_workspace,
            workspace_restore_diagnostics,
        })
    }
}
