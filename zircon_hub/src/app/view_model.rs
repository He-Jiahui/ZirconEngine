use std::path::Path;

use slint::{ModelRc, SharedString, VecModel};

use crate::engines::SourceEngineInstall;
use crate::projects::{now_unix_ms, RecentProject};
use crate::settings::HubSettings;
use crate::state::{HubPage, HubSnapshot, ProjectSortMode, ProjectViewMode};

use super::quick_action::HubQuickAction;
use super::{
    NavItemData, ProjectCardData, QuickActionData, RecentProjectRowData, SourceEngineData,
};

const PROJECT_CARD_LIMIT: usize = 4;
const RECENT_ROW_LIMIT: usize = 8;
const MILLIS_PER_MINUTE: u64 = 60_000;
const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;
const MILLIS_PER_WEEK: u64 = 7 * MILLIS_PER_DAY;

pub(super) fn model_from<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items))
}

pub(super) fn navigation_items(selected_page: HubPage) -> Vec<NavItemData> {
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
    .map(|(page, icon)| NavItemData {
        id: shared(page.id()),
        title: shared(page.title()),
        icon: shared(icon),
        active: page == selected_page,
    })
    .collect()
}

pub(super) fn project_cards(snapshot: &HubSnapshot) -> Vec<ProjectCardData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(PROJECT_CARD_LIMIT)
        .enumerate()
        .map(|(index, project)| project_card(index, &project))
        .collect()
}

pub(super) fn recent_project_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(RECENT_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project))
        .collect()
}

pub(super) fn quick_actions() -> Vec<QuickActionData> {
    vec![
        QuickActionData {
            id: shared(HubQuickAction::BuildProject.id()),
            icon: shared("/>"),
            title: shared("Build Project"),
            detail: shared("Build the configured source editor/runtime payload"),
            enabled: true,
        },
        QuickActionData {
            id: shared(HubQuickAction::InstallToDevice.id()),
            icon: shared("[]"),
            title: shared("Install to Device"),
            detail: shared("Deploy your project to a connected device"),
            enabled: false,
        },
        QuickActionData {
            id: shared(HubQuickAction::PackageProject.id()),
            icon: shared("{}"),
            title: shared("Package Project"),
            detail: shared("Create a distributable package"),
            enabled: false,
        },
        QuickActionData {
            id: shared(HubQuickAction::OpenEditor.id()),
            icon: shared("<>"),
            title: shared("Open in Editor"),
            detail: shared("Launch the editor without selecting a project"),
            enabled: true,
        },
    ]
}

pub(super) fn source_engine_data(
    engines: &[SourceEngineInstall],
    settings: &HubSettings,
) -> SourceEngineData {
    let (title, source_path, output_path, last_build) = engines.first().map_or_else(
        || {
            (
                "No Source Engine".to_string(),
                path_text(&settings.default_source_dir),
                path_text(&settings.default_build_output_dir),
                "Not built yet".to_string(),
            )
        },
        |engine| {
            (
                engine.display_name.clone(),
                path_text(&engine.source_dir),
                path_text(&engine.output_dir),
                engine
                    .last_build_unix_ms
                    .map(|value| format!("Built {}", relative_time(now_unix_ms(), value)))
                    .unwrap_or_else(|| "Not built yet".to_string()),
            )
        },
    );

    SourceEngineData {
        title: shared(title),
        version: shared(format!("Zircon Engine {}", env!("CARGO_PKG_VERSION"))),
        source_path: shared(source_path),
        output_path: shared(output_path),
        last_build: shared(last_build),
        status: shared(if engines.is_empty() {
            "Configure source"
        } else {
            "Source registered"
        }),
        build_profile: shared(settings.build_profile.as_mode()),
        jobs: shared(settings.jobs.to_string()),
    }
}

pub(super) fn selected_page_title(page: HubPage) -> SharedString {
    shared(page.title())
}

pub(super) fn selected_page_id(page: HubPage) -> SharedString {
    shared(page.id())
}

pub(super) fn selected_page_subtitle(page: HubPage) -> SharedString {
    shared(page.subtitle())
}

pub(super) fn project_sort_label(sort: ProjectSortMode) -> SharedString {
    shared(sort.label())
}

pub(super) fn project_view_mode_id(mode: ProjectViewMode) -> SharedString {
    shared(mode.id())
}

fn project_card(index: usize, project: &RecentProject) -> ProjectCardData {
    ProjectCardData {
        title: shared(display_name(project)),
        project_path: shared(path_text(&project.path)),
        modified: shared(relative_time(now_unix_ms(), project.last_opened_unix_ms)),
        version: shared(engine_version_for_index(index)),
        platform: shared(platform_label()),
        accent: index as i32,
    }
}

fn recent_project_row(index: usize, project: &RecentProject) -> RecentProjectRowData {
    RecentProjectRowData {
        title: shared(display_name(project)),
        project_path: shared(path_text(&project.path)),
        modified: shared(relative_time(now_unix_ms(), project.last_opened_unix_ms)),
        version: shared(engine_version_for_index(index)),
        accent: index as i32,
    }
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

fn path_text(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        return "Not configured".to_string();
    }
    path.to_string_lossy().into_owned()
}

fn relative_time(now_ms: u64, then_ms: u64) -> String {
    let elapsed = now_ms.saturating_sub(then_ms);
    if elapsed < MILLIS_PER_MINUTE {
        return "just now".to_string();
    }
    if elapsed < MILLIS_PER_HOUR {
        return format!("{}m ago", elapsed / MILLIS_PER_MINUTE);
    }
    if elapsed < MILLIS_PER_DAY {
        return format!("{}h ago", elapsed / MILLIS_PER_HOUR);
    }
    if elapsed < MILLIS_PER_WEEK {
        return format!("{}d ago", elapsed / MILLIS_PER_DAY);
    }
    format!("{}w ago", elapsed / MILLIS_PER_WEEK)
}

fn engine_version_for_index(index: usize) -> &'static str {
    match index % 4 {
        0 | 1 => "0.1.0",
        2 => "0.1.x",
        _ => "source",
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

fn shared(value: impl Into<String>) -> SharedString {
    SharedString::from(value.into())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::state::{HubSnapshot, ProjectSortMode, ProjectViewMode, TaskStatus};

    use super::*;

    #[test]
    fn view_model_filters_project_cards_and_recent_rows() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: "stellar".to_string(),
            task_status: TaskStatus::idle(),
            recent_projects: vec![
                RecentProject::new("Elysium", "E:/Projects/Elysium", 10),
                RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 20),
            ],
            engines: Vec::new(),
            settings: HubSettings::default(),
        };

        let cards = project_cards(&snapshot);
        let rows = recent_project_rows(&snapshot);

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, SharedString::from("Stellar Outpost"));
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].project_path,
            SharedString::from("E:/Projects/StellarOutpost")
        );
    }

    #[test]
    fn view_model_limits_visible_project_cards() {
        let projects = (0..6)
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
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            task_status: TaskStatus::idle(),
            recent_projects: projects,
            engines: Vec::new(),
            settings: HubSettings::default(),
        };

        assert_eq!(project_cards(&snapshot).len(), PROJECT_CARD_LIMIT);
        assert_eq!(recent_project_rows(&snapshot).len(), 6);
    }

    #[test]
    fn relative_time_uses_compact_labels() {
        let now = 10 * MILLIS_PER_DAY;

        assert_eq!(relative_time(now, now), "just now");
        assert_eq!(relative_time(now, now - (2 * MILLIS_PER_HOUR)), "2h ago");
        assert_eq!(relative_time(now, now - (3 * MILLIS_PER_DAY)), "3d ago");
    }
}
