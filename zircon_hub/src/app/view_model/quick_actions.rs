use slint::SharedString;

use crate::settings::HubLanguage;
use crate::state::{HubSnapshot, ProjectEngineScopeState, ProjectScope};

use super::super::quick_action::HubQuickAction;
use super::super::QuickActionData;
use super::{localization, media, shared};

pub(super) fn quick_actions(snapshot: &HubSnapshot, language: HubLanguage) -> Vec<QuickActionData> {
    let project_target = quick_action_project_target(snapshot);
    [
        (
            HubQuickAction::BuildProject,
            "/>",
            localization::text(language, "Build Project", "构建项目"),
        ),
        (
            HubQuickAction::InstallToDevice,
            "[]",
            localization::text(language, "Install to Device", "安装到设备"),
        ),
        (
            HubQuickAction::PackageProject,
            "{}",
            localization::text(language, "Package Project", "打包项目"),
        ),
        (
            HubQuickAction::OpenEditor,
            "<>",
            localization::text(language, "Open in Editor", "在编辑器中打开"),
        ),
    ]
    .into_iter()
    .map(|(action, icon, title)| {
        let id = action.id();
        let icon_image = media::quick_action_icon(id);
        QuickActionData {
            id: shared(id),
            icon: shared(icon),
            icon_image: icon_image.clone().unwrap_or_default(),
            has_icon_image: icon_image.is_some(),
            title,
            detail: quick_action_detail(action, &project_target, language),
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

fn quick_action_detail(
    action: HubQuickAction,
    target: &QuickActionProjectTarget,
    language: HubLanguage,
) -> SharedString {
    let text = match (action, target) {
        (
            HubQuickAction::BuildProject,
            QuickActionProjectTarget::Selected {
                name,
                source_engine_state,
            },
        ) => build_detail_for_project(name, *source_engine_state, language),
        (
            HubQuickAction::BuildProject,
            QuickActionProjectTarget::LatestRecent {
                name,
                source_engine_state,
            },
        ) => match source_engine_state {
            ProjectSourceEngineState::Ready => format_pair(
                language,
                format!("Build latest recent project {name}"),
                format!("构建最近项目 {name}"),
            ),
            ProjectSourceEngineState::MissingBinding => format_pair(
                language,
                format!("Bind a Source Engine to latest recent project {name} before building"),
                format!("先为最近项目 {name} 绑定源码引擎"),
            ),
            ProjectSourceEngineState::Unavailable => format_pair(
                language,
                format!("Bound Source Engine for latest recent project {name} is unavailable"),
                format!("最近项目 {name} 绑定的源码引擎不可用"),
            ),
        },
        (HubQuickAction::BuildProject, QuickActionProjectTarget::StaleSelection) => {
            quick_action_reference_detail(
                language,
                "Selected project is no longer available",
                "已选项目不再可用",
            )
            .to_string()
        }
        (HubQuickAction::BuildProject, QuickActionProjectTarget::None) => localized_pair(
            language,
            (
                "Select a project with a bound Source Engine before building",
                "先选择已绑定源码引擎的项目",
            ),
        )
        .to_string(),
        (HubQuickAction::PackageProject, target) => project_action_detail(
            target,
            language,
            "Package selected project",
            "打包已选项目",
            "Package latest recent project",
            "打包最近项目",
            "Select a project before packaging",
            "先选择项目再打包",
        ),
        (HubQuickAction::InstallToDevice, target) => project_action_detail(
            target,
            language,
            "Install selected project",
            "安装已选项目",
            "Install latest recent project",
            "安装最近项目",
            "Select a project before installing",
            "先选择项目再安装",
        ),
        (HubQuickAction::OpenEditor, QuickActionProjectTarget::Selected { name, .. }) => {
            format_pair(
                language,
                format!("Open {name} in Editor"),
                format!("在编辑器中打开 {name}"),
            )
        }
        (HubQuickAction::OpenEditor, QuickActionProjectTarget::LatestRecent { name, .. }) => {
            format_pair(
                language,
                format!("Open latest recent project {name} in Editor"),
                format!("在编辑器中打开最近项目 {name}"),
            )
        }
        (HubQuickAction::OpenEditor, QuickActionProjectTarget::StaleSelection)
        | (HubQuickAction::OpenEditor, QuickActionProjectTarget::None) => {
            quick_action_reference_detail(
                language,
                "Open Editor without a project",
                "不带项目打开编辑器",
            )
            .to_string()
        }
    };
    shared(text)
}

fn build_detail_for_project(
    name: &str,
    source_engine_state: ProjectSourceEngineState,
    language: HubLanguage,
) -> String {
    match source_engine_state {
        ProjectSourceEngineState::Ready => format_pair(
            language,
            format!("Build selected project {name}"),
            format!("构建已选项目 {name}"),
        ),
        ProjectSourceEngineState::MissingBinding => format_pair(
            language,
            format!("Bind a Source Engine to {name} before building"),
            format!("先为 {name} 绑定源码引擎"),
        ),
        ProjectSourceEngineState::Unavailable => format_pair(
            language,
            format!("Bound Source Engine for {name} is unavailable"),
            format!("{name} 绑定的源码引擎不可用"),
        ),
    }
}

fn project_action_detail(
    target: &QuickActionProjectTarget,
    language: HubLanguage,
    selected_prefix_en: &str,
    selected_prefix_zh: &str,
    latest_prefix_en: &str,
    latest_prefix_zh: &str,
    none_en: &str,
    none_zh: &str,
) -> String {
    match target {
        QuickActionProjectTarget::Selected { name, .. } => format_pair(
            language,
            format!("{selected_prefix_en} {name}"),
            format!("{selected_prefix_zh} {name}"),
        ),
        QuickActionProjectTarget::LatestRecent { name, .. } => format_pair(
            language,
            format!("{latest_prefix_en} {name}"),
            format!("{latest_prefix_zh} {name}"),
        ),
        QuickActionProjectTarget::StaleSelection => localized_pair(
            language,
            (
                "Selected project is no longer available",
                "已选项目不再可用",
            ),
        )
        .to_string(),
        QuickActionProjectTarget::None => localized_pair(language, (none_en, none_zh)).to_string(),
    }
}

fn format_pair(language: HubLanguage, english: String, chinese: String) -> String {
    match language {
        HubLanguage::English => english,
        HubLanguage::Chinese => chinese,
    }
}

fn quick_action_project_target(snapshot: &HubSnapshot) -> QuickActionProjectTarget {
    match snapshot.scope().project {
        ProjectScope::Selected(project) => QuickActionProjectTarget::Selected {
            name: project.display_name,
            source_engine_state: project_source_engine_state(project.engine_state),
        },
        ProjectScope::StaleSelection { .. } => QuickActionProjectTarget::StaleSelection,
        ProjectScope::LatestRecent(project) => QuickActionProjectTarget::LatestRecent {
            name: project.display_name,
            source_engine_state: project_source_engine_state(project.engine_state),
        },
        ProjectScope::None => QuickActionProjectTarget::None,
    }
}

fn project_source_engine_state(state: ProjectEngineScopeState) -> ProjectSourceEngineState {
    match state {
        ProjectEngineScopeState::Ready => ProjectSourceEngineState::Ready,
        ProjectEngineScopeState::MissingBinding => ProjectSourceEngineState::MissingBinding,
        ProjectEngineScopeState::Unavailable => ProjectSourceEngineState::Unavailable,
    }
}

fn quick_action_reference_detail(
    language: HubLanguage,
    english: &str,
    chinese: &str,
) -> SharedString {
    SharedString::from(localized_pair(language, (english, chinese)))
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
            SharedString::from("Build selected project Game")
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
            SharedString::from("Bind a Source Engine to Game before building")
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
            SharedString::from("Bound Source Engine for Game is unavailable")
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
            SharedString::from("Selected project is no longer available")
        );
        assert_eq!(
            action(&actions, "open-editor").detail,
            SharedString::from("Open Editor without a project")
        );
    }

    #[test]
    fn quick_actions_describe_no_selection_and_latest_recent_scope() {
        let latest = PathBuf::from("E:/Projects/Latest");
        let mut latest_snapshot = snapshot_with_projects(
            None,
            vec![
                RecentProject::new("Old", "E:/Projects/Old", 1),
                RecentProject::new("Latest", latest.clone(), 20),
            ],
        );
        bind_source_engine(&mut latest_snapshot, &latest);

        let latest_actions = quick_actions(&latest_snapshot, HubLanguage::English);
        assert!(action(&latest_actions, "build-project").enabled);
        assert_eq!(
            action(&latest_actions, "build-project").detail,
            SharedString::from("Build latest recent project Latest")
        );
        assert_eq!(
            action(&latest_actions, "package-project").detail,
            SharedString::from("Package latest recent project Latest")
        );

        let empty_actions = quick_actions(
            &snapshot_with_projects(None, Vec::new()),
            HubLanguage::English,
        );
        assert_eq!(
            action(&empty_actions, "build-project").detail,
            SharedString::from("Select a project with a bound Source Engine before building")
        );
        assert_eq!(
            action(&empty_actions, "package-project").detail,
            SharedString::from("Select a project before packaging")
        );
        assert_eq!(
            action(&empty_actions, "install-device").detail,
            SharedString::from("Select a project before installing")
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
            action_history: Vec::new(),
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
