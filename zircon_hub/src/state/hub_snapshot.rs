use crate::assets::AssetCatalogEntry;
use crate::engines::SourceEngineInstall;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::projects::{ProjectMetadataMap, RecentProject};
use crate::settings::HubSettings;
use crate::team::TeamOverview;
use std::path::PathBuf;

use super::{
    HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubSnapshot {
    pub selected_page: HubPage,
    pub project_filter: ProjectFilterMode,
    pub project_sort: ProjectSortMode,
    pub project_view_mode: ProjectViewMode,
    pub project_subpage: ProjectSubpage,
    pub search_query: String,
    pub selected_project_path: Option<PathBuf>,
    pub selected_template_id: String,
    pub new_project_engine_id: Option<String>,
    pub pending_delete_project_path: Option<PathBuf>,
    pub task_status: TaskStatus,
    pub recent_projects: Vec<RecentProject>,
    pub project_metadata: ProjectMetadataMap,
    pub assets: Vec<AssetCatalogEntry>,
    pub learn_resources: Vec<LearnCatalogEntry>,
    pub plugins: Vec<PluginCatalogEntry>,
    pub team: TeamOverview,
    pub engines: Vec<SourceEngineInstall>,
    pub active_engine_id: Option<String>,
    pub settings: HubSettings,
}

impl HubSnapshot {
    pub fn filtered_recent_projects(&self) -> Vec<RecentProject> {
        let query = self.search_query.trim().to_ascii_lowercase();
        let mut projects: Vec<_> = self
            .recent_projects
            .iter()
            .filter(|project| self.project_filter.includes(project))
            .filter(|project| query.is_empty() || project_matches_query(project, &query))
            .cloned()
            .collect();

        match self.project_sort {
            ProjectSortMode::LastModified => projects
                .sort_by(|left, right| right.last_opened_unix_ms.cmp(&left.last_opened_unix_ms)),
            ProjectSortMode::Name => {
                projects.sort_by_key(|project| project_display_name(project).to_ascii_lowercase());
            }
        }

        projects
    }
}

impl ProjectFilterMode {
    fn includes(self, project: &RecentProject) -> bool {
        match self {
            Self::All => true,
            Self::Existing => project.path.exists(),
            Self::Missing => !project.path.exists(),
        }
    }
}

fn project_matches_query(project: &RecentProject, query: &str) -> bool {
    project.display_name.to_ascii_lowercase().contains(query)
        || project
            .path
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains(query)
}

fn project_display_name(project: &RecentProject) -> String {
    if project.display_name.trim().is_empty() {
        return project
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Zircon Project")
            .to_string();
    }
    project.display_name.clone()
}

#[cfg(test)]
mod tests {
    use crate::settings::HubSettings;
    use std::fs;

    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn filtered_recent_projects_sorts_by_selected_mode() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::Name,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Zeta", "E:/Projects/Zeta", 30),
                RecentProject::new("Alpha", "E:/Projects/Alpha", 10),
            ],
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let projects = snapshot.filtered_recent_projects();

        assert_eq!(projects[0].display_name, "Alpha");
        assert_eq!(projects[1].display_name, "Zeta");
    }

    #[test]
    fn filtered_recent_projects_applies_path_filter_before_sorting() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-filter-test-{}",
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let existing = root.join("Existing");
        let missing = root.join("Missing");
        fs::create_dir_all(&existing).unwrap();
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::Existing,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Missing", missing.clone(), 30),
                RecentProject::new("Existing", existing.clone(), 10),
            ],
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let projects = snapshot.filtered_recent_projects();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].display_name, "Existing");
    }
}
