use std::path::Path;

use slint::{ModelRc, SharedString, VecModel};

use crate::engines::SourceEngineInstall;
use crate::projects::{metadata_for_path, now_unix_ms, project_paths_match, RecentProject};
use crate::settings::{HubLanguage, HubSettings};
use crate::state::{HubPage, HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectViewMode};

use super::localization;
use super::{
    AssetData, CloudServiceData, CloudSummaryData, HeaderStatusData, LearnData, NavItemData,
    NavigationItem, PluginData, QuickActionData, SettingStatusData, SourceBuildHistoryRowData,
    SourceEngineData, SourceEngineRowData, TeamData, TeamMemberData, UiTextData,
};

mod assets;
mod cloud;
mod learn;
mod media;
mod plugins;
mod projects;
mod quick_actions;
mod team;

const PROJECT_CARD_LIMIT: usize = 12;
const RECENT_ROW_LIMIT: usize = 8;
const BUILD_HISTORY_ROW_LIMIT: usize = 5;
const MILLIS_PER_MINUTE: u64 = 60_000;
const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;
const MILLIS_PER_WEEK: u64 = 7 * MILLIS_PER_DAY;

pub(super) use projects::{
    dashboard_project_rows, dashboard_project_title, project_browser_rows, project_cards,
    project_create_enabled, project_create_engine_label, project_create_template_label,
    project_detail, project_engine_rows, project_subpage_id, project_templates,
};
pub(super) fn model_from<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items))
}

pub(super) fn quick_actions(snapshot: &HubSnapshot, language: HubLanguage) -> Vec<QuickActionData> {
    quick_actions::quick_actions(snapshot, language)
}

pub(super) fn ui_text(language: HubLanguage) -> UiTextData {
    localization::ui_text(language)
}

pub(super) fn navigation_items(selected_page: HubPage, language: HubLanguage) -> Vec<NavItemData> {
    [
        (HubPage::Projects, "[]"),
        (HubPage::Editor, "<>"),
        (HubPage::Assets, "{}"),
        (HubPage::Builds, "/>"),
        (HubPage::Plugins, "##"),
        (HubPage::Cloud, "~~"),
        (HubPage::Team, "**"),
        (HubPage::Learn, "??"),
        (HubPage::Settings, "::"),
    ]
    .into_iter()
    .map(|(page, icon)| {
        let icon_image = media::navigation_icon(page);
        NavItemData {
            id: shared(page.id()),
            title: localization::page_title(page, language),
            icon: shared(icon),
            icon_image: icon_image.clone().unwrap_or_default(),
            has_icon_image: icon_image.is_some(),
            active: page == selected_page,
        }
    })
    .collect()
}

pub(super) fn material_navigation_items(items: &[NavItemData]) -> Vec<NavigationItem> {
    items
        .iter()
        .map(|item| NavigationItem {
            icon: item.icon_image.clone(),
            selected_icon: item.icon_image.clone(),
            text: item.title.clone(),
            show_badge: false,
            badge: SharedString::default(),
        })
        .collect()
}

pub(super) fn selected_nav_index(items: &[NavItemData]) -> i32 {
    items
        .iter()
        .position(|item| item.active)
        .map(|index| index as i32)
        .unwrap_or_default()
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

pub(super) fn plugin_items(snapshot: &HubSnapshot) -> Vec<PluginData> {
    plugins::plugin_items(snapshot)
}

pub(super) fn asset_items(snapshot: &HubSnapshot) -> Vec<AssetData> {
    assets::asset_items(snapshot)
}

pub(super) fn learn_items(snapshot: &HubSnapshot) -> Vec<LearnData> {
    learn::learn_items(snapshot)
}

pub(super) fn team_summary(snapshot: &HubSnapshot) -> TeamData {
    team::team_summary(snapshot)
}

pub(super) fn team_members(snapshot: &HubSnapshot) -> Vec<TeamMemberData> {
    team::team_members(snapshot)
}

pub(super) fn cloud_summary(snapshot: &HubSnapshot) -> CloudSummaryData {
    cloud::cloud_summary(snapshot)
}

pub(super) fn cloud_services(language: HubLanguage) -> Vec<CloudServiceData> {
    cloud::cloud_services(language)
}

pub(super) fn source_engine_data(snapshot: &HubSnapshot) -> SourceEngineData {
    match selected_project_engine_context(snapshot) {
        BuildContextEngine::Engine(engine) => source_engine_data_for_context(
            Some(engine),
            !snapshot.engines.is_empty(),
            &snapshot.settings,
        ),
        BuildContextEngine::SelectedProjectUnbound => source_engine_data_for_missing_context(
            localization::text(snapshot.settings.language, "No Source Engine", "无源码引擎"),
            localization::text(
                snapshot.settings.language,
                "Bind project engine",
                "绑定项目引擎",
            ),
            &snapshot.settings,
        ),
        BuildContextEngine::SelectedProjectUnavailable => source_engine_data_for_missing_context(
            localization::text(
                snapshot.settings.language,
                "Source Engine unavailable",
                "源码引擎不可用",
            ),
            localization::text(snapshot.settings.language, "Unavailable", "不可用"),
            &snapshot.settings,
        ),
        BuildContextEngine::NoSelectedProject => source_engine_data_for_context(
            selected_engine(&snapshot.engines, snapshot.active_engine_id.as_deref()),
            !snapshot.engines.is_empty(),
            &snapshot.settings,
        ),
    }
}

fn source_engine_data_for_context(
    engine: Option<&SourceEngineInstall>,
    has_registered_engine: bool,
    settings: &HubSettings,
) -> SourceEngineData {
    let language = settings.language;
    let (title, source_path, output_path, last_build) = engine.map_or_else(
        || {
            (
                "No Source Engine".to_string(),
                path_text(&settings.default_source_dir, language),
                path_text(&settings.default_build_output_dir, language),
                localization::text(language, "Not built yet", "尚未构建").to_string(),
            )
        },
        |engine| {
            (
                engine.display_name.clone(),
                path_text(&engine.source_dir, language),
                path_text(&engine.output_dir, language),
                engine
                    .last_build_unix_ms
                    .map(|value| {
                        format!(
                            "{} {}",
                            localization::text(language, "Built", "构建于"),
                            relative_time(now_unix_ms(), value, language)
                        )
                    })
                    .unwrap_or_else(|| {
                        localization::text(language, "Not built yet", "尚未构建").to_string()
                    }),
            )
        },
    );

    SourceEngineData {
        title: shared(title),
        version: shared(format!("Zircon Engine {}", env!("CARGO_PKG_VERSION"))),
        source_path: shared(source_path),
        output_path: shared(output_path),
        last_build: shared(last_build),
        status: localization::text(
            language,
            if !has_registered_engine {
                "Configure source"
            } else {
                "Source registered"
            },
            if !has_registered_engine {
                "配置源码"
            } else {
                "源码已注册"
            },
        ),
        build_profile: shared(settings.build_profile.as_mode()),
        jobs: shared(settings.jobs.to_string()),
    }
}

fn source_engine_data_for_missing_context(
    title: SharedString,
    status: SharedString,
    settings: &HubSettings,
) -> SourceEngineData {
    let not_configured = localization::text(settings.language, "Not configured", "未配置");
    SourceEngineData {
        title,
        version: shared(format!("Zircon Engine {}", env!("CARGO_PKG_VERSION"))),
        source_path: not_configured.clone(),
        output_path: not_configured,
        last_build: localization::text(settings.language, "Not built yet", "尚未构建"),
        status,
        build_profile: shared(settings.build_profile.as_mode()),
        jobs: shared(settings.jobs.to_string()),
    }
}

pub(super) fn source_engine_rows(snapshot: &HubSnapshot) -> Vec<SourceEngineRowData> {
    let language = snapshot.settings.language;
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
            SourceEngineRowData {
                id: shared(&engine.id),
                title: shared(&engine.display_name),
                version: shared(format!("Zircon Engine {}", env!("CARGO_PKG_VERSION"))),
                source_path: shared(path_text(&engine.source_dir, language)),
                output_path: shared(path_text(&engine.output_dir, language)),
                last_build: shared(
                    engine
                        .last_build_unix_ms
                        .map(|value| {
                            format!(
                                "{} {}",
                                localization::text(language, "Built", "构建于"),
                                relative_time(now_unix_ms(), value, language)
                            )
                        })
                        .unwrap_or_else(|| {
                            localization::text(language, "Not built yet", "尚未构建").to_string()
                        }),
                ),
                status: localization::text(
                    language,
                    if active { "Active" } else { "Registered" },
                    if active { "当前" } else { "已注册" },
                ),
                active,
            }
        })
        .collect()
}

pub(super) fn source_build_history_rows(snapshot: &HubSnapshot) -> Vec<SourceBuildHistoryRowData> {
    let language = snapshot.settings.language;
    build_context_engine(snapshot)
        .map(|engine| {
            engine
                .build_history
                .iter()
                .take(BUILD_HISTORY_ROW_LIMIT)
                .map(|record| SourceBuildHistoryRowData {
                    status: localization::text(
                        language,
                        if record.status == "success" {
                            "Success"
                        } else {
                            "Failed"
                        },
                        if record.status == "success" {
                            "成功"
                        } else {
                            "失败"
                        },
                    ),
                    finished: shared(relative_time(
                        now_unix_ms(),
                        record.finished_unix_ms,
                        language,
                    )),
                    profile: shared(match record.jobs {
                        Some(jobs) => match language {
                            HubLanguage::English => format!("{} / {jobs} jobs", record.profile),
                            HubLanguage::Chinese => format!("{} / {jobs} 任务", record.profile),
                        },
                        None => record.profile.clone(),
                    }),
                    output_path: shared(path_text(&record.output_dir, language)),
                    detail: localization::text(
                        language,
                        if record.detail.trim().is_empty() {
                            "No build detail"
                        } else {
                            record.detail.as_str()
                        },
                        if record.detail.trim().is_empty() {
                            "没有构建详情"
                        } else {
                            record.detail.as_str()
                        },
                    ),
                    success: record.status == "success",
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn settings_statuses(settings: &HubSettings) -> Vec<SettingStatusData> {
    let language = settings.language;
    vec![
        executable_status("Python", &settings.python_path, language),
        executable_status("Cargo", &settings.cargo_path, language),
        executable_status("Rustup", &settings.rustup_path, language),
        directory_status(
            &localization::text(language, "Project Directory", "项目目录"),
            &settings.default_project_dir,
            &localization::text(language, "Ready", "就绪"),
            &localization::text(language, "Created when needed", "按需创建"),
            &localization::text(language, "Not configured", "未配置"),
        ),
        directory_status(
            &localization::text(language, "Source Checkout", "源码检出"),
            &settings.default_source_dir,
            &localization::text(language, "Ready", "就绪"),
            &localization::text(language, "Missing source checkout", "源码检出缺失"),
            &localization::text(language, "Not configured", "未配置"),
        ),
        directory_status(
            &localization::text(language, "Build Output", "构建输出"),
            &settings.default_build_output_dir,
            &localization::text(language, "Ready", "就绪"),
            &localization::text(language, "Created by builds", "构建时创建"),
            &localization::text(language, "Not configured", "未配置"),
        ),
        directory_status(
            &localization::text(language, "Device Install", "设备安装"),
            &settings.default_device_install_dir,
            &localization::text(language, "Ready", "就绪"),
            &localization::text(language, "Created when installing", "安装时创建"),
            &localization::text(language, "Not configured", "未配置"),
        ),
    ]
}

pub(super) fn header_statuses(snapshot: &HubSnapshot) -> Vec<HeaderStatusData> {
    let language = snapshot.settings.language;
    let settings_statuses = settings_statuses(&snapshot.settings);
    let ok_count = settings_statuses
        .iter()
        .filter(|status| status.state == SharedString::from("ok"))
        .count();
    let warn_count = settings_statuses
        .iter()
        .filter(|status| status.state == SharedString::from("warn"))
        .count();
    let error_count = settings_statuses
        .iter()
        .filter(|status| status.state == SharedString::from("error"))
        .count()
        + usize::from(snapshot.task_status.label == "Action failed");

    [
        (
            if snapshot.task_status.running {
                ">"
            } else {
                "o"
            },
            localization::text(
                language,
                if snapshot.task_status.running {
                    "Running"
                } else {
                    "Ready"
                },
                if snapshot.task_status.running {
                    "运行中"
                } else {
                    "就绪"
                },
            ),
            if snapshot.task_status.running {
                "running"
            } else {
                "ok"
            },
        ),
        (
            "o",
            shared(format!(
                "{} {ok_count}",
                localization::text(language, "Success", "成功")
            )),
            "ok",
        ),
        (
            "!",
            shared(format!(
                "{} {warn_count}",
                localization::text(language, "Warning", "警告")
            )),
            if warn_count == 0 { "muted" } else { "warn" },
        ),
        (
            "x",
            shared(format!(
                "{} {error_count}",
                localization::text(language, "Error", "错误")
            )),
            if error_count == 0 { "muted" } else { "error" },
        ),
    ]
    .into_iter()
    .map(|(icon, text, state)| {
        let icon_image = media::status_icon(state);
        HeaderStatusData {
            icon: shared(icon),
            icon_image: icon_image.clone().unwrap_or_default(),
            has_icon_image: icon_image.is_some(),
            text,
            state: shared(state),
        }
    })
    .collect()
}

pub(super) fn selected_page_title(page: HubPage, language: HubLanguage) -> SharedString {
    localization::page_title(page, language)
}

pub(super) fn selected_page_id(page: HubPage) -> SharedString {
    shared(page.id())
}

pub(super) fn selected_page_subtitle(page: HubPage, language: HubLanguage) -> SharedString {
    localization::page_subtitle(page, language)
}

pub(super) fn project_filter_label(
    filter: ProjectFilterMode,
    language: HubLanguage,
) -> SharedString {
    localization::project_filter_label(filter, language)
}

pub(super) fn project_sort_label(sort: ProjectSortMode, language: HubLanguage) -> SharedString {
    localization::project_sort_label(sort, language)
}

pub(super) fn project_view_mode_id(mode: ProjectViewMode) -> SharedString {
    shared(mode.id())
}

fn relative_time(now_ms: u64, then_ms: u64, language: HubLanguage) -> String {
    let elapsed = now_ms.saturating_sub(then_ms);
    if elapsed < MILLIS_PER_MINUTE {
        return localization::text(language, "just now", "刚刚").to_string();
    }
    if elapsed < MILLIS_PER_HOUR {
        let value = elapsed / MILLIS_PER_MINUTE;
        return match language {
            HubLanguage::English => format!("{value}m ago"),
            HubLanguage::Chinese => format!("{value} 分钟前"),
        };
    }
    if elapsed < MILLIS_PER_DAY {
        let value = elapsed / MILLIS_PER_HOUR;
        return match language {
            HubLanguage::English => format!("{value}h ago"),
            HubLanguage::Chinese => format!("{value} 小时前"),
        };
    }
    if elapsed < MILLIS_PER_WEEK {
        let value = elapsed / MILLIS_PER_DAY;
        return match language {
            HubLanguage::English => format!("{value}d ago"),
            HubLanguage::Chinese => format!("{value} 天前"),
        };
    }
    let value = elapsed / MILLIS_PER_WEEK;
    match language {
        HubLanguage::English => format!("{value}w ago"),
        HubLanguage::Chinese => format!("{value} 周前"),
    }
}

fn path_text(path: &Path, language: HubLanguage) -> String {
    if path.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置").to_string();
    }
    path.to_string_lossy().into_owned()
}

fn selected_engine<'a>(
    engines: &'a [SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> Option<&'a SourceEngineInstall> {
    active_engine_id
        .and_then(|id| engines.iter().find(|engine| engine.id == id))
        .or_else(|| engines.first())
}

enum BuildContextEngine<'a> {
    Engine(&'a SourceEngineInstall),
    NoSelectedProject,
    SelectedProjectUnbound,
    SelectedProjectUnavailable,
}

fn build_context_engine(snapshot: &HubSnapshot) -> Option<&SourceEngineInstall> {
    match selected_project_engine_context(snapshot) {
        BuildContextEngine::Engine(engine) => Some(engine),
        BuildContextEngine::NoSelectedProject => {
            selected_engine(&snapshot.engines, snapshot.active_engine_id.as_deref())
        }
        BuildContextEngine::SelectedProjectUnbound
        | BuildContextEngine::SelectedProjectUnavailable => None,
    }
}

fn selected_project_engine_context(snapshot: &HubSnapshot) -> BuildContextEngine<'_> {
    let Some(project) = selected_project_for_context(snapshot) else {
        return BuildContextEngine::NoSelectedProject;
    };
    let Some(engine_id) = metadata_for_path(&snapshot.project_metadata, &project.path)
        .and_then(|metadata| metadata.engine_id.as_deref())
    else {
        return BuildContextEngine::SelectedProjectUnbound;
    };
    snapshot
        .engines
        .iter()
        .find(|engine| engine.id == engine_id)
        .map(BuildContextEngine::Engine)
        .unwrap_or(BuildContextEngine::SelectedProjectUnavailable)
}

fn selected_project_for_context(snapshot: &HubSnapshot) -> Option<&RecentProject> {
    let selected_path = snapshot.selected_project_path.as_ref()?;
    snapshot
        .recent_projects
        .iter()
        .find(|project| project_paths_match(&project.path, selected_path))
}

fn executable_status(label: &str, value: &str, language: HubLanguage) -> SettingStatusData {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return setting_status(
            label,
            localization::text(language, "Not configured", "未配置"),
            "error",
        );
    }
    if command_looks_like_path(trimmed) {
        let path = Path::new(trimmed);
        return if path.exists() {
            setting_status(
                label,
                localization::text(language, "Path exists", "路径存在"),
                "ok",
            )
        } else {
            setting_status(
                label,
                localization::text(language, "Path not found", "路径不存在"),
                "error",
            )
        };
    }
    setting_status(
        label,
        localization::text(language, "Resolved from PATH", "从 PATH 解析"),
        "info",
    )
}

fn directory_status(
    label: &SharedString,
    path: &Path,
    exists_detail: &SharedString,
    missing_detail: &SharedString,
    empty_detail: &SharedString,
) -> SettingStatusData {
    if path.as_os_str().is_empty() {
        return setting_status(label.clone(), empty_detail.clone(), "error");
    }
    if path.exists() {
        return setting_status(label.clone(), exists_detail.clone(), "ok");
    }
    setting_status(label.clone(), missing_detail.clone(), "warn")
}

fn command_looks_like_path(value: &str) -> bool {
    value.contains('\\') || value.contains('/') || Path::new(value).is_absolute()
}

fn setting_status(
    label: impl Into<SharedString>,
    detail: SharedString,
    state: &str,
) -> SettingStatusData {
    SettingStatusData {
        label: label.into(),
        detail,
        state: shared(state),
    }
}

fn shared(value: impl Into<String>) -> SharedString {
    SharedString::from(value.into())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::projects::RecentProject;
    use crate::state::{
        HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode,
        TaskStatus,
    };

    use super::*;

    #[test]
    fn view_model_filters_project_cards_and_recent_rows() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: "stellar".to_string(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Elysium", "E:/Projects/Elysium", 10),
                RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 20),
            ],
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let cards = project_cards(&snapshot);
        let rows = dashboard_project_rows(&snapshot);

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, SharedString::from("Stellar Outpost"));
        assert_eq!(cards[0].version, SharedString::from("Unbound"));
        assert!(cards[0].has_cover);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].project_path,
            SharedString::from("E:/Projects/StellarOutpost")
        );
        assert!(rows[0].has_cover);
    }

    #[test]
    fn project_rows_mark_selected_project_path() {
        let selected_path = PathBuf::from("E:\\Projects\\StellarOutpost\\");
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(selected_path),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Elysium", "E:/Projects/Elysium", 10),
                RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 20),
            ],
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let cards = project_cards(&snapshot);
        let rows = dashboard_project_rows(&snapshot);

        assert!(cards[0].selected);
        assert!(!cards[1].selected);
        assert!(rows[0].selected);
        assert!(!rows[1].selected);
    }

    #[test]
    fn view_model_limits_visible_project_cards() {
        let projects = (0..14)
            .map(|index| {
                RecentProject::new(
                    format!("Project {index}"),
                    PathBuf::from(format!("E:/Projects/{index}")),
                    index,
                )
            })
            .collect();
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: projects,
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        assert_eq!(project_cards(&snapshot).len(), PROJECT_CARD_LIMIT);
        assert_eq!(dashboard_project_rows(&snapshot).len(), RECENT_ROW_LIMIT);
        assert_eq!(project_browser_rows(&snapshot).len(), 14);
    }

    #[test]
    fn project_create_requires_available_source_engine() {
        let mut snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::NewProject,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![SourceEngineInstall {
                id: "local-source".to_string(),
                display_name: "Local Source".to_string(),
                source_dir: PathBuf::from("E:/source"),
                output_dir: PathBuf::from("E:/out"),
                last_build_unix_ms: None,
                build_history: Vec::new(),
            }],
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        assert!(!project_create_enabled(&snapshot));

        snapshot.new_project_engine_id = Some("missing-source".to_string());
        assert!(!project_create_enabled(&snapshot));
        assert_eq!(
            project_create_engine_label(&snapshot),
            SharedString::from("Selected Source Engine is unavailable")
        );

        snapshot.new_project_engine_id = Some("local-source".to_string());
        assert!(project_create_enabled(&snapshot));
        assert_eq!(
            project_create_engine_label(&snapshot),
            SharedString::from("Local Source")
        );
    }

    #[test]
    fn project_browser_prefers_pinned_projects_and_falls_back_to_recent() {
        let mut metadata = crate::projects::ProjectMetadataMap::new();
        metadata.insert(
            crate::projects::project_metadata_key("E:/Projects/Pinned"),
            crate::projects::ProjectMetadata {
                pinned: true,
                ..crate::projects::ProjectMetadata::default()
            },
        );
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::List,
            project_subpage: ProjectSubpage::ProjectBrowser,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Recent", "E:/Projects/Recent", 50),
                RecentProject::new("Pinned", "E:/Projects/Pinned", 10),
            ],
            project_metadata: metadata,
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let pinned_rows = project_browser_rows(&snapshot);
        let pinned_dashboard_rows = dashboard_project_rows(&snapshot);
        let mut fallback = snapshot.clone();
        fallback.project_metadata.clear();
        let fallback_rows = project_browser_rows(&fallback);
        let fallback_dashboard_rows = dashboard_project_rows(&fallback);

        assert_eq!(pinned_rows.len(), 1);
        assert_eq!(pinned_dashboard_rows.len(), 1);
        assert_eq!(pinned_rows[0].title, SharedString::from("Pinned"));
        assert_eq!(pinned_dashboard_rows[0].title, SharedString::from("Pinned"));
        assert_eq!(
            dashboard_project_title(&snapshot, HubLanguage::English),
            "Pinned Projects"
        );
        assert_eq!(
            dashboard_project_title(&snapshot, HubLanguage::Chinese),
            "置顶项目"
        );
        assert_eq!(fallback_dashboard_rows.len(), 2);
        assert_eq!(fallback_rows[0].title, SharedString::from("Recent"));
        assert_eq!(
            dashboard_project_title(&fallback, HubLanguage::English),
            "Recent Projects"
        );
        assert_eq!(
            dashboard_project_title(&fallback, HubLanguage::Chinese),
            "最近项目"
        );
    }

    #[test]
    fn project_rows_use_bound_engine_metadata_without_active_fallback() {
        let mut metadata = crate::projects::ProjectMetadataMap::new();
        metadata.insert(
            crate::projects::project_metadata_key("E:/Projects/Bound"),
            crate::projects::ProjectMetadata {
                engine_id: Some("bound".to_string()),
                ..crate::projects::ProjectMetadata::default()
            },
        );
        metadata.insert(
            crate::projects::project_metadata_key("E:/Projects/Stale"),
            crate::projects::ProjectMetadata {
                engine_id: Some("missing-source".to_string()),
                ..crate::projects::ProjectMetadata::default()
            },
        );
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::Name,
            project_view_mode: ProjectViewMode::List,
            project_subpage: ProjectSubpage::ProjectBrowser,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/Projects/Unbound")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Unbound", "E:/Projects/Unbound", 50),
                RecentProject::new("Stale", "E:/Projects/Stale", 25),
                RecentProject::new("Bound", "E:/Projects/Bound", 10),
            ],
            project_metadata: metadata,
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![
                SourceEngineInstall {
                    id: "active".to_string(),
                    display_name: "Active Source".to_string(),
                    source_dir: PathBuf::from("E:/active"),
                    output_dir: PathBuf::from("E:/out-active"),
                    last_build_unix_ms: None,
                    build_history: Vec::new(),
                },
                SourceEngineInstall {
                    id: "bound".to_string(),
                    display_name: "Bound Source".to_string(),
                    source_dir: PathBuf::from("E:/bound"),
                    output_dir: PathBuf::from("E:/out-bound"),
                    last_build_unix_ms: None,
                    build_history: Vec::new(),
                },
            ],
            active_engine_id: Some("active".to_string()),
            settings: HubSettings::default(),
        };

        let rows = project_browser_rows(&snapshot);
        let bound = rows
            .iter()
            .find(|row| row.title == SharedString::from("Bound"))
            .expect("bound project row should be projected");
        let unbound = rows
            .iter()
            .find(|row| row.title == SharedString::from("Unbound"))
            .expect("unbound project row should be projected");
        let stale = rows
            .iter()
            .find(|row| row.title == SharedString::from("Stale"))
            .expect("stale project row should be projected");
        let detail = project_detail(&snapshot);

        assert_eq!(bound.engine_id, SharedString::from("bound"));
        assert_eq!(bound.engine_label, SharedString::from("Bound Source"));
        assert_ne!(bound.version, SharedString::from("Unbound"));
        assert_eq!(stale.engine_id, SharedString::from("missing-source"));
        assert_eq!(
            stale.engine_label,
            SharedString::from("Source Engine unavailable")
        );
        assert_eq!(stale.version, SharedString::from("Unavailable"));
        assert_eq!(unbound.engine_id, SharedString::default());
        assert_eq!(unbound.engine_label, SharedString::from("No Source Engine"));
        assert_eq!(unbound.version, SharedString::from("Unbound"));
        assert_eq!(detail.engine_id, SharedString::default());
        assert_eq!(detail.engine_label, SharedString::from("No Source Engine"));
        assert_eq!(detail.version, SharedString::from("Unbound"));
        assert_eq!(detail.status, SharedString::from("Missing"));

        let mut chinese_snapshot = snapshot.clone();
        chinese_snapshot.settings.language = HubLanguage::Chinese;
        let chinese_rows = project_browser_rows(&chinese_snapshot);
        let chinese_stale = chinese_rows
            .iter()
            .find(|row| row.title == SharedString::from("Stale"))
            .expect("stale project row should be projected in Chinese");
        let chinese_unbound = chinese_rows
            .iter()
            .find(|row| row.title == SharedString::from("Unbound"))
            .expect("unbound project row should be projected in Chinese");
        assert_eq!(
            chinese_stale.engine_label,
            SharedString::from("源码引擎不可用")
        );
        assert_eq!(chinese_stale.version, SharedString::from("不可用"));
        assert_eq!(
            chinese_unbound.engine_label,
            SharedString::from("无源码引擎")
        );
        assert_eq!(chinese_unbound.version, SharedString::from("未绑定"));
        let chinese_detail = project_detail(&chinese_snapshot);
        assert_eq!(chinese_detail.status, SharedString::from("缺失"));
    }

    #[test]
    fn project_detail_can_build_requires_available_bound_source_engine() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-can-build-{}-{}",
            std::process::id(),
            now_unix_ms()
        ));
        let bound_project = root.join("bound");
        let unbound_project = root.join("unbound");
        let stale_project = root.join("stale");
        for project in [&bound_project, &unbound_project, &stale_project] {
            fs::create_dir_all(project).expect("test project root should be created");
            fs::write(project.join("zircon-project.toml"), "name = \"Demo\"\n")
                .expect("test project manifest should be created");
        }
        let mut metadata = crate::projects::ProjectMetadataMap::new();
        metadata.insert(
            crate::projects::project_metadata_key(&bound_project),
            crate::projects::ProjectMetadata {
                engine_id: Some("bound".to_string()),
                ..crate::projects::ProjectMetadata::default()
            },
        );
        metadata.insert(
            crate::projects::project_metadata_key(&stale_project),
            crate::projects::ProjectMetadata {
                engine_id: Some("missing-source".to_string()),
                ..crate::projects::ProjectMetadata::default()
            },
        );

        let snapshot_for = |selected_project_path: PathBuf| HubSnapshot {
            selected_page: HubPage::Builds,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::Name,
            project_view_mode: ProjectViewMode::List,
            project_subpage: ProjectSubpage::ProjectDetail,
            search_query: String::new(),
            selected_project_path: Some(selected_project_path),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: root.clone(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Bound", bound_project.clone(), 30),
                RecentProject::new("Unbound", unbound_project.clone(), 20),
                RecentProject::new("Stale", stale_project.clone(), 10),
            ],
            project_metadata: metadata.clone(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![SourceEngineInstall {
                id: "bound".to_string(),
                display_name: "Bound Source".to_string(),
                source_dir: root.join("source"),
                output_dir: root.join("out"),
                last_build_unix_ms: None,
                build_history: Vec::new(),
            }],
            active_engine_id: Some("active-source-must-not-be-used".to_string()),
            settings: HubSettings::default(),
        };

        let bound_detail = project_detail(&snapshot_for(bound_project.clone()));
        assert!(bound_detail.can_open);
        assert!(bound_detail.can_build);
        assert_eq!(
            bound_detail.engine_label,
            SharedString::from("Bound Source")
        );

        let unbound_detail = project_detail(&snapshot_for(unbound_project.clone()));
        assert!(unbound_detail.can_open);
        assert!(!unbound_detail.can_build);
        assert_eq!(
            unbound_detail.engine_label,
            SharedString::from("No Source Engine")
        );

        let stale_detail = project_detail(&snapshot_for(stale_project.clone()));
        assert!(stale_detail.can_open);
        assert!(!stale_detail.can_build);
        assert_eq!(
            stale_detail.engine_label,
            SharedString::from("Source Engine unavailable")
        );

        let row_snapshot = snapshot_for(bound_project.clone());
        let rows = project_browser_rows(&row_snapshot);
        let bound_row = rows
            .iter()
            .find(|row| row.title == SharedString::from("Bound"))
            .expect("bound row should be projected");
        let unbound_row = rows
            .iter()
            .find(|row| row.title == SharedString::from("Unbound"))
            .expect("unbound row should be projected");
        let stale_row = rows
            .iter()
            .find(|row| row.title == SharedString::from("Stale"))
            .expect("stale row should be projected");
        assert!(bound_row.can_build);
        assert!(!unbound_row.can_build);
        assert!(!stale_row.can_build);

        fs::remove_dir_all(root).expect("test project roots should be removed");
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
        assert_eq!(
            relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::Chinese),
            "3 天前"
        );
    }

    #[test]
    fn settings_statuses_classify_commands_and_missing_source_paths() {
        let mut settings = HubSettings {
            python_path: "python".to_string(),
            cargo_path: "C:/missing/cargo.exe".to_string(),
            ..HubSettings::default()
        };
        settings.default_source_dir = PathBuf::from("E:/missing/zircon/source");

        let statuses = settings_statuses(&settings);

        assert_eq!(statuses[0].detail, SharedString::from("Resolved from PATH"));
        assert_eq!(statuses[0].state, SharedString::from("info"));
        assert_eq!(statuses[1].detail, SharedString::from("Path not found"));
        assert_eq!(statuses[1].state, SharedString::from("error"));
        assert_eq!(
            statuses[4].detail,
            SharedString::from("Missing source checkout")
        );
        assert_eq!(statuses[4].state, SharedString::from("warn"));
    }

    #[test]
    fn header_statuses_reflect_task_and_configuration_counts() {
        let root =
            std::env::temp_dir().join(format!("zircon-hub-header-status-test-{}", now_unix_ms()));
        let project_dir = root.join("projects");
        let source_dir = root.join("source");
        let output_dir = root.join("out");
        let device_dir = root.join("device");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&device_dir).unwrap();
        let mut settings = HubSettings {
            cargo_path: "C:/missing/cargo.exe".to_string(),
            ..HubSettings::default()
        };
        settings.default_project_dir = project_dir;
        settings.default_source_dir = source_dir;
        settings.default_build_output_dir = output_dir;
        settings.default_device_install_dir = device_dir;
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: project_dir.clone(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::running("Building", "Running build command"),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings,
        };

        let statuses = header_statuses(&snapshot);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(statuses[0].text, SharedString::from("Running"));
        assert_eq!(statuses[0].state, SharedString::from("running"));
        assert_eq!(statuses[2].text, SharedString::from("Warning 2"));
        assert_eq!(statuses[2].state, SharedString::from("warn"));
        assert_eq!(statuses[3].text, SharedString::from("Error 1"));
        assert_eq!(statuses[3].state, SharedString::from("error"));
    }

    #[test]
    fn source_engine_rows_mark_active_engine() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Editor,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![
                SourceEngineInstall {
                    id: "first".to_string(),
                    display_name: "First".to_string(),
                    source_dir: PathBuf::from("E:/first"),
                    output_dir: PathBuf::from("E:/out-first"),
                    last_build_unix_ms: None,
                    build_history: Vec::new(),
                },
                SourceEngineInstall {
                    id: "second".to_string(),
                    display_name: "Second".to_string(),
                    source_dir: PathBuf::from("E:/second"),
                    output_dir: PathBuf::from("E:/out-second"),
                    last_build_unix_ms: Some(7),
                    build_history: Vec::new(),
                },
            ],
            active_engine_id: Some("second".to_string()),
            settings: HubSettings::default(),
        };

        let rows = source_engine_rows(&snapshot);

        assert!(!rows[0].active);
        assert!(rows[1].active);
        assert_eq!(rows[1].status, SharedString::from("Active"));
    }

    #[test]
    fn source_engine_data_prefers_selected_project_bound_engine() {
        let selected_project = PathBuf::from("E:/Projects/Game");
        let mut project_metadata = crate::projects::ProjectMetadataMap::new();
        project_metadata.insert(
            crate::projects::project_metadata_key(&selected_project),
            crate::projects::ProjectMetadata {
                engine_id: Some("selected-source".to_string()),
                ..crate::projects::ProjectMetadata::default()
            },
        );
        let snapshot = HubSnapshot {
            selected_page: HubPage::Builds,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:\\Projects\\Game\\")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![RecentProject::new("Game", selected_project, 11)],
            project_metadata,
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![
                SourceEngineInstall {
                    id: "active-source".to_string(),
                    display_name: "Active".to_string(),
                    source_dir: PathBuf::from("E:/active"),
                    output_dir: PathBuf::from("E:/active-out"),
                    last_build_unix_ms: Some(5),
                    build_history: Vec::new(),
                },
                SourceEngineInstall {
                    id: "selected-source".to_string(),
                    display_name: "Selected".to_string(),
                    source_dir: PathBuf::from("E:/selected"),
                    output_dir: PathBuf::from("E:/selected-out"),
                    last_build_unix_ms: Some(9),
                    build_history: Vec::new(),
                },
            ],
            active_engine_id: Some("active-source".to_string()),
            settings: HubSettings::default(),
        };

        let source = source_engine_data(&snapshot);

        assert_eq!(source.title, SharedString::from("Selected"));
        assert_eq!(source.source_path, SharedString::from("E:/selected"));
        assert_eq!(source.output_path, SharedString::from("E:/selected-out"));
    }

    #[test]
    fn source_engine_data_does_not_fallback_when_selected_project_is_unbound() {
        let selected_project = PathBuf::from("E:/Projects/Game");
        let snapshot = HubSnapshot {
            selected_page: HubPage::Builds,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:\\Projects\\Game\\")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![RecentProject::new("Game", selected_project, 11)],
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![SourceEngineInstall {
                id: "active-source".to_string(),
                display_name: "Active".to_string(),
                source_dir: PathBuf::from("E:/active"),
                output_dir: PathBuf::from("E:/active-out"),
                last_build_unix_ms: Some(5),
                build_history: vec![crate::engines::SourceBuildRecord {
                    finished_unix_ms: 5,
                    status: "success".to_string(),
                    profile: "debug".to_string(),
                    jobs: Some(2),
                    output_dir: PathBuf::from("E:/active-out"),
                    detail: "active history".to_string(),
                }],
            }],
            active_engine_id: Some("active-source".to_string()),
            settings: HubSettings::default(),
        };

        let source = source_engine_data(&snapshot);
        let rows = source_build_history_rows(&snapshot);

        assert_eq!(source.title, SharedString::from("No Source Engine"));
        assert_eq!(source.status, SharedString::from("Bind project engine"));
        assert!(rows.is_empty());
    }

    #[test]
    fn build_history_rows_use_active_engine_records() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Editor,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![SourceEngineInstall {
                id: "active".to_string(),
                display_name: "Active".to_string(),
                source_dir: PathBuf::from("E:/active"),
                output_dir: PathBuf::from("E:/out"),
                last_build_unix_ms: Some(9),
                build_history: vec![crate::engines::SourceBuildRecord {
                    finished_unix_ms: 9,
                    status: "success".to_string(),
                    profile: "debug".to_string(),
                    jobs: Some(4),
                    output_dir: PathBuf::from("E:/out"),
                    detail: "ok".to_string(),
                }],
            }],
            active_engine_id: Some("active".to_string()),
            settings: HubSettings::default(),
        };

        let rows = source_build_history_rows(&snapshot);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, SharedString::from("Success"));
        assert_eq!(rows[0].profile, SharedString::from("debug / 4 jobs"));
        assert!(rows[0].success);
    }

    #[test]
    fn build_history_rows_prefer_selected_project_bound_engine_records() {
        let selected_project = PathBuf::from("E:/Projects/Game");
        let mut project_metadata = crate::projects::ProjectMetadataMap::new();
        project_metadata.insert(
            crate::projects::project_metadata_key(&selected_project),
            crate::projects::ProjectMetadata {
                engine_id: Some("selected-source".to_string()),
                ..crate::projects::ProjectMetadata::default()
            },
        );
        let snapshot = HubSnapshot {
            selected_page: HubPage::Builds,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:\\Projects\\Game\\")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![RecentProject::new("Game", selected_project, 11)],
            project_metadata,
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: vec![
                SourceEngineInstall {
                    id: "active-source".to_string(),
                    display_name: "Active".to_string(),
                    source_dir: PathBuf::from("E:/active"),
                    output_dir: PathBuf::from("E:/active-out"),
                    last_build_unix_ms: Some(5),
                    build_history: vec![crate::engines::SourceBuildRecord {
                        finished_unix_ms: 5,
                        status: "failed".to_string(),
                        profile: "debug".to_string(),
                        jobs: Some(2),
                        output_dir: PathBuf::from("E:/active-out"),
                        detail: "active history".to_string(),
                    }],
                },
                SourceEngineInstall {
                    id: "selected-source".to_string(),
                    display_name: "Selected".to_string(),
                    source_dir: PathBuf::from("E:/selected"),
                    output_dir: PathBuf::from("E:/selected-out"),
                    last_build_unix_ms: Some(9),
                    build_history: vec![crate::engines::SourceBuildRecord {
                        finished_unix_ms: 9,
                        status: "success".to_string(),
                        profile: "release".to_string(),
                        jobs: Some(8),
                        output_dir: PathBuf::from("E:/selected-out"),
                        detail: "selected history".to_string(),
                    }],
                },
            ],
            active_engine_id: Some("active-source".to_string()),
            settings: HubSettings::default(),
        };

        let rows = source_build_history_rows(&snapshot);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].detail, SharedString::from("selected history"));
        assert_eq!(rows[0].output_path, SharedString::from("E:/selected-out"));
        assert_eq!(rows[0].profile, SharedString::from("release / 8 jobs"));
        assert!(rows[0].success);
    }
}
