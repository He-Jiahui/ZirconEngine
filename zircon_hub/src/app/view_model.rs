use std::path::Path;

use slint::{ModelRc, SharedString, VecModel};

use crate::engines::SourceEngineInstall;
use crate::projects::{now_unix_ms, RecentProject};
use crate::settings::{HubLanguage, HubSettings};
use crate::state::{HubPage, HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectViewMode};

use super::localization;
use super::quick_action::HubQuickAction;
use super::{
    AssetData, CloudServiceData, CloudSummaryData, HeaderStatusData, LearnData, NavItemData,
    PluginData, ProjectCardData, QuickActionData, RecentProjectRowData, SettingStatusData,
    SourceBuildHistoryRowData, SourceEngineData, SourceEngineRowData, TeamData, TeamMemberData,
    UiTextData,
};

mod assets;
mod cloud;
mod learn;
mod media;
mod plugins;
mod team;

const PROJECT_CARD_LIMIT: usize = 12;
const PROJECT_LIST_ROW_LIMIT: usize = 4;
const RECENT_ROW_LIMIT: usize = 8;
const BUILD_HISTORY_ROW_LIMIT: usize = 5;
const MILLIS_PER_MINUTE: u64 = 60_000;
const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;
const MILLIS_PER_WEEK: u64 = 7 * MILLIS_PER_DAY;

pub(super) fn model_from<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items))
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

pub(super) fn project_cards(snapshot: &HubSnapshot) -> Vec<ProjectCardData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(PROJECT_CARD_LIMIT)
        .enumerate()
        .map(|(index, project)| project_card(index, &project, snapshot))
        .collect()
}

pub(super) fn recent_project_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(RECENT_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(super) fn project_list_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(PROJECT_LIST_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(super) fn quick_actions(language: HubLanguage) -> Vec<QuickActionData> {
    [
        (
            HubQuickAction::BuildProject,
            "/>",
            localization::text(language, "Build Project", "构建项目"),
            localization::text(
                language,
                "Build the configured source editor/runtime payload",
                "构建已配置的编辑器/运行时源码产物",
            ),
        ),
        (
            HubQuickAction::InstallToDevice,
            "[]",
            localization::text(language, "Install to Device", "安装到设备"),
            localization::text(
                language,
                "Copy the latest package into the configured device folder",
                "复制最新包到已配置的设备目录",
            ),
        ),
        (
            HubQuickAction::PackageProject,
            "{}",
            localization::text(language, "Package Project", "打包项目"),
            localization::text(
                language,
                "Stage the latest recent project into a package folder",
                "将最近项目暂存为本地包目录",
            ),
        ),
        (
            HubQuickAction::OpenEditor,
            "<>",
            localization::text(language, "Open in Editor", "在编辑器中打开"),
            localization::text(
                language,
                "Launch the editor without selecting a project",
                "不选择项目直接启动编辑器",
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
            enabled: true,
        }
    })
    .collect()
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

pub(super) fn source_engine_data(
    engines: &[SourceEngineInstall],
    settings: &HubSettings,
    active_engine_id: Option<&str>,
) -> SourceEngineData {
    let language = settings.language;
    let (title, source_path, output_path, last_build) = selected_engine(engines, active_engine_id)
        .map_or_else(
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
            if engines.is_empty() {
                "Configure source"
            } else {
                "Source registered"
            },
            if engines.is_empty() {
                "配置源码"
            } else {
                "源码已注册"
            },
        ),
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
    selected_engine(&snapshot.engines, snapshot.active_engine_id.as_deref())
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

fn project_card(index: usize, project: &RecentProject, snapshot: &HubSnapshot) -> ProjectCardData {
    let language = snapshot.settings.language;
    let cover_image = media::project_cover(index, project);
    let has_cover = cover_image.is_some();
    ProjectCardData {
        title: shared(display_name(project)),
        project_path: shared(path_text(&project.path, language)),
        modified: shared(relative_time(
            now_unix_ms(),
            project.last_opened_unix_ms,
            language,
        )),
        version: shared(engine_version_for_index(index)),
        platform: shared(platform_label()),
        cover_image: cover_image.unwrap_or_default(),
        has_cover,
        selected: project_is_selected(project, snapshot),
        accent: index as i32,
    }
}

fn recent_project_row(
    index: usize,
    project: &RecentProject,
    snapshot: &HubSnapshot,
) -> RecentProjectRowData {
    let language = snapshot.settings.language;
    let cover_image = media::project_cover(index, project);
    let has_cover = cover_image.is_some();
    RecentProjectRowData {
        title: shared(display_name(project)),
        project_path: shared(path_text(&project.path, language)),
        modified: shared(relative_time(
            now_unix_ms(),
            project.last_opened_unix_ms,
            language,
        )),
        version: shared(engine_version_for_index(index)),
        cover_image: cover_image.unwrap_or_default(),
        has_cover,
        selected: project_is_selected(project, snapshot),
        accent: index as i32,
    }
}

fn project_is_selected(project: &RecentProject, snapshot: &HubSnapshot) -> bool {
    snapshot
        .selected_project_path
        .as_ref()
        .is_some_and(|selected| selected == &project.path)
}

fn display_name(project: &RecentProject) -> String {
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

fn path_text(path: &Path, language: HubLanguage) -> String {
    if path.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置").to_string();
    }
    path.to_string_lossy().into_owned()
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

fn engine_version_for_index(index: usize) -> &'static str {
    match index % 5 {
        0 | 1 => "1.8.2",
        2 => "1.8.1",
        3 => "1.8.0",
        _ => "1.7.9",
    }
}

fn platform_label() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Desktop"
    }
}

fn selected_engine<'a>(
    engines: &'a [SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> Option<&'a SourceEngineInstall> {
    active_engine_id
        .and_then(|id| engines.iter().find(|engine| engine.id == id))
        .or_else(|| engines.first())
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

    use crate::state::{
        HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn view_model_filters_project_cards_and_recent_rows() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: "stellar".to_string(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Elysium", "E:/Projects/Elysium", 10),
                RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 20),
            ],
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let cards = project_cards(&snapshot);
        let rows = recent_project_rows(&snapshot);

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, SharedString::from("Stellar Outpost"));
        assert_eq!(cards[0].version, SharedString::from("1.8.2"));
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
        let selected_path = PathBuf::from("E:/Projects/StellarOutpost");
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            selected_project_path: Some(selected_path),
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Elysium", "E:/Projects/Elysium", 10),
                RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 20),
            ],
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let cards = project_cards(&snapshot);
        let rows = recent_project_rows(&snapshot);

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
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: projects,
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        assert_eq!(project_cards(&snapshot).len(), PROJECT_CARD_LIMIT);
        assert_eq!(project_list_rows(&snapshot).len(), PROJECT_LIST_ROW_LIMIT);
        assert_eq!(recent_project_rows(&snapshot).len(), RECENT_ROW_LIMIT);
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
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::running("Building", "Running build command"),
            recent_projects: Vec::new(),
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
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
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
    fn build_history_rows_use_active_engine_records() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Editor,
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
}
