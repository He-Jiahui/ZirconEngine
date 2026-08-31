use zircon_runtime::asset::{ProjectInfo, project::ProjectManager};
use zircon_runtime::scene::Scene;
use zircon_runtime::scene::world::SceneProjectError;

use crate::core::settings::SettingsAuthority;
#[cfg(test)]
use crate::core::settings::SettingsStore;

use super::editor_project_document::{EditorProjectDocument, ProjectSettingsLoadState};
use super::editor_workspace_persistence::load_editor_workspace_with_diagnostics;

impl EditorProjectDocument {
    pub(crate) fn load_from_activated_project(
        project: &ProjectManager,
        project_info: ProjectInfo,
        settings: &SettingsAuthority,
        allows_scene_restore: bool,
    ) -> Result<Self, SceneProjectError> {
        let project_settings = ProjectSettingsLoadState::from_authority_load(
            settings.load_project_layer_from_environment(project.paths().root()),
        );
        Self::assemble(
            project,
            project_info,
            project_settings,
            allows_scene_restore,
        )
    }

    fn assemble(
        project: &ProjectManager,
        project_info: ProjectInfo,
        project_settings: ProjectSettingsLoadState,
        allows_scene_restore: bool,
    ) -> Result<Self, SceneProjectError> {
        let root = project.paths().root().to_path_buf();
        let (editor_workspace, workspace_restore_diagnostics) =
            load_editor_workspace_with_diagnostics(&root);

        Ok(Self {
            root_path: root,
            manifest: project.manifest().clone(),
            project_info,
            project_settings,
            world: if allows_scene_restore {
                Scene::load_scene_from_uri(project, &project.manifest().default_scene)?
            } else {
                Scene::new()
            },
            editor_workspace,
            workspace_restore_diagnostics,
        })
    }

    #[cfg(test)]
    pub(crate) fn load_from_project_for_tests(
        project: &ProjectManager,
    ) -> Result<Self, SceneProjectError> {
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
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_document_load_uses_only_the_activated_settings_authority_in_production() {
        let document_source = include_str!("editor_project_document_load.rs").replace("\r\n", "\n");
        let host_source = include_str!("../../host/project_access.rs").replace("\r\n", "\n");

        let test_helper_gate = document_source
            .find("    #[cfg(test)]\n    pub(crate) fn load_from_project_for_tests(")
            .expect("test-only direct project loader");
        let production_source = &document_source[..test_helper_gate];
        let retired_loader_signature = ["fn load_from_", "project("].concat();
        assert!(!production_source.contains(&retired_loader_signature));
        assert!(!production_source.contains("SettingsAuthority::with_defaults()"));
        assert!(production_source.contains("pub(crate) fn load_from_activated_project("));
        assert!(production_source.contains("world: if allows_scene_restore"));
        assert!(production_source.contains("Scene::load_scene_from_uri"));
        assert!(production_source.contains("Scene::new()"));

        let test_helper_source = &document_source[test_helper_gate..];
        assert!(test_helper_source.contains("SettingsAuthority::with_defaults()"));
        assert!(host_source.contains(
            "EditorProjectDocument::load_from_activated_project(\n            &project,\n            project_info,\n            self.settings.as_ref(),\n            allows_scene_restore,\n        )?;"
        ));
    }
}
