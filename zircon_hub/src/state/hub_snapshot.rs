use crate::engines::SourceEngineInstall;
use crate::projects::RecentProject;
use crate::settings::HubSettings;

use super::{HubPage, ProjectSortMode, ProjectViewMode, TaskStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubSnapshot {
    pub selected_page: HubPage,
    pub project_sort: ProjectSortMode,
    pub project_view_mode: ProjectViewMode,
    pub search_query: String,
    pub task_status: TaskStatus,
    pub recent_projects: Vec<RecentProject>,
    pub engines: Vec<SourceEngineInstall>,
    pub settings: HubSettings,
}

impl HubSnapshot {
    pub fn filtered_recent_projects(&self) -> Vec<RecentProject> {
        let query = self.search_query.trim().to_ascii_lowercase();
        let mut projects: Vec<_> = self
            .recent_projects
            .iter()
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
    use crate::state::{HubPage, ProjectSortMode, ProjectViewMode, TaskStatus};

    use super::*;

    #[test]
    fn filtered_recent_projects_sorts_by_selected_mode() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_sort: ProjectSortMode::Name,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Zeta", "E:/Projects/Zeta", 30),
                RecentProject::new("Alpha", "E:/Projects/Alpha", 10),
            ],
            engines: Vec::new(),
            settings: HubSettings::default(),
        };

        let projects = snapshot.filtered_recent_projects();

        assert_eq!(projects[0].display_name, "Alpha");
        assert_eq!(projects[1].display_name, "Zeta");
    }
}
