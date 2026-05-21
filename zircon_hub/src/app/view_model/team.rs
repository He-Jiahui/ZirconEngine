use std::path::Path;

use slint::SharedString;

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
    if selected_project_path.is_some_and(|project_path| {
        project_path.starts_with(&team.repository_path)
            || team.repository_path.starts_with(project_path)
    }) {
        return localization::text(language, "Selected project repository", "选中项目仓库")
            .to_string();
    }
    localization::text(language, "Source engine repository", "Source Engine 仓库").to_string()
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
            SharedString::from("Source engine repository")
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
}
