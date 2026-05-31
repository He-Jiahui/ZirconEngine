use crate::error::HubError;
use crate::state::HubScope;

use super::{root_paths::push_development_roots, HubRuntime};

impl HubRuntime {
    pub(super) fn refresh_source_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }

    pub(super) fn selected_project_catalog_root(&self) -> Option<std::path::PathBuf> {
        self.current_scope()
            .selected_project()
            .map(|project| project.path.clone())
    }

    pub(super) fn source_engine_catalog_roots(&self) -> Vec<std::path::PathBuf> {
        let mut roots = Vec::new();
        let scope = self.current_scope();
        let Some(engine_id) = scope.source_engine.engine_id() else {
            return roots;
        };
        if let Some(engine) = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
        {
            push_development_roots(&mut roots, engine.source_dir.clone());
        }
        roots
    }

    fn current_scope(&self) -> HubScope {
        HubScope::resolve(
            self.selected_project_path.as_deref(),
            &self.config.recent_projects,
            &self.config.project_metadata,
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::engines::SourceEngineInstall;
    use crate::projects::{metadata_for_path_mut, ProjectTemplate, RecentProject};
    use crate::settings::HubConfig;
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };
    use crate::team::TeamOverview;

    use super::*;

    fn runtime_with_projects(projects: Vec<RecentProject>) -> HubRuntime {
        HubRuntime {
            config_path: PathBuf::from("hub.toml"),
            editor_config_path: PathBuf::from("editor.json"),
            config: HubConfig {
                recent_projects: projects,
                ..HubConfig::default()
            },
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: ProjectTemplate::RenderableEmpty.id().to_string(),
            new_project_location: HubConfig::default().settings.default_project_dir,
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        }
    }

    fn engine(id: &str, source_dir: &str) -> SourceEngineInstall {
        SourceEngineInstall {
            id: id.to_string(),
            display_name: format!("{id} Engine"),
            source_dir: PathBuf::from(source_dir),
            output_dir: PathBuf::from(format!("E:/out/{id}")),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        }
    }

    #[test]
    fn selected_project_catalog_root_ignores_stale_selection() {
        let mut runtime =
            runtime_with_projects(vec![RecentProject::new("Game", "E:/Projects/Game", 20)]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Missing"));

        assert!(runtime.selected_project_catalog_root().is_none());
    }

    #[test]
    fn source_engine_catalog_roots_prefer_project_bound_engine() {
        let mut runtime =
            runtime_with_projects(vec![RecentProject::new("Game", "E:/Projects/Game", 20)]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Game"));
        runtime.config.engines = vec![
            engine("active", "E:/Engines/Active"),
            engine("project", "E:/Engines/Project"),
        ];
        runtime.config.active_engine_id = Some("active".to_string());
        metadata_for_path_mut(&mut runtime.config.project_metadata, "E:/Projects/Game").engine_id =
            Some("project".to_string());

        let roots = runtime.source_engine_catalog_roots();

        assert_eq!(roots.first(), Some(&PathBuf::from("E:/Engines/Project")));
        assert!(!roots
            .iter()
            .any(|root| root == &PathBuf::from("E:/Engines/Active")));
    }

    #[test]
    fn source_engine_catalog_roots_do_not_fallback_for_unbound_selected_project() {
        let mut runtime =
            runtime_with_projects(vec![RecentProject::new("Game", "E:/Projects/Game", 20)]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Game"));
        runtime.config.engines = vec![engine("active", "E:/Engines/Active")];
        runtime.config.active_engine_id = Some("active".to_string());

        assert!(runtime.source_engine_catalog_roots().is_empty());
    }

    #[test]
    fn source_engine_catalog_roots_use_active_engine_without_selected_project() {
        let mut runtime = runtime_with_projects(Vec::new());
        runtime.config.engines = vec![
            engine("first", "E:/Engines/First"),
            engine("active", "E:/Engines/Active"),
        ];
        runtime.config.active_engine_id = Some("active".to_string());

        let roots = runtime.source_engine_catalog_roots();

        assert_eq!(roots.first(), Some(&PathBuf::from("E:/Engines/Active")));
    }
}
