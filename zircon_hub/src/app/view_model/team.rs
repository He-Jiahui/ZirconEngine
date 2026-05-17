use slint::SharedString;

use crate::settings::HubLanguage;
use crate::state::HubSnapshot;
use crate::team::{TeamMemberEntry, TeamOverview};

use super::super::{TeamData, TeamMemberData};
use super::localization;

pub(super) fn team_summary(snapshot: &HubSnapshot) -> TeamData {
    team_data(&snapshot.team, snapshot.settings.language)
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

fn team_data(team: &TeamOverview, language: HubLanguage) -> TeamData {
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
        status: localization::text(language, "Local workspace contributors", "本地工作区贡献者"),
    }
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
    use crate::state::{HubPage, ProjectFilterMode, ProjectSortMode, ProjectViewMode, TaskStatus};

    use super::*;

    #[test]
    fn team_summary_falls_back_for_missing_git_identity() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Team,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
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
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, SharedString::from("Ada"));
        assert_eq!(members[0].commits, SharedString::from("3"));
    }
}
