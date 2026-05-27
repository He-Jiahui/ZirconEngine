use std::path::Path;

use slint::SharedString;

use crate::projects::project_metadata_key;
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;
use crate::team::{TeamMemberEntry, TeamOverview};

use super::super::{TeamData, TeamMemberData};
use super::localization;

pub(super) fn team_summary(snapshot: &HubSnapshot) -> TeamData {
    team_data(
        &snapshot.team,
        snapshot.settings.language,
        snapshot.selected_project_path.as_deref(),
    )
}

pub(super) fn team_members(snapshot: &HubSnapshot) -> Vec<TeamMemberData> {
    snapshot
        .team
        .members
        .iter()
        .enumerate()
        .map(|(index, member)| team_member_data(index, member))
        .collect()
}

fn team_data(
    team: &TeamOverview,
    language: HubLanguage,
    selected_project_path: Option<&Path>,
) -> TeamData {
    TeamData {
        repository_path: shared(if team.repository_path.as_os_str().is_empty() {
            localization::text(language, "No local Git workspace", "未发现本地 Git 工作区")
                .to_string()
        } else {
            team.repository_path.to_string_lossy().into_owned()
        }),
        identity_name: shared(if team.identity_name.trim().is_empty() {
            localization::text(language, "Not configured", "未配置").to_string()
        } else {
            team.identity_name.clone()
        }),
        identity_email: shared(if team.identity_email.trim().is_empty() {
            localization::text(language, "Not configured", "未配置").to_string()
        } else {
            team.identity_email.clone()
        }),
        status: shared(team_status(team, language, selected_project_path)),
    }
}

fn team_status(
    team: &TeamOverview,
    language: HubLanguage,
    selected_project_path: Option<&Path>,
) -> String {
    if team.repository_path.as_os_str().is_empty() {
        return localization::text(language, "No local Git workspace", "未发现本地 Git 工作区")
            .to_string();
    }
    if let Some(project_path) = selected_project_path {
        if paths_share_repository_scope(project_path, &team.repository_path) {
            return localization::text(language, "Selected project repository", "选中项目仓库")
                .to_string();
        }
        return localization::text(
            language,
            "Selected project repository unavailable; showing Source Engine repository",
            "未找到选中项目仓库；显示 Source Engine 仓库",
        )
        .to_string();
    }
    localization::text(language, "Source Engine repository", "Source Engine 仓库").to_string()
}

fn paths_share_repository_scope(project_path: &Path, repository_path: &Path) -> bool {
    if project_path.starts_with(repository_path) || repository_path.starts_with(project_path) {
        return true;
    }

    if let (Ok(project_path), Ok(repository_path)) =
        (project_path.canonicalize(), repository_path.canonicalize())
    {
        return project_path.starts_with(&repository_path)
            || repository_path.starts_with(&project_path);
    }

    let project_key = project_metadata_key(project_path);
    let repository_key = project_metadata_key(repository_path);
    project_key == repository_key
        || project_key.starts_with(&format!("{repository_key}/"))
        || repository_key.starts_with(&format!("{project_key}/"))
}

fn team_member_data(index: usize, member: &TeamMemberEntry) -> TeamMemberData {
    TeamMemberData {
        name: shared(if member.name.trim().is_empty() {
            "Unknown"
        } else {
            member.name.as_str()
        }),
        email: shared(member.email.clone()),
        commits: shared(member.commits.to_string()),
        accent: index as i32,
    }
}

fn shared(value: impl Into<SharedString>) -> SharedString {
    value.into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::settings::HubSettings;
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn team_summary_falls_back_for_missing_git_identity() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Team,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview {
                repository_path: PathBuf::from("E:/repo"),
                identity_name: String::new(),
                identity_email: String::new(),
                members: vec![TeamMemberEntry {
                    name: "Ada".to_string(),
                    email: "ada@example.com".to_string(),
                    commits: 3,
                }],
            },
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let summary = team_summary(&snapshot);
        let members = team_members(&snapshot);

        assert_eq!(summary.identity_name, SharedString::from("Not configured"));
        assert_eq!(
            summary.status,
            SharedString::from("Source Engine repository")
        );
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, SharedString::from("Ada"));
        assert_eq!(members[0].commits, SharedString::from("3"));
    }

    #[test]
    fn team_summary_labels_selected_project_repository() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Team,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/repo/projects/demo")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview {
                repository_path: PathBuf::from("E:/repo"),
                identity_name: "Ada".to_string(),
                identity_email: "ada@example.com".to_string(),
                members: Vec::new(),
            },
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let summary = team_summary(&snapshot);

        assert_eq!(
            summary.status,
            SharedString::from("Selected project repository")
        );
    }

    #[test]
    fn team_summary_labels_selected_project_repository_with_normalized_paths() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Team,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:\\Repo\\Projects\\Demo\\")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview {
                repository_path: PathBuf::from("e:/repo"),
                identity_name: "Ada".to_string(),
                identity_email: "ada@example.com".to_string(),
                members: Vec::new(),
            },
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let summary = team_summary(&snapshot);

        assert_eq!(
            summary.status,
            SharedString::from("Selected project repository")
        );
    }

    #[test]
    fn team_summary_labels_source_engine_fallback_for_missing_selected_project_repository() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Team,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/missing/projects/demo")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview {
                repository_path: PathBuf::from("E:/source-engine"),
                identity_name: "Ada".to_string(),
                identity_email: "ada@example.com".to_string(),
                members: Vec::new(),
            },
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let summary = team_summary(&snapshot);

        assert_eq!(
            summary.status,
            SharedString::from(
                "Selected project repository unavailable; showing Source Engine repository"
            )
        );
    }
}
