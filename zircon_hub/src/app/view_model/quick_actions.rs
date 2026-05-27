use slint::SharedString;

use crate::projects::project_paths_match;
use crate::projects::{metadata_for_path, RecentProject};
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
            project_build_target_detail(language, &project_target),
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
                    "Selected project is no longer available to install",
                    "选中项目已不可用，无法安装",
                ),
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
                    "Selected project is no longer available to package",
                    "选中项目已不可用，无法打包",
                ),
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
                (
                    "Selected project unavailable; launch editor without a project",
                    "选中项目不可用；不带项目启动编辑器",
                ),
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
    Selected {
        name: String,
        source_engine_state: ProjectSourceEngineState,
    },
    LatestRecent {
        name: String,
        source_engine_state: ProjectSourceEngineState,
    },
    StaleSelection,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectSourceEngineState {
    Ready,
    MissingBinding,
    Unavailable,
}

impl QuickActionProjectTarget {
    fn has_project(&self) -> bool {
        !matches!(self, Self::None | Self::StaleSelection)
    }

    fn has_source_engine(&self) -> bool {
        match self {
            Self::Selected {
                source_engine_state,
                ..
            }
            | Self::LatestRecent {
                source_engine_state,
                ..
            } => *source_engine_state == ProjectSourceEngineState::Ready,
            Self::None | Self::StaleSelection => false,
        }
    }
}

fn quick_action_enabled(action: HubQuickAction, target: &QuickActionProjectTarget) -> bool {
    match action {
        HubQuickAction::BuildProject => target.has_source_engine(),
        HubQuickAction::PackageProject | HubQuickAction::InstallToDevice => target.has_project(),
        HubQuickAction::OpenEditor => true,
    }
}

fn quick_action_project_target(snapshot: &HubSnapshot) -> QuickActionProjectTarget {
    if let Some(selected_path) = snapshot.selected_project_path.as_ref() {
        if let Some(project) = snapshot
            .recent_projects
            .iter()
            .find(|project| project_paths_match(&project.path, selected_path))
        {
            return QuickActionProjectTarget::Selected {
                name: project_display_name(project),
                source_engine_state: project_source_engine_state(project, snapshot),
            };
        }
        return QuickActionProjectTarget::StaleSelection;
    }

    snapshot
        .recent_projects
        .iter()
        .max_by_key(|project| project.last_opened_unix_ms)
        .map(|project| QuickActionProjectTarget::LatestRecent {
            name: project_display_name(project),
            source_engine_state: project_source_engine_state(project, snapshot),
        })
        .unwrap_or(QuickActionProjectTarget::None)
}

fn project_source_engine_state(
    project: &RecentProject,
    snapshot: &HubSnapshot,
) -> ProjectSourceEngineState {
    let Some(engine_id) = metadata_for_path(&snapshot.project_metadata, &project.path)
        .and_then(|metadata| metadata.engine_id.as_deref())
    else {
        return ProjectSourceEngineState::MissingBinding;
    };

    if snapshot.engines.iter().any(|engine| engine.id == engine_id) {
        ProjectSourceEngineState::Ready
    } else {
        ProjectSourceEngineState::Unavailable
    }
}

fn project_build_target_detail(
    language: HubLanguage,
    target: &QuickActionProjectTarget,
) -> SharedString {
    match target {
        QuickActionProjectTarget::Selected {
            name,
            source_engine_state: ProjectSourceEngineState::Ready,
        } => SharedString::from(format!(
            "{}: {}",
            localized_pair(
                language,
                (
                    "Build selected project's Source Engine",
                    "构建选中项目的 Source Engine"
                )
            ),
            name
        )),
        QuickActionProjectTarget::Selected {
            name,
            source_engine_state: ProjectSourceEngineState::Unavailable,
        } => SharedString::from(format!(
            "{}: {}",
            localized_pair(
                language,
                (
                    "Bound Source Engine is unavailable for selected project",
                    "选中项目绑定的 Source Engine 不可用"
                )
            ),
            name
        )),
        QuickActionProjectTarget::Selected { name, .. } => SharedString::from(format!(
            "{}: {}",
            localized_pair(
                language,
                (
                    "Bind a Source Engine before building selected project",
                    "请先为选中项目绑定 Source Engine 再构建"
                )
            ),
            name
        )),
        QuickActionProjectTarget::LatestRecent {
            name,
            source_engine_state: ProjectSourceEngineState::Ready,
        } => SharedString::from(format!(
            "{}: {}",
            localized_pair(
                language,
                (
                    "Build latest recent project's Source Engine",
                    "构建最近项目的 Source Engine"
                )
            ),
            name
        )),
        QuickActionProjectTarget::LatestRecent {
            name,
            source_engine_state: ProjectSourceEngineState::Unavailable,
        } => SharedString::from(format!(
            "{}: {}",
            localized_pair(
                language,
                (
                    "Bound Source Engine is unavailable for latest recent project",
                    "最近项目绑定的 Source Engine 不可用"
                )
            ),
            name
        )),
        QuickActionProjectTarget::LatestRecent { name, .. } => SharedString::from(format!(
            "{}: {}",
            localized_pair(
                language,
                (
                    "Bind a Source Engine before building latest recent project",
                    "请先为最近项目绑定 Source Engine 再构建"
                )
            ),
            name
        )),
        QuickActionProjectTarget::StaleSelection => SharedString::from(localized_pair(
            language,
            (
                "Selected project is no longer available to build",
                "选中项目已不可用，无法构建",
            ),
        )),
        QuickActionProjectTarget::None => SharedString::from(localized_pair(
            language,
            (
                "Select or add a project before building",
                "请先选择或添加项目再构建",
            ),
        )),
    }
}

fn project_target_detail(
    language: HubLanguage,
    target: &QuickActionProjectTarget,
    selected: (&str, &str),
    latest_recent: (&str, &str),
    stale: (&str, &str),
    none: (&str, &str),
) -> SharedString {
    match target {
        QuickActionProjectTarget::Selected { name, .. } => {
            SharedString::from(format!("{}: {}", localized_pair(language, selected), name))
        }
        QuickActionProjectTarget::LatestRecent { name, .. } => SharedString::from(format!(
            "{}: {}",
            localized_pair(language, latest_recent),
            name
        )),
        QuickActionProjectTarget::StaleSelection => {
            SharedString::from(localized_pair(language, stale))
        }
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

    use crate::engines::SourceEngineInstall;
    use crate::projects::{project_metadata_key, ProjectMetadata, RecentProject};
    use crate::settings::{HubLanguage, HubSettings};
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn build_action_uses_selected_project_target() {
        let selected = PathBuf::from("E:/Projects/Game");
        let mut snapshot = snapshot_with_projects(
            Some(PathBuf::from("E:\\Projects\\Game\\")),
            vec![RecentProject::new("Game", selected.clone(), 10)],
        );
        bind_source_engine(&mut snapshot, &selected);

        let actions = quick_actions(&snapshot, HubLanguage::English);
        let build = action(&actions, "build-project");

        assert!(build.enabled);
        assert_eq!(
            build.detail,
            SharedString::from("Build selected project's Source Engine: Game")
        );
    }

    #[test]
    fn build_action_disables_unbound_selected_project() {
        let selected = PathBuf::from("E:/Projects/Game");
        let snapshot = snapshot_with_projects(
            Some(PathBuf::from("E:\\Projects\\Game\\")),
            vec![RecentProject::new("Game", selected, 10)],
        );

        let actions = quick_actions(&snapshot, HubLanguage::English);

        assert!(!action(&actions, "build-project").enabled);
        assert!(action(&actions, "package-project").enabled);
        assert_eq!(
            action(&actions, "build-project").detail,
            SharedString::from("Bind a Source Engine before building selected project: Game")
        );
    }

    #[test]
    fn build_action_explains_unavailable_bound_source_engine() {
        let selected = PathBuf::from("E:/Projects/Game");
        let mut snapshot = snapshot_with_projects(
            Some(PathBuf::from("E:\\Projects\\Game\\")),
            vec![RecentProject::new("Game", selected.clone(), 10)],
        );
        bind_unavailable_source_engine(&mut snapshot, &selected);

        let actions = quick_actions(&snapshot, HubLanguage::English);

        assert!(!action(&actions, "build-project").enabled);
        assert!(action(&actions, "package-project").enabled);
        assert_eq!(
            action(&actions, "build-project").detail,
            SharedString::from("Bound Source Engine is unavailable for selected project: Game")
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

    #[test]
    fn quick_actions_do_not_fallback_when_selected_project_is_stale() {
        let selected = PathBuf::from("E:/Projects/Missing");
        let latest = PathBuf::from("E:/Projects/Other");
        let mut snapshot = snapshot_with_projects(
            Some(selected),
            vec![RecentProject::new("Other", latest.clone(), 20)],
        );
        bind_source_engine(&mut snapshot, &latest);

        let actions = quick_actions(&snapshot, HubLanguage::English);

        assert!(!action(&actions, "build-project").enabled);
        assert!(!action(&actions, "package-project").enabled);
        assert!(!action(&actions, "install-device").enabled);
        assert_eq!(
            action(&actions, "build-project").detail,
            SharedString::from("Selected project is no longer available to build")
        );
        assert_eq!(
            action(&actions, "open-editor").detail,
            SharedString::from("Selected project unavailable; launch editor without a project")
        );
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
            new_project_location: PathBuf::from("E:/Projects"),
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

    fn bind_source_engine(snapshot: &mut HubSnapshot, project_path: &PathBuf) {
        snapshot.engines.push(SourceEngineInstall {
            id: "source-local".to_string(),
            display_name: "Local Source".to_string(),
            source_dir: PathBuf::from("E:/Source/ZirconEngine"),
            output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        });
        snapshot.project_metadata.insert(
            project_metadata_key(project_path),
            ProjectMetadata {
                engine_id: Some("source-local".to_string()),
                ..ProjectMetadata::default()
            },
        );
    }

    fn bind_unavailable_source_engine(snapshot: &mut HubSnapshot, project_path: &PathBuf) {
        snapshot.project_metadata.insert(
            project_metadata_key(project_path),
            ProjectMetadata {
                engine_id: Some("missing-source".to_string()),
                ..ProjectMetadata::default()
            },
        );
    }

    fn action<'a>(actions: &'a [QuickActionData], id: &str) -> &'a QuickActionData {
        actions
            .iter()
            .find(|action| action.id == SharedString::from(id))
            .expect("quick action id must exist")
    }
}
