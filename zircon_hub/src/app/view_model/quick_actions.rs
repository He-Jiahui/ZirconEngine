use slint::SharedString;

use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::quick_action::HubQuickAction;
use super::super::QuickActionData;
use super::{localization, media, project_display_name, shared};

pub(super) fn quick_actions(snapshot: &HubSnapshot, language: HubLanguage) -> Vec<QuickActionData> {
    let project_target = quick_action_project_target(snapshot);
    [
        (
            HubQuickAction::BuildProject,
            "/>",
            localization::text(language, "Build Project", "构建项目"),
            project_target_detail(
                language,
                &project_target,
                (
                    "Build selected project's Source Engine",
                    "构建选中项目的 Source Engine",
                ),
                (
                    "Build latest recent project's Source Engine",
                    "构建最近项目的 Source Engine",
                ),
                (
                    "Select or add a project before building",
                    "请先选择或添加项目再构建",
                ),
            ),
        ),
        (
            HubQuickAction::InstallToDevice,
            "[]",
            localization::text(language, "Install to Device", "安装到设备"),
            project_target_detail(
                language,
                &project_target,
                ("Install selected project package", "安装选中项目包"),
                ("Install latest recent project package", "安装最近项目包"),
                (
                    "Select or add a project before installing",
                    "请先选择或添加项目再安装",
                ),
            ),
        ),
        (
            HubQuickAction::PackageProject,
            "{}",
            localization::text(language, "Package Project", "打包项目"),
            project_target_detail(
                language,
                &project_target,
                ("Package selected project", "打包选中项目"),
                ("Package latest recent project", "打包最近项目"),
                (
                    "Select or add a project before packaging",
                    "请先选择或添加项目再打包",
                ),
            ),
        ),
        (
            HubQuickAction::OpenEditor,
            "<>",
            localization::text(language, "Open in Editor", "在编辑器中打开"),
            project_target_detail(
                language,
                &project_target,
                ("Open selected project", "打开选中项目"),
                ("Open latest recent project", "打开最近项目"),
                ("Launch the editor without a project", "不带项目启动编辑器"),
            ),
        ),
    ]
    .into_iter()
    .map(|(action, icon, title, detail)| {
        let id = action.id();
        let icon_image = media::quick_action_icon(id);
        QuickActionData {
            id: shared(id),
            icon: shared(icon),
            icon_image: icon_image.clone().unwrap_or_default(),
            has_icon_image: icon_image.is_some(),
            title,
            detail,
            enabled: quick_action_enabled(action, &project_target),
        }
    })
    .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum QuickActionProjectTarget {
    Selected(String),
    LatestRecent(String),
    None,
}

impl QuickActionProjectTarget {
    fn has_project(&self) -> bool {
        !matches!(self, Self::None)
    }
}

fn quick_action_enabled(action: HubQuickAction, target: &QuickActionProjectTarget) -> bool {
    match action {
        HubQuickAction::BuildProject
        | HubQuickAction::PackageProject
        | HubQuickAction::InstallToDevice => target.has_project(),
        HubQuickAction::OpenEditor => true,
    }
}

fn quick_action_project_target(snapshot: &HubSnapshot) -> QuickActionProjectTarget {
    if let Some(selected_path) = snapshot.selected_project_path.as_ref() {
        if let Some(project) = snapshot
            .recent_projects
            .iter()
            .find(|project| &project.path == selected_path)
        {
            return QuickActionProjectTarget::Selected(project_display_name(project));
        }
    }

    snapshot
        .recent_projects
        .iter()
        .max_by_key(|project| project.last_opened_unix_ms)
        .map(|project| QuickActionProjectTarget::LatestRecent(project_display_name(project)))
        .unwrap_or(QuickActionProjectTarget::None)
}

fn project_target_detail(
    language: HubLanguage,
    target: &QuickActionProjectTarget,
    selected: (&str, &str),
    latest_recent: (&str, &str),
    none: (&str, &str),
) -> SharedString {
    match target {
        QuickActionProjectTarget::Selected(name) => {
            SharedString::from(format!("{}: {}", localized_pair(language, selected), name))
        }
        QuickActionProjectTarget::LatestRecent(name) => SharedString::from(format!(
            "{}: {}",
            localized_pair(language, latest_recent),
            name
        )),
        QuickActionProjectTarget::None => SharedString::from(localized_pair(language, none)),
    }
}

fn localized_pair<'a>(language: HubLanguage, pair: (&'a str, &'a str)) -> &'a str {
    match language {
        HubLanguage::English => pair.0,
        HubLanguage::Chinese => pair.1,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::projects::RecentProject;
    use crate::settings::{HubLanguage, HubSettings};
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn build_action_uses_selected_project_target() {
        let selected = PathBuf::from("E:/Projects/Game");
        let snapshot = snapshot_with_projects(
            Some(selected.clone()),
            vec![RecentProject::new("Game", selected, 10)],
        );

        let actions = quick_actions(&snapshot, HubLanguage::English);
        let build = action(&actions, "build-project");

        assert!(build.enabled);
        assert_eq!(
            build.detail,
            SharedString::from("Build selected project's Source Engine: Game")
        );
    }

    #[test]
    fn project_only_actions_disable_without_a_project() {
        let snapshot = snapshot_with_projects(None, Vec::new());

        let actions = quick_actions(&snapshot, HubLanguage::English);

        assert!(!action(&actions, "build-project").enabled);
        assert!(!action(&actions, "package-project").enabled);
        assert!(!action(&actions, "install-device").enabled);
        assert!(action(&actions, "open-editor").enabled);
    }

    fn snapshot_with_projects(
        selected_project_path: Option<PathBuf>,
        recent_projects: Vec<RecentProject>,
    ) -> HubSnapshot {
        HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path,
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects,
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        }
    }

    fn action<'a>(actions: &'a [QuickActionData], id: &str) -> &'a QuickActionData {
        actions
            .iter()
            .find(|action| action.id == SharedString::from(id))
            .expect("quick action id must exist")
    }
}
