use std::path::Path;

use serde::Serialize;

use crate::engines::SourceEngineInstall;
use crate::projects::{metadata_for_path, now_unix_ms, project_paths_match, RecentProject};
use crate::settings::HubLanguage;
use crate::state::{HubSnapshot, ProjectEngineScopeState, ProjectScope, TaskSeverity};
use crate::team::{TeamMemberEntry, TeamOverview};

use super::action_id::HubActionId;

mod action_history;
mod catalog;
mod coming_soon;
mod display;
mod localized;
mod new_project;
mod project_templates;
mod settings_dto;
mod source_engines;
mod ui_text;

use action_history::{action_history_rows, HubActionHistoryItem};
use catalog::{asset_rows, learn_rows, plugin_rows};
use coming_soon::{coming_soon_entries, HubComingSoonEntry};
use display::{path_text, path_text_en, relative_time};
pub(crate) use localized::HubTextBundle;
use new_project::{new_project_draft, HubNewProjectDraft};
use project_templates::{project_template_label, project_template_rows, HubProjectTemplate};
use settings_dto::{settings_summary, HubSettingsSummary};
pub(crate) use settings_dto::{
    validate_settings_for_save, HubSettingsActionPayload, HubSettingsPayload,
};
use source_engines::source_build_history_rows;
use ui_text::{ui_text, HubUiText};

const PROJECT_CARD_LIMIT: usize = 12;
const RECENT_ROW_LIMIT: usize = 8;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubViewModel {
    pub product_name: String,
    pub engine_version: String,
    pub active_page: String,
    pub page_title: String,
    pub page_subtitle: String,
    pub project_filter: String,
    pub project_sort: String,
    pub project_view_mode: String,
    pub project_subpage: String,
    pub project_templates: Vec<HubProjectTemplate>,
    pub new_project_draft: HubNewProjectDraft,
    pub search_query: String,
    pub selected_project_id: Option<String>,
    pub active_source_engine_id: Option<String>,
    pub task_summary: HubTaskSummary,
    pub task_status: Vec<HubStatusPill>,
    pub projects: Vec<HubProjectSummary>,
    pub browser_projects: Vec<HubRecentProject>,
    pub recent_projects: Vec<HubRecentProject>,
    pub selected_project: Option<HubProjectDetail>,
    pub quick_actions: Vec<HubQuickAction>,
    pub source_engines: Vec<HubSourceEngineSummary>,
    pub assets: Vec<HubAssetItem>,
    pub plugins: Vec<HubPluginItem>,
    pub learn_resources: Vec<HubLearnItem>,
    pub team: HubTeamSummary,
    pub action_history: Vec<HubActionHistoryItem>,
    pub coming_soon: Vec<HubComingSoonEntry>,
    pub settings: HubSettingsSummary,
    pub settings_draft: HubSettingsSummary,
    pub ui: HubUiText,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubStatusPill {
    pub id: String,
    pub label: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubTaskSummary {
    pub label: String,
    pub detail: String,
    pub tone: String,
    pub running: bool,
    pub recovery: Option<String>,
    pub operation: String,
    pub progress_percent: u8,
    pub task_id: u64,
    pub queued: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubProjectSummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub modified: String,
    pub engine_version: String,
    pub platform: String,
    pub cover_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubRecentProject {
    pub id: String,
    pub name: String,
    pub engine_version: String,
    pub modified: String,
    pub location: String,
    pub cover_id: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubProjectDetail {
    pub id: String,
    pub name: String,
    pub path: String,
    pub modified: String,
    pub engine_version: String,
    pub platform: String,
    pub cover_id: String,
    pub pinned: bool,
    pub engine_id: Option<String>,
    pub template_id: Option<String>,
    pub template_label: String,
    pub exists: bool,
    pub status: String,
    pub pending_delete: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubQuickAction {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub icon: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSourceBuildHistoryItem {
    pub id: String,
    pub status: String,
    pub status_tone: String,
    pub profile: String,
    pub jobs: Option<u16>,
    pub detail: String,
    pub secondary_detail: String,
    pub log_excerpt: String,
    pub command_line: Vec<String>,
    pub output_dir: String,
    pub finished: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSourceEngineSummary {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub output_path: String,
    pub status: String,
    pub active: bool,
    pub build_history: Vec<HubSourceBuildHistoryItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubAssetItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub detail: String,
    pub source: String,
    pub source_key: String,
    pub size: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubPluginItem {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub maturity: String,
    pub maturity_tone: String,
    pub scope: String,
    pub scope_key: String,
    pub editor_scoped: bool,
    pub module_count: usize,
    pub default_packaging: Vec<String>,
    pub package_root: String,
    pub manifest_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubLearnItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub category_key: String,
    pub source: String,
    pub source_key: String,
    pub summary: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubTeamSummary {
    pub repository_path: String,
    pub identity_name: String,
    pub identity_email: String,
    pub repository_available: bool,
    pub members: Vec<HubTeamMember>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubTeamMember {
    pub id: String,
    pub name: String,
    pub email: String,
    pub commits: u32,
    pub commits_label: String,
}

impl HubViewModel {
    pub(crate) fn from_snapshot(snapshot: &HubSnapshot) -> Self {
        let filtered_projects = snapshot.filtered_recent_projects();
        let active_engine = active_source_engine(snapshot);
        let active_source_engine_id = active_engine.map(|engine| engine.id.clone());
        let selected_project_id = snapshot
            .selected_project_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());
        let text = HubTextBundle::new(snapshot.settings.language);

        Self {
            product_name: "Zircon Hub".to_string(),
            engine_version: active_engine
                .map(source_engine_display_title)
                .unwrap_or_else(source_engine_version_label),
            active_page: snapshot.selected_page.id().to_string(),
            page_title: text.page_title(snapshot.selected_page).to_string(),
            page_subtitle: text.page_subtitle(snapshot.selected_page).to_string(),
            project_filter: snapshot.project_filter.id().to_string(),
            project_sort: snapshot.project_sort.id().to_string(),
            project_view_mode: snapshot.project_view_mode.id().to_string(),
            project_subpage: snapshot.project_subpage.id().to_string(),
            project_templates: project_template_rows(snapshot.settings.language),
            new_project_draft: new_project_draft(snapshot),
            search_query: snapshot.search_query.clone(),
            selected_project_id,
            active_source_engine_id,
            task_summary: task_summary(snapshot, text),
            task_status: header_statuses(snapshot, text),
            projects: filtered_projects
                .iter()
                .take(PROJECT_CARD_LIMIT)
                .map(|project| project_summary(snapshot, project, true, snapshot.settings.language))
                .collect(),
            browser_projects: filtered_projects
                .iter()
                .map(|project| recent_project_row(snapshot, project))
                .collect(),
            recent_projects: filtered_projects
                .iter()
                .take(RECENT_ROW_LIMIT)
                .map(|project| recent_project_row(snapshot, project))
                .collect(),
            selected_project: selected_project_detail(snapshot),
            quick_actions: quick_actions(snapshot),
            source_engines: source_engine_rows(snapshot),
            assets: asset_rows(snapshot),
            plugins: plugin_rows(snapshot),
            learn_resources: learn_rows(snapshot),
            team: team_summary(&snapshot.team, snapshot.settings.language),
            action_history: action_history_rows(
                snapshot,
                now_unix_ms(),
                snapshot.settings.language,
            ),
            coming_soon: coming_soon_entries(snapshot.settings.language),
            settings: settings_summary(&snapshot.settings),
            settings_draft: settings_summary(&snapshot.settings_draft),
            ui: ui_text(snapshot.settings.language),
        }
    }
}

fn task_summary(snapshot: &HubSnapshot, text: HubTextBundle) -> HubTaskSummary {
    HubTaskSummary {
        label: text.status_label(&snapshot.task_status.label),
        detail: text.render_message(&snapshot.task_status.detail),
        tone: if snapshot.task_status.running {
            "running".to_string()
        } else {
            severity_tone(snapshot.task_status.severity).to_string()
        },
        running: snapshot.task_status.running,
        recovery: snapshot
            .task_status
            .recovery
            .as_ref()
            .map(|recovery| text.render_message(recovery)),
        operation: operation_summary(snapshot, text),
        progress_percent: snapshot.task_status.progress_percent,
        task_id: snapshot.task_status.task_id,
        queued: snapshot.queued_background_actions,
    }
}

fn header_statuses(snapshot: &HubSnapshot, text: HubTextBundle) -> Vec<HubStatusPill> {
    if !snapshot.task_status.running && snapshot.task_status.severity == TaskSeverity::Info {
        return Vec::new();
    }

    let (id, tone) = if snapshot.task_status.running {
        ("running", "running")
    } else {
        let tone = severity_tone(snapshot.task_status.severity);
        (tone, tone)
    };

    vec![status(
        id,
        &text.status_label(&snapshot.task_status.label),
        tone,
    )]
}

fn status(id: &str, label: &str, tone: &str) -> HubStatusPill {
    HubStatusPill {
        id: id.to_string(),
        label: label.to_string(),
        tone: tone.to_string(),
    }
}

fn operation_summary(snapshot: &HubSnapshot, text: HubTextBundle) -> String {
    let Some(operation) = snapshot.task_status.operation else {
        return text.status_label(&snapshot.task_status.label);
    };
    let scope = text.operation_scope(operation);
    match snapshot
        .task_status
        .target
        .as_deref()
        .filter(|target| !target.trim().is_empty())
    {
        Some(target) => format!("{scope}: {}", text.operation_target(target)),
        None => scope.to_string(),
    }
}

fn project_summary(
    snapshot: &HubSnapshot,
    project: &RecentProject,
    include_modified_prefix: bool,
    language: HubLanguage,
) -> HubProjectSummary {
    let name = recent_project_display_name(project);
    let modified_relative = relative_time(now_unix_ms(), project.last_opened_unix_ms, language);
    HubProjectSummary {
        id: project_id(project),
        name: name.clone(),
        path: path_text(&project.path, language),
        modified: if include_modified_prefix {
            match language {
                HubLanguage::English => format!("Modified {modified_relative}"),
                HubLanguage::Chinese => format!("修改于 {modified_relative}"),
            }
        } else {
            modified_relative
        },
        engine_version: project_engine_version(snapshot, project),
        platform: project_platform(&project.path),
        cover_id: project_cover_id(&name),
    }
}

fn recent_project_row(snapshot: &HubSnapshot, project: &RecentProject) -> HubRecentProject {
    let summary = project_summary(snapshot, project, false, snapshot.settings.language);
    let pinned = metadata_for_path(&snapshot.project_metadata, &project.path)
        .is_some_and(|metadata| metadata.pinned);
    HubRecentProject {
        id: summary.id,
        name: summary.name,
        engine_version: summary.engine_version,
        modified: summary.modified,
        location: summary.path,
        cover_id: summary.cover_id,
        pinned,
    }
}

fn selected_project_detail(snapshot: &HubSnapshot) -> Option<HubProjectDetail> {
    let selected_path = snapshot.selected_project_path.as_ref()?;
    if let Some(project) = snapshot
        .recent_projects
        .iter()
        .find(|project| project_paths_match(&project.path, selected_path))
    {
        return Some(project_detail_from_recent(snapshot, project));
    }

    Some(stale_project_detail(snapshot, selected_path))
}

fn project_detail_from_recent(snapshot: &HubSnapshot, project: &RecentProject) -> HubProjectDetail {
    let summary = project_summary(snapshot, project, false, snapshot.settings.language);
    project_detail_from_parts(
        snapshot,
        &project.path,
        summary.id,
        summary.name,
        summary.modified,
        summary.engine_version,
        summary.platform,
        summary.cover_id,
    )
}

fn stale_project_detail(snapshot: &HubSnapshot, path: &Path) -> HubProjectDetail {
    let text = HubTextBundle::new(snapshot.settings.language);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| text.pair("Selected Project", "已选项目"))
        .to_string();
    project_detail_from_parts(
        snapshot,
        path,
        path_text(path, snapshot.settings.language),
        name.clone(),
        text.pair("Unknown", "未知").to_string(),
        project_engine_version_for_path(snapshot, path),
        project_platform(path),
        project_cover_id(&name),
    )
}

fn project_detail_from_parts(
    snapshot: &HubSnapshot,
    path: &Path,
    id: String,
    name: String,
    modified: String,
    engine_version: String,
    platform: String,
    cover_id: String,
) -> HubProjectDetail {
    let text = HubTextBundle::new(snapshot.settings.language);
    let metadata = metadata_for_path(&snapshot.project_metadata, path);
    let exists = path.exists();
    let pending_delete = snapshot
        .pending_delete_project_path
        .as_ref()
        .is_some_and(|pending| project_paths_match(pending, path));
    HubProjectDetail {
        id,
        name,
        path: path_text(path, snapshot.settings.language),
        modified,
        engine_version,
        platform,
        cover_id,
        pinned: metadata.is_some_and(|metadata| metadata.pinned),
        engine_id: metadata.and_then(|metadata| metadata.engine_id.clone()),
        template_id: metadata.and_then(|metadata| metadata.last_selected_template.clone()),
        template_label: project_template_label(
            metadata.and_then(|metadata| metadata.last_selected_template.as_deref()),
            snapshot.settings.language,
        ),
        exists,
        status: if exists {
            text.pair("Available", "可用")
        } else {
            text.pair("Missing", "缺失")
        }
        .to_string(),
        pending_delete,
    }
}

fn quick_actions(snapshot: &HubSnapshot) -> Vec<HubQuickAction> {
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

fn source_engine_rows(snapshot: &HubSnapshot) -> Vec<HubSourceEngineSummary> {
    let text = HubTextBundle::new(snapshot.settings.language);
    let now_ms = now_unix_ms();
    snapshot
        .engines
        .iter()
        .map(|engine| {
            let active = Some(engine.id.as_str()) == snapshot.active_engine_id.as_deref()
                || (snapshot.active_engine_id.is_none()
                    && snapshot
                        .engines
                        .first()
                        .is_some_and(|first| first.id == engine.id));
            HubSourceEngineSummary {
                id: engine.id.clone(),
                name: source_engine_display_title(engine),
                source_path: path_text(&engine.source_dir, snapshot.settings.language),
                output_path: path_text(&engine.output_dir, snapshot.settings.language),
                status: if active {
                    text.pair("Active", "当前")
                } else {
                    text.pair("Registered", "已注册")
                }
                .to_string(),
                active,
                build_history: source_build_history_rows(
                    engine,
                    now_ms,
                    snapshot.settings.language,
                ),
            }
        })
        .collect()
}

fn team_summary(team: &TeamOverview, language: HubLanguage) -> HubTeamSummary {
    HubTeamSummary {
        repository_path: path_text(&team.repository_path, language),
        identity_name: team.identity_name.clone(),
        identity_email: team.identity_email.clone(),
        repository_available: !team.repository_path.as_os_str().is_empty(),
        members: team
            .members
            .iter()
            .map(|member| team_member_row(member, language))
            .collect(),
    }
}

fn team_member_row(member: &TeamMemberEntry, language: HubLanguage) -> HubTeamMember {
    let id = format!("{}:{}", member.name, member.email);
    HubTeamMember {
        id,
        name: member.name.clone(),
        email: member.email.clone(),
        commits: member.commits,
        commits_label: commit_count_label(member.commits, language),
    }
}

fn commit_count_label(commits: u32, language: HubLanguage) -> String {
    match language {
        HubLanguage::English => {
            if commits == 1 {
                "1 commit".to_string()
            } else {
                format!("{commits} commits")
            }
        }
        HubLanguage::Chinese => format!("{commits} 次提交"),
    }
}

fn active_source_engine(snapshot: &HubSnapshot) -> Option<&SourceEngineInstall> {
    snapshot
        .active_engine_id
        .as_deref()
        .and_then(|id| snapshot.engines.iter().find(|engine| engine.id == id))
        .or_else(|| snapshot.engines.first())
}

fn source_engine_version_label() -> String {
    "Zircon Engine 1.8.2".to_string()
}

fn source_engine_display_title(engine: &SourceEngineInstall) -> String {
    if matches!(
        engine.display_name.as_str(),
        "ZirconEngine Source" | "zircon-1.8.2 Source"
    ) {
        source_engine_version_label()
    } else {
        engine.display_name.clone()
    }
}

fn project_engine_version(snapshot: &HubSnapshot, project: &RecentProject) -> String {
    project_engine_version_for_path(snapshot, &project.path)
}

fn project_engine_version_for_path(snapshot: &HubSnapshot, path: &Path) -> String {
    metadata_for_path(&snapshot.project_metadata, path)
        .and_then(|metadata| metadata.engine_id.as_deref())
        .and_then(|engine_id| {
            snapshot
                .engines
                .iter()
                .find(|engine| engine.id == engine_id)
        })
        .map(source_engine_display_title)
        .unwrap_or_else(|| "1.8.2".to_string())
}

fn project_platform(path: &Path) -> String {
    let text = path.to_string_lossy();
    if text.contains(':') || text.contains('\\') {
        "Windows".to_string()
    } else {
        "Linux".to_string()
    }
}

fn project_id(project: &RecentProject) -> String {
    path_text_en(&project.path)
}

fn project_cover_id(name: &str) -> String {
    match name {
        "Elysium Chronicles" => "elysium",
        "Stellar Outpost" => "stellar-outpost",
        "Sands of Time" => "sands-of-time",
        "Whispering Woods" => "whispering-woods",
        "Neon Streets" => "neon-streets",
        _ => "elysium",
    }
    .to_string()
}

fn recent_project_display_name(project: &RecentProject) -> String {
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

fn severity_tone(severity: TaskSeverity) -> &'static str {
    match severity {
        TaskSeverity::Info => "neutral",
        TaskSeverity::Success => "success",
        TaskSeverity::Warning => "warning",
        TaskSeverity::Error => "error",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::engines::SourceEngineInstall;
    use crate::projects::{project_metadata_key, ProjectMetadata};
    use crate::settings::{HubLanguage, HubSettings};
    use crate::state::{
        DeliveryMessageId, HubMessage, HubMessageId, HubPage, ProjectFilterMode, ProjectMessageId,
        ProjectSortMode, ProjectSubpage, ProjectViewMode, SettingsMessageId, TaskStatus,
        TASK_PROGRESS_PREPARED_PERCENT,
    };
    use crate::team::TeamOverview;

    use super::display::{MILLIS_PER_DAY, MILLIS_PER_HOUR};
    use super::*;

    #[test]
    fn view_model_projects_come_from_snapshot_filtering_and_state_ids() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::Name,
            project_view_mode: ProjectViewMode::List,
            project_subpage: ProjectSubpage::ProjectBrowser,
            search_query: "stellar".to_string(),
            selected_project_path: Some(PathBuf::from("E:/Projects/StellarOutpost")),
            new_project_name: String::new(),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            queued_background_actions: 0,
            recent_projects: vec![
                RecentProject::new("Elysium Chronicles", "E:/Projects/Elysium", 30),
                RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
            ],
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
            settings_draft: HubSettings::default(),
        };

        let model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(model.projects.len(), 1);
        assert_eq!(model.projects[0].name, "Stellar Outpost");
        assert_eq!(model.browser_projects.len(), 1);
        assert_eq!(model.project_view_mode, "list");
        assert_eq!(model.project_subpage, "project-browser");
        assert_eq!(
            model.selected_project_id.as_deref(),
            Some("E:/Projects/StellarOutpost")
        );
        assert_eq!(
            model
                .selected_project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Stellar Outpost")
        );
        assert_eq!(model.task_summary.label, "就绪");
        assert_eq!(model.task_summary.operation, "Hub");
        assert_eq!(model.task_summary.progress_percent, 0);
        assert_eq!(model.assets.len(), 0);
        assert_eq!(model.plugins.len(), 0);
        assert_eq!(model.learn_resources.len(), 0);
        assert_eq!(model.team.members.len(), 0);
        assert_eq!(model.action_history.len(), 0);
    }

    #[test]
    fn quick_actions_use_selected_project_scope_and_engine_binding() {
        let selected = PathBuf::from("E:/Projects/Game");
        let mut snapshot = snapshot_with_projects(
            Some(PathBuf::from("E:\\Projects\\Game\\")),
            vec![RecentProject::new("Game", selected.clone(), 10)],
        );
        bind_source_engine(&mut snapshot, &selected);

        let model = HubViewModel::from_snapshot(&snapshot);

        assert!(quick_action(&model, "build-project").enabled);
        assert_eq!(
            quick_action(&model, "build-project").detail,
            "构建已选项目 Game"
        );
        assert_eq!(
            quick_action(&model, "package-project").detail,
            "打包已选项目 Game"
        );
    }

    #[test]
    fn selected_project_template_label_localizes_stable_template_metadata() {
        let selected = PathBuf::from("E:/Projects/Game");
        let mut snapshot = snapshot_with_projects(
            Some(selected.clone()),
            vec![RecentProject::new("Game", selected.clone(), 10)],
        );
        snapshot.settings.language = HubLanguage::Chinese;
        snapshot.project_metadata.insert(
            project_metadata_key(&selected),
            ProjectMetadata {
                last_selected_template: Some("renderable-empty".to_string()),
                ..ProjectMetadata::default()
            },
        );

        let model = HubViewModel::from_snapshot(&snapshot);
        let project = model
            .selected_project
            .as_ref()
            .expect("selected project detail should be projected");

        assert_eq!(project.template_id.as_deref(), Some("renderable-empty"));
        assert_eq!(project.template_label, "可渲染空项目");
    }

    #[test]
    fn quick_actions_disable_unbound_or_stale_project_targets() {
        let selected = PathBuf::from("E:/Projects/Game");
        let unbound = HubViewModel::from_snapshot(&snapshot_with_projects(
            Some(PathBuf::from("E:\\Projects\\Game\\")),
            vec![RecentProject::new("Game", selected, 10)],
        ));

        assert!(!quick_action(&unbound, "build-project").enabled);
        assert!(quick_action(&unbound, "package-project").enabled);
        assert_eq!(
            quick_action(&unbound, "build-project").detail,
            "先为 Game 绑定源码引擎"
        );

        let stale = HubViewModel::from_snapshot(&snapshot_with_projects(
            Some(PathBuf::from("E:/Projects/Missing")),
            vec![RecentProject::new("Latest", "E:/Projects/Latest", 20)],
        ));

        assert!(!quick_action(&stale, "build-project").enabled);
        assert!(!quick_action(&stale, "package-project").enabled);
        assert_eq!(
            quick_action(&stale, "build-project").detail,
            "已选项目不再可用"
        );
        assert_eq!(
            quick_action(&stale, "open-editor").detail,
            "不带项目打开编辑器"
        );
    }

    #[test]
    fn quick_actions_use_latest_recent_only_when_no_project_is_selected() {
        let latest = PathBuf::from("E:/Projects/Latest");
        let mut snapshot = snapshot_with_projects(
            None,
            vec![
                RecentProject::new("Old", "E:/Projects/Old", 1),
                RecentProject::new("Latest", latest.clone(), 20),
            ],
        );
        bind_source_engine(&mut snapshot, &latest);

        let model = HubViewModel::from_snapshot(&snapshot);

        assert!(quick_action(&model, "build-project").enabled);
        assert_eq!(
            quick_action(&model, "build-project").detail,
            "构建最近项目 Latest"
        );
        assert_eq!(
            quick_action(&model, "install-device").detail,
            "安装最近项目 Latest"
        );
    }

    #[test]
    fn source_engine_rows_localize_status_labels() {
        let mut snapshot = snapshot_with_projects(None, Vec::new());
        snapshot.settings.language = HubLanguage::Chinese;
        snapshot.active_engine_id = Some("source-local".to_string());
        snapshot.engines.push(SourceEngineInstall {
            id: "source-local".to_string(),
            display_name: "Local Source".to_string(),
            source_dir: PathBuf::from("E:/Source/ZirconEngine"),
            output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        });
        snapshot.engines.push(SourceEngineInstall {
            id: "source-backup".to_string(),
            display_name: "Backup Source".to_string(),
            source_dir: PathBuf::from("E:/Source/ZirconEngineBackup"),
            output_dir: PathBuf::from("E:/Source/ZirconEngineBackup/out"),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        });

        let model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(model.source_engines[0].status, "当前");
        assert_eq!(model.source_engines[1].status, "已注册");
    }

    #[test]
    fn relative_time_uses_compact_labels() {
        let now = 10 * MILLIS_PER_DAY;

        assert_eq!(relative_time(now, now, HubLanguage::English), "just now");
        assert_eq!(
            relative_time(now, now - (2 * MILLIS_PER_HOUR), HubLanguage::English),
            "2h ago"
        );
        assert_eq!(
            relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::English),
            "3d ago"
        );
        assert_eq!(relative_time(now, now, HubLanguage::Chinese), "刚刚");
        assert_eq!(
            relative_time(now, now - (2 * MILLIS_PER_HOUR), HubLanguage::Chinese),
            "2 小时前"
        );
        assert_eq!(
            relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::Chinese),
            "3 天前"
        );
    }

    #[test]
    fn task_summary_localizes_running_message_detail_and_operation_scope() {
        let mut snapshot = snapshot_with_projects(None, Vec::new());
        snapshot.settings.language = HubLanguage::Chinese;
        snapshot.task_status = TaskStatus::running_operation(
            "Packaging",
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::CopyingProjectToPackage,
            )),
            crate::state::TaskOperationKind::Project,
            "Game",
        );

        let model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(model.task_summary.label, "打包中");
        assert_eq!(model.task_summary.detail, "正在复制项目到包输出目录");
        assert_eq!(model.task_summary.operation, "项目: Game");
    }

    #[test]
    fn task_summary_projects_backend_progress_percent() {
        let mut snapshot = snapshot_with_projects(None, Vec::new());
        snapshot.task_status = TaskStatus::running_operation(
            "Packaging",
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::CopyingProjectToPackage,
            )),
            crate::state::TaskOperationKind::Project,
            "Game",
        )
        .with_progress_percent(TASK_PROGRESS_PREPARED_PERCENT);

        let model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(
            model.task_summary.progress_percent,
            TASK_PROGRESS_PREPARED_PERCENT
        );
    }

    #[test]
    fn task_summary_localizes_import_cancelled_status() {
        let mut snapshot = snapshot_with_projects(None, Vec::new());
        snapshot.settings.language = HubLanguage::Chinese;
        snapshot.task_status = TaskStatus::warning(
            "Import cancelled",
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::NoProjectFolderSelected,
            )),
            HubMessage::new(HubMessageId::Project(ProjectMessageId::RunImportAgain)),
        )
        .with_operation(crate::state::TaskOperationKind::Project, "Import Project");

        let model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(model.task_summary.label, "已取消导入");
        assert_eq!(model.task_summary.detail, "未选择项目文件夹");
        assert_eq!(
            model.task_summary.recovery.as_deref(),
            Some("重新运行导入项目并选择 Zircon 项目文件夹")
        );
    }

    #[test]
    fn task_summary_localizes_backend_operation_targets() {
        let mut snapshot = snapshot_with_projects(None, Vec::new());
        snapshot.settings.language = HubLanguage::Chinese;
        snapshot.task_status = TaskStatus::error(
            "Open Output failed",
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::OpenOutputTargetRequired,
            )),
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::ChooseRecordedOutputRecovery,
            )),
        )
        .with_operation(crate::state::TaskOperationKind::Process, "Output Folder");

        let output_model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(output_model.task_summary.operation, "进程: 输出文件夹");

        snapshot.task_status = TaskStatus::success(
            "Settings saved",
            HubMessage::with_params(
                HubMessageId::Settings(SettingsMessageId::SettingsSavedPath),
                ["E:\\Git\\ZirconEngine\\zircon_hub\\hub.toml"],
            ),
        )
        .with_operation(crate::state::TaskOperationKind::Settings, "Hub settings");

        let settings_model = HubViewModel::from_snapshot(&snapshot);

        assert_eq!(settings_model.task_summary.operation, "设置: Hub 设置");
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
            new_project_name: String::new(),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            queued_background_actions: 0,
            recent_projects,
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
            settings_draft: HubSettings::default(),
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

    fn quick_action<'a>(model: &'a HubViewModel, id: &str) -> &'a HubQuickAction {
        model
            .quick_actions
            .iter()
            .find(|action| action.id == id)
            .expect("quick action id should exist")
    }
}
