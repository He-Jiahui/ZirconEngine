use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::projects::{
    load_editor_recent_project_session, merge_recent_projects, project_paths_match,
    save_editor_recent_projects, save_editor_recent_projects_with_last_project, ProjectTemplate,
};
use crate::settings::{default_hub_config_path, editor_config_path, HubConfig, HubRuntimeState};
use crate::state::{
    push_action_record, HubActionRecord, HubPage, ProjectFilterMode, ProjectSortMode,
    ProjectSubpage, ProjectViewMode, TaskStatus,
};
use crate::team::TeamOverview;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn load() -> Result<Self, HubError> {
        let config_path = default_hub_config_path();
        let editor_config_path = editor_config_path();
        Self::load_from_paths(config_path, editor_config_path)
    }

    pub(super) fn load_from_config_path(config_path: PathBuf) -> Result<Self, HubError> {
        let editor_config_path = editor_config_path();
        Self::load_from_paths(config_path, editor_config_path)
    }

    pub(super) fn load_from_paths(
        config_path: PathBuf,
        editor_config_path: PathBuf,
    ) -> Result<Self, HubError> {
        let mut config = HubConfig::load(&config_path)?;
        let editor_recent = load_editor_recent_project_session(&editor_config_path)?;
        let last_project_path = editor_recent.last_project_path;
        config.recent_projects =
            merge_recent_projects(config.recent_projects, editor_recent.recent_projects);
        config.repair_registries();
        let runtime_state = config.runtime.clone();
        let selected_project_path = startup_selected_project_path(
            runtime_state.selected_project_path.as_deref(),
            last_project_path.as_deref(),
            &config.recent_projects,
        );
        let mut runtime = Self {
            config_path,
            editor_config_path,
            config,
            selected_page: runtime_state.selected_page,
            project_filter: runtime_state.project_filter,
            project_sort: runtime_state.project_sort,
            project_view_mode: runtime_state.project_view_mode,
            project_subpage: runtime_state.project_subpage,
            search_query: runtime_state.search_query,
            selected_project_path,
            selected_template_id: runtime_state.selected_template_id,
            new_project_location: runtime_state.new_project_location,
            new_project_engine_id: runtime_state.new_project_engine_id,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        };
        runtime.register_source_engine_from_settings();
        runtime.prune_stale_project_engine_bindings();
        runtime.config.repair_registries();
        if let Some(path) = runtime.selected_project_path.clone() {
            runtime.activate_project_engine_for_path(&path);
        }
        runtime.ensure_new_project_engine_selection();
        runtime.refresh_source_scoped_views()?;
        runtime.persist()?;
        Ok(runtime)
    }

    pub(super) fn empty_for_error(config_path: PathBuf) -> Self {
        let config = HubConfig::default();
        Self {
            config_path,
            editor_config_path: editor_config_path(),
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: ProjectTemplate::RenderableEmpty.id().to_string(),
            new_project_location: config.settings.default_project_dir.clone(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
            config,
        }
    }

    pub(super) fn persist(&self) -> Result<(), HubError> {
        self.persist_with_last_project(None)
    }

    pub(super) fn persist_hub_config(&self) -> Result<(), HubError> {
        let mut config = self.config.clone();
        config.runtime = self.runtime_state_for_config();
        config.save(&self.config_path)?;
        Ok(())
    }

    pub(super) fn persist_with_last_project(
        &self,
        last_project_path: Option<&Path>,
    ) -> Result<(), HubError> {
        self.persist_hub_config()?;
        match last_project_path {
            Some(path) => save_editor_recent_projects_with_last_project(
                &self.editor_config_path,
                &self.config.recent_projects,
                Some(path),
            )?,
            None => {
                save_editor_recent_projects(&self.editor_config_path, &self.config.recent_projects)?
            }
        }
        Ok(())
    }

    pub(super) fn record_action(&mut self, record: HubActionRecord) {
        push_action_record(&mut self.config.action_history, record);
    }

    pub(super) fn record_action_and_persist(
        &mut self,
        record: HubActionRecord,
    ) -> Result<(), HubError> {
        self.record_action(record);
        self.persist_hub_config()
    }

    fn runtime_state_for_config(&self) -> HubRuntimeState {
        HubRuntimeState {
            selected_page: self.selected_page,
            project_subpage: self.project_subpage,
            project_filter: self.project_filter,
            project_sort: self.project_sort,
            project_view_mode: self.project_view_mode,
            search_query: self.search_query.clone(),
            selected_project_path: self.selected_project_path.clone(),
            selected_template_id: self.selected_template_id.clone(),
            new_project_location: self.new_project_location.clone(),
            new_project_engine_id: self.new_project_engine_id.clone(),
        }
    }
}

fn startup_selected_project_path(
    persisted_selected_project_path: Option<&Path>,
    last_project_path: Option<&Path>,
    recent_projects: &[crate::projects::RecentProject],
) -> Option<PathBuf> {
    if let Some(path) = persisted_selected_project_path {
        return Some(
            recent_projects
                .iter()
                .find(|project| project_paths_match(&project.path, path))
                .map(|project| project.path.clone())
                .unwrap_or_else(|| path.to_path_buf()),
        );
    }

    let last_project_path = last_project_path?;
    recent_projects
        .iter()
        .find(|project| project_paths_match(&project.path, last_project_path))
        .map(|project| project.path.clone())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::projects::{project_metadata_key, RecentProject};

    use super::*;

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn startup_selection_restores_editor_last_project_when_it_matches_recent_projects() {
        let recent_projects = vec![
            RecentProject::new("Elysium", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ];

        let selected = startup_selected_project_path(
            None,
            Some(Path::new("E:\\Projects\\StellarOutpost\\")),
            &recent_projects,
        );

        assert_eq!(selected, Some(PathBuf::from("E:/Projects/StellarOutpost")));
        assert!(startup_selected_project_path(
            None,
            Some(Path::new("E:/Projects/Missing")),
            &recent_projects,
        )
        .is_none());
    }

    #[test]
    fn startup_selection_preserves_persisted_stale_project_path() {
        let recent_projects = vec![RecentProject::new("Recent", "E:/Projects/Recent", 30)];

        let selected = startup_selected_project_path(
            Some(Path::new("E:/Projects/Missing")),
            Some(Path::new("E:/Projects/Recent")),
            &recent_projects,
        );

        assert_eq!(selected, Some(PathBuf::from("E:/Projects/Missing")));
    }

    #[test]
    fn startup_selection_canonicalizes_persisted_recent_project_path() {
        let recent_projects = vec![RecentProject::new("Game", "E:/Projects/Game", 30)];

        let selected = startup_selected_project_path(
            Some(Path::new("E:\\Projects\\Game\\")),
            None,
            &recent_projects,
        );

        assert_eq!(selected, Some(PathBuf::from("E:/Projects/Game")));
    }

    #[test]
    fn load_from_paths_merges_repairs_and_persists_runtime_state_deterministically() {
        let temp = temp_test_dir("zircon-hub-runtime-load-persistence");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let project_path = temp.join("Game");
        fs::create_dir_all(&project_path).unwrap();

        let mut config = HubConfig::default();
        config.recent_projects = vec![
            RecentProject::new("Old Game", &project_path, 1),
            RecentProject::new("Game", &project_path, 4),
        ];
        config.project_metadata.insert(
            project_metadata_key(&project_path),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("missing-engine".to_string()),
                last_selected_template: Some("renderable-empty".to_string()),
            },
        );
        config.project_metadata.insert(
            project_metadata_key(temp.join("Removed")),
            crate::projects::ProjectMetadata {
                pinned: true,
                ..crate::projects::ProjectMetadata::default()
            },
        );
        config.settings.default_source_dir = temp.join("ZirconEngine");
        config.settings.default_build_output_dir = temp.join("out");
        config.active_engine_id = Some("missing-engine".to_string());
        config.runtime.selected_page = HubPage::Builds;
        config.runtime.project_subpage = ProjectSubpage::ProjectDetail;
        config.runtime.project_filter = ProjectFilterMode::Existing;
        config.runtime.project_sort = ProjectSortMode::Name;
        config.runtime.project_view_mode = ProjectViewMode::List;
        config.runtime.search_query = "game".to_string();
        config.runtime.selected_project_path = Some(project_path.clone());
        config.runtime.selected_template_id = ProjectTemplate::RenderableEmpty.id().to_string();
        config.runtime.new_project_location = temp.join("Drafts");
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            format!(
                r#"{{"editor.startup.session":{{"last_project_path":"{}","recent_projects":[{{"display_name":"Editor Game","path":"{}","last_opened_unix_ms":9}}]}}}}"#,
                project_path.to_string_lossy().replace('\\', "/"),
                project_path.to_string_lossy().replace('\\', "/")
            ),
        )
        .unwrap();

        let runtime = HubRuntime::load_from_paths(config_path.clone(), editor_config_path.clone())
            .expect("runtime should load and persist repaired registries");

        assert_eq!(runtime.selected_project_path, Some(project_path.clone()));
        assert_eq!(runtime.selected_page, HubPage::Builds);
        assert_eq!(runtime.project_subpage, ProjectSubpage::ProjectDetail);
        assert_eq!(runtime.project_filter, ProjectFilterMode::Existing);
        assert_eq!(runtime.project_sort, ProjectSortMode::Name);
        assert_eq!(runtime.project_view_mode, ProjectViewMode::List);
        assert_eq!(runtime.search_query, "game");
        assert_eq!(runtime.new_project_location, temp.join("Drafts"));
        assert_eq!(runtime.config.recent_projects.len(), 1);
        assert_eq!(
            runtime.config.recent_projects[0].display_name,
            "Editor Game"
        );
        let metadata = runtime
            .config
            .project_metadata
            .get(&project_metadata_key(&project_path))
            .expect("owned project metadata should survive load repair");
        assert!(metadata.pinned);
        assert!(metadata.engine_id.is_none());
        assert!(runtime.config.project_metadata.len() == 1);

        let saved = HubConfig::load(&config_path).unwrap();
        assert_eq!(saved.recent_projects.len(), 1);
        assert_eq!(saved.runtime.selected_page, HubPage::Builds);
        assert_eq!(saved.runtime.project_subpage, ProjectSubpage::ProjectDetail);
        assert_eq!(
            saved.runtime.selected_project_path,
            Some(project_path.clone())
        );
        assert!(saved.project_metadata[&project_metadata_key(&project_path)]
            .engine_id
            .is_none());
        let editor_session = load_editor_recent_project_session(&editor_config_path).unwrap();
        assert_eq!(editor_session.last_project_path, Some(project_path));
        assert_eq!(editor_session.recent_projects.len(), 1);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn load_from_paths_restores_persisted_stale_project_without_latest_recent_fallback() {
        let temp = temp_test_dir("zircon-hub-runtime-load-persisted-stale-project");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let recent_path = temp.join("Recent");
        let missing_path = temp.join("Missing");

        let mut config = HubConfig::default();
        config.recent_projects = vec![RecentProject::new("Recent", &recent_path, 10)];
        config.runtime.selected_project_path = Some(missing_path.clone());
        config.runtime.project_subpage = ProjectSubpage::ProjectDetail;
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            format!(
                r#"{{"editor.startup.session":{{"last_project_path":"{}","recent_projects":[]}}}}"#,
                recent_path.to_string_lossy().replace('\\', "/")
            ),
        )
        .unwrap();

        let runtime = HubRuntime::load_from_paths(config_path.clone(), editor_config_path).unwrap();

        assert_eq!(runtime.selected_project_path, Some(missing_path.clone()));
        assert_eq!(runtime.project_subpage, ProjectSubpage::ProjectDetail);
        let saved = HubConfig::load(&config_path).unwrap();
        assert_eq!(saved.runtime.selected_project_path, Some(missing_path));
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn load_from_paths_rejects_stale_editor_last_project_without_latest_recent_fallback() {
        let temp = temp_test_dir("zircon-hub-runtime-load-stale-last-project");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let recent_path = temp.join("Recent");
        let missing_path = temp.join("Missing");

        let mut config = HubConfig::default();
        config.recent_projects = vec![RecentProject::new("Recent", &recent_path, 10)];
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            format!(
                r#"{{"editor.startup.session":{{"last_project_path":"{}","recent_projects":[]}}}}"#,
                missing_path.to_string_lossy().replace('\\', "/")
            ),
        )
        .unwrap();

        let runtime = HubRuntime::load_from_paths(config_path, editor_config_path).unwrap();

        assert!(runtime.selected_project_path.is_none());
        assert_eq!(runtime.config.recent_projects.len(), 1);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn persist_clears_editor_last_project_when_project_leaves_hub_recents() {
        let temp = temp_test_dir("zircon-hub-runtime-clear-editor-last-project");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let project_path = temp.join("Removed");

        let mut config = HubConfig::default();
        config.recent_projects = vec![RecentProject::new("Removed", &project_path, 10)];
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            format!(
                r#"{{"editor.startup.session":{{"last_project_path":"{}","recent_projects":[{{"display_name":"Removed","path":"{}","last_opened_unix_ms":10}}]}}}}"#,
                project_path.to_string_lossy().replace('\\', "/"),
                project_path.to_string_lossy().replace('\\', "/")
            ),
        )
        .unwrap();

        let mut runtime = HubRuntime::load_from_paths(config_path, editor_config_path.clone())
            .expect("runtime should load with editor last project selected");
        runtime.remove_project_from_hub_path(&project_path);
        runtime.persist().unwrap();

        let editor_session = load_editor_recent_project_session(&editor_config_path).unwrap();
        assert!(editor_session.last_project_path.is_none());
        assert!(editor_session.recent_projects.is_empty());
        fs::remove_dir_all(temp).unwrap();
    }
}
