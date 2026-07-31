use zircon_runtime::asset::{project::ProjectManager, ProjectInfo};
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::Scene;

use crate::core::settings::{
    settings_registry_with_defaults, SettingsLoad, SettingsScope, SettingsStore,
};

use super::editor_project_document::{EditorProjectDocument, ProjectSettingsLoadState};
use super::editor_workspace_persistence::load_editor_workspace_with_diagnostics;

impl EditorProjectDocument {
    pub fn load_from_project(project: &ProjectManager) -> Result<Self, SceneProjectError> {
        Self::load_from_activated_project(project, ProjectInfo::from_project(project))
    }

    pub(crate) fn load_from_activated_project(
        project: &ProjectManager,
        project_info: ProjectInfo,
    ) -> Result<Self, SceneProjectError> {
        let root = project.paths().root().to_path_buf();
        let (editor_workspace, workspace_restore_diagnostics) =
            load_editor_workspace_with_diagnostics(&root);
        let project_settings = load_project_settings_state(&root);

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

fn load_project_settings_state(root: &std::path::Path) -> ProjectSettingsLoadState {
    let store = SettingsStore::from_roots(root, Some(root));
    let path = store
        .paths()
        .project()
        .expect("a project settings store always has a project path")
        .to_path_buf();
    let mut registry = settings_registry_with_defaults();
    match store.load_into(SettingsScope::Project, &mut registry) {
        Ok(SettingsLoad::Loaded {
            path,
            schema_version,
            ..
        }) => ProjectSettingsLoadState::Persisted {
            path,
            schema_version,
        },
        Ok(SettingsLoad::Missing { path }) => ProjectSettingsLoadState::Missing { path },
        Err(error) => ProjectSettingsLoadState::Invalid {
            path,
            message: error.to_string(),
        },
    }
}
