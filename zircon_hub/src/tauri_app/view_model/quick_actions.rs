use crate::settings::HubLanguage;
use crate::state::{HubSnapshot, ProjectEngineScopeState, ProjectScope};
use crate::tauri_app::action_id::HubActionId;

use super::HubQuickAction;

pub(super) fn quick_actions(snapshot: &HubSnapshot) -> Vec<HubQuickAction> {
    let project_target = quick_action_project_target(snapshot);
    [
        quick_action(
            HubActionId::BuildProject.as_str(),
            localized_quick_action_title(QuickActionKind::BuildProject, snapshot.settings.language),
            "build",
            quick_action_detail(
                QuickActionKind::BuildProject,
                &project_target,
                snapshot.settings.language,
            ),
            quick_action_enabled(QuickActionKind::BuildProject, &project_target),
        ),
        quick_action(
            HubActionId::InstallDevice.as_str(),
            localized_quick_action_title(
                QuickActionKind::InstallToDevice,
                snapshot.settings.language,
            ),
            "device",
            quick_action_detail(
                QuickActionKind::InstallToDevice,
                &project_target,
                snapshot.settings.language,
            ),
            quick_action_enabled(QuickActionKind::InstallToDevice, &project_target),
        ),
        quick_action(
            HubActionId::PackageProject.as_str(),
            localized_quick_action_title(
                QuickActionKind::PackageProject,
                snapshot.settings.language,
            ),
            "package",
            quick_action_detail(
                QuickActionKind::PackageProject,
                &project_target,
                snapshot.settings.language,
            ),
            quick_action_enabled(QuickActionKind::PackageProject, &project_target),
        ),
        quick_action(
            HubActionId::OpenEditor.as_str(),
            localized_quick_action_title(QuickActionKind::OpenEditor, snapshot.settings.language),
            "editor",
            quick_action_detail(
                QuickActionKind::OpenEditor,
                &project_target,
                snapshot.settings.language,
            ),
            quick_action_enabled(QuickActionKind::OpenEditor, &project_target),
        ),
    ]
    .into_iter()
    .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuickActionKind {
    BuildProject,
    InstallToDevice,
    PackageProject,
    OpenEditor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum QuickActionProjectTarget {
    Selected {
        name: String,
        source_engine_state: QuickActionSourceEngineState,
    },
    LatestRecent {
        name: String,
        source_engine_state: QuickActionSourceEngineState,
    },
    StaleSelection,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuickActionSourceEngineState {
    Ready,
    MissingBinding,
    Unavailable,
}

fn localized_quick_action_title(action: QuickActionKind, language: HubLanguage) -> &'static str {
    match (action, language) {
        (QuickActionKind::BuildProject, HubLanguage::Chinese) => "构建项目",
        (QuickActionKind::InstallToDevice, HubLanguage::Chinese) => "安装到设备",
        (QuickActionKind::PackageProject, HubLanguage::Chinese) => "打包项目",
        (QuickActionKind::OpenEditor, HubLanguage::Chinese) => "在编辑器中打开",
        (QuickActionKind::BuildProject, HubLanguage::English) => "Build Project",
        (QuickActionKind::InstallToDevice, HubLanguage::English) => "Install to Device",
        (QuickActionKind::PackageProject, HubLanguage::English) => "Package Project",
        (QuickActionKind::OpenEditor, HubLanguage::English) => "Open in Editor",
    }
}

fn quick_action(
    id: &str,
    title: &str,
    icon: &str,
    detail: String,
    enabled: bool,
) -> HubQuickAction {
    HubQuickAction {
        id: id.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        icon: icon.to_string(),
        enabled,
    }
}

fn quick_action_project_target(snapshot: &HubSnapshot) -> QuickActionProjectTarget {
    match snapshot.scope().project {
        ProjectScope::Selected(project) => QuickActionProjectTarget::Selected {
            name: project.display_name,
            source_engine_state: quick_action_source_engine_state(project.engine_state),
        },
        ProjectScope::LatestRecent(project) => QuickActionProjectTarget::LatestRecent {
            name: project.display_name,
            source_engine_state: quick_action_source_engine_state(project.engine_state),
        },
        ProjectScope::StaleSelection { .. } => QuickActionProjectTarget::StaleSelection,
        ProjectScope::None => QuickActionProjectTarget::None,
    }
}

fn quick_action_source_engine_state(
    state: ProjectEngineScopeState,
) -> QuickActionSourceEngineState {
    match state {
        ProjectEngineScopeState::Ready => QuickActionSourceEngineState::Ready,
        ProjectEngineScopeState::MissingBinding => QuickActionSourceEngineState::MissingBinding,
        ProjectEngineScopeState::Unavailable => QuickActionSourceEngineState::Unavailable,
    }
}

fn quick_action_enabled(action: QuickActionKind, target: &QuickActionProjectTarget) -> bool {
    match action {
        QuickActionKind::BuildProject => target.has_source_engine(),
        QuickActionKind::PackageProject | QuickActionKind::InstallToDevice => target.has_project(),
        QuickActionKind::OpenEditor => true,
    }
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
            } => *source_engine_state == QuickActionSourceEngineState::Ready,
            Self::None | Self::StaleSelection => false,
        }
    }
}

fn quick_action_detail(
    action: QuickActionKind,
    target: &QuickActionProjectTarget,
    language: HubLanguage,
) -> String {
    match (action, target) {
        (
            QuickActionKind::BuildProject,
            QuickActionProjectTarget::Selected {
                name,
                source_engine_state,
            },
        ) => build_detail_for_project(name, *source_engine_state, language),
        (
            QuickActionKind::BuildProject,
            QuickActionProjectTarget::LatestRecent {
                name,
                source_engine_state,
            },
        ) => match source_engine_state {
            QuickActionSourceEngineState::Ready => format_pair(
                language,
                format!("Build latest recent project {name}"),
                format!("构建最近项目 {name}"),
            ),
            QuickActionSourceEngineState::MissingBinding => format_pair(
                language,
                format!("Bind a Source Engine to latest recent project {name} before building"),
                format!("先为最近项目 {name} 绑定源码引擎"),
            ),
            QuickActionSourceEngineState::Unavailable => format_pair(
                language,
                format!("Bound Source Engine for latest recent project {name} is unavailable"),
                format!("最近项目 {name} 绑定的源码引擎不可用"),
            ),
        },
        (QuickActionKind::BuildProject, QuickActionProjectTarget::StaleSelection) => {
            localized_pair(
                language,
                (
                    "Selected project is no longer available",
                    "已选项目不再可用",
                ),
            )
            .to_string()
        }
        (QuickActionKind::BuildProject, QuickActionProjectTarget::None) => localized_pair(
            language,
            (
                "Select a project with a bound Source Engine before building",
                "先选择已绑定源码引擎的项目",
            ),
        )
        .to_string(),
        (QuickActionKind::PackageProject, target) => project_action_detail(
            target,
            language,
            "Package selected project",
            "打包已选项目",
            "Package latest recent project",
            "打包最近项目",
            "Select a project before packaging",
            "先选择项目再打包",
        ),
        (QuickActionKind::InstallToDevice, target) => project_action_detail(
            target,
            language,
            "Install selected project",
            "安装已选项目",
            "Install latest recent project",
            "安装最近项目",
            "Select a project before installing",
            "先选择项目再安装",
        ),
        (QuickActionKind::OpenEditor, QuickActionProjectTarget::Selected { name, .. }) => {
            format_pair(
                language,
                format!("Open {name} in Editor"),
                format!("在编辑器中打开 {name}"),
            )
        }
        (QuickActionKind::OpenEditor, QuickActionProjectTarget::LatestRecent { name, .. }) => {
            format_pair(
                language,
                format!("Open latest recent project {name} in Editor"),
                format!("在编辑器中打开最近项目 {name}"),
            )
        }
        (QuickActionKind::OpenEditor, QuickActionProjectTarget::StaleSelection)
        | (QuickActionKind::OpenEditor, QuickActionProjectTarget::None) => localized_pair(
            language,
            ("Open Editor without a project", "不带项目打开编辑器"),
        )
        .to_string(),
    }
}

fn build_detail_for_project(
    name: &str,
    source_engine_state: QuickActionSourceEngineState,
    language: HubLanguage,
) -> String {
    match source_engine_state {
        QuickActionSourceEngineState::Ready => format_pair(
            language,
            format!("Build selected project {name}"),
            format!("构建已选项目 {name}"),
        ),
        QuickActionSourceEngineState::MissingBinding => format_pair(
            language,
            format!("Bind a Source Engine to {name} before building"),
            format!("先为 {name} 绑定源码引擎"),
        ),
        QuickActionSourceEngineState::Unavailable => format_pair(
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

fn localized_pair<'a>(language: HubLanguage, pair: (&'a str, &'a str)) -> &'a str {
    match language {
        HubLanguage::English => pair.0,
        HubLanguage::Chinese => pair.1,
    }
}
