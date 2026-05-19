use std::path::Path;

use slint::SharedString;

use crate::projects::{metadata_for_path, now_unix_ms, project_template_catalog, RecentProject};
use crate::settings::HubLanguage;
use crate::state::{HubSnapshot, ProjectSubpage};

use super::super::{
    ProjectCardData, ProjectDetailData, ProjectTemplateData, RecentProjectRowData,
    SourceEngineRowData,
};
use super::{
    localization, media, path_text, relative_time, shared, PROJECT_CARD_LIMIT,
    PROJECT_LIST_ROW_LIMIT, RECENT_ROW_LIMIT,
};

pub(in crate::app) fn project_cards(snapshot: &HubSnapshot) -> Vec<ProjectCardData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(PROJECT_CARD_LIMIT)
        .enumerate()
        .map(|(index, project)| project_card(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn recent_project_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(RECENT_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn project_list_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(PROJECT_LIST_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn project_browser_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    project_browser_projects(snapshot)
        .into_iter()
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn dashboard_project_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    project_browser_projects(snapshot)
        .into_iter()
        .take(RECENT_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn dashboard_project_title(
    snapshot: &HubSnapshot,
    language: HubLanguage,
) -> SharedString {
    if snapshot
        .filtered_recent_projects()
        .iter()
        .any(|project| project_is_pinned(project, snapshot))
    {
        localization::text(language, "Pinned Projects", "置顶项目")
    } else {
        localization::text(language, "Recent Projects", "最近项目")
    }
}

pub(in crate::app) fn project_templates(snapshot: &HubSnapshot) -> Vec<ProjectTemplateData> {
    project_template_catalog()
        .iter()
        .map(|template| ProjectTemplateData {
            id: shared(template.id),
            title: shared(template.title),
            category: shared(template.category),
            description: shared(template.description),
            enabled: template.enabled,
            selected: template.id == snapshot.selected_template_id,
        })
        .collect()
}

pub(in crate::app) fn project_create_enabled(snapshot: &HubSnapshot) -> bool {
    crate::projects::ProjectTemplate::from_enabled_id(&snapshot.selected_template_id).is_some()
        && snapshot.new_project_engine_id.is_some()
}

pub(in crate::app) fn project_create_template_label(snapshot: &HubSnapshot) -> SharedString {
    project_template_catalog()
        .iter()
        .find(|template| template.id == snapshot.selected_template_id)
        .map(|template| shared(template.title))
        .unwrap_or_else(|| {
            localization::text(
                snapshot.settings.language,
                "No template selected",
                "未选择模板",
            )
        })
}

pub(in crate::app) fn project_create_engine_label(snapshot: &HubSnapshot) -> SharedString {
    let Some(engine_id) = snapshot.new_project_engine_id.as_deref() else {
        return localization::text(
            snapshot.settings.language,
            "No Source Engine selected",
            "未选择源码引擎",
        );
    };

    snapshot
        .engines
        .iter()
        .find(|engine| engine.id == engine_id)
        .map(|engine| shared(&engine.display_name))
        .unwrap_or_else(|| {
            localization::text(
                snapshot.settings.language,
                "Selected Source Engine is unavailable",
                "已选源码引擎不可用",
            )
        })
}

pub(in crate::app) fn project_detail(snapshot: &HubSnapshot) -> ProjectDetailData {
    let project = selected_recent_project(snapshot)
        .or_else(|| project_browser_projects(snapshot).into_iter().next())
        .unwrap_or_default();
    let row = recent_project_row(0, &project, snapshot);
    ProjectDetailData {
        title: row.title,
        project_path: row.project_path,
        modified: row.modified,
        version: row.version,
        engine_id: row.engine_id,
        engine_label: row.engine_label,
        status: if row.missing {
            shared("Missing")
        } else if row.can_open {
            shared("Ready")
        } else {
            shared("Invalid")
        },
        cover_image: row.cover_image,
        has_cover: row.has_cover,
        selected: row.selected,
        pinned: row.pinned,
        missing: row.missing,
        can_open: row.can_open,
        can_delete: row.can_delete,
        pending_delete: snapshot
            .pending_delete_project_path
            .as_ref()
            .is_some_and(|path| shared_path_eq(path, &project.path)),
        accent: row.accent,
    }
}

pub(in crate::app) fn project_engine_rows(snapshot: &HubSnapshot) -> Vec<SourceEngineRowData> {
    let selected_project = selected_recent_project(snapshot);
    let selected_engine_id = match snapshot.project_subpage {
        ProjectSubpage::NewProject => snapshot.new_project_engine_id.as_deref(),
        ProjectSubpage::ProjectDetail => selected_project
            .as_ref()
            .and_then(|project| project_engine_id(project, snapshot)),
        _ => snapshot.active_engine_id.as_deref(),
    };
    source_engine_rows_for_selection(snapshot, selected_engine_id)
}

pub(in crate::app) fn project_subpage_id(subpage: ProjectSubpage) -> SharedString {
    shared(subpage.id())
}

fn project_card(index: usize, project: &RecentProject, snapshot: &HubSnapshot) -> ProjectCardData {
    let language = snapshot.settings.language;
    let cover_image = media::project_cover(index, project);
    let has_cover = cover_image.is_some();
    let engine_id = project_engine_id(project, snapshot).unwrap_or_default();
    let engine_label = project_engine_label(project, snapshot);
    let missing = !project.path.exists();
    ProjectCardData {
        title: shared(display_name(project)),
        project_path: shared(path_text(&project.path, language)),
        modified: shared(relative_time(
            now_unix_ms(),
            project.last_opened_unix_ms,
            language,
        )),
        version: shared(project_version_label(engine_id)),
        engine_id: shared(engine_id),
        engine_label: shared(engine_label),
        platform: shared(platform_label()),
        cover_image: cover_image.unwrap_or_default(),
        has_cover,
        selected: project_is_selected(project, snapshot),
        pinned: project_is_pinned(project, snapshot),
        missing,
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
    let engine_id = project_engine_id(project, snapshot).unwrap_or_default();
    let engine_label = project_engine_label(project, snapshot);
    let validation = crate::projects::validate_project_root(&project.path);
    let missing = !project.path.exists();
    RecentProjectRowData {
        title: shared(display_name(project)),
        project_path: shared(path_text(&project.path, language)),
        modified: shared(relative_time(
            now_unix_ms(),
            project.last_opened_unix_ms,
            language,
        )),
        version: shared(project_version_label(engine_id)),
        engine_id: shared(engine_id),
        engine_label: shared(engine_label),
        cover_image: cover_image.unwrap_or_default(),
        has_cover,
        selected: project_is_selected(project, snapshot),
        pinned: project_is_pinned(project, snapshot),
        missing,
        can_open: validation == crate::projects::ProjectValidation::Valid,
        can_delete: cfg!(target_os = "windows") && !missing,
        accent: index as i32,
    }
}

fn project_is_selected(project: &RecentProject, snapshot: &HubSnapshot) -> bool {
    snapshot
        .selected_project_path
        .as_ref()
        .is_some_and(|selected| selected == &project.path)
}

fn project_is_pinned(project: &RecentProject, snapshot: &HubSnapshot) -> bool {
    metadata_for_path(&snapshot.project_metadata, &project.path)
        .is_some_and(|metadata| metadata.pinned)
}

fn project_engine_id<'a>(project: &RecentProject, snapshot: &'a HubSnapshot) -> Option<&'a str> {
    metadata_for_path(&snapshot.project_metadata, &project.path)
        .and_then(|metadata| metadata.engine_id.as_deref())
        .or(snapshot.active_engine_id.as_deref())
}

fn project_engine_label(project: &RecentProject, snapshot: &HubSnapshot) -> String {
    project_engine_id(project, snapshot)
        .and_then(|id| {
            snapshot
                .engines
                .iter()
                .find(|engine| engine.id == id)
                .map(|engine| engine.display_name.clone())
        })
        .unwrap_or_else(|| "No Source Engine".to_string())
}

fn project_version_label(engine_id: &str) -> String {
    if engine_id.trim().is_empty() {
        "Unbound".to_string()
    } else {
        format!("Zircon Engine {}", env!("CARGO_PKG_VERSION"))
    }
}

fn project_browser_projects(snapshot: &HubSnapshot) -> Vec<RecentProject> {
    let projects = snapshot.filtered_recent_projects();
    let pinned: Vec<_> = projects
        .iter()
        .filter(|project| project_is_pinned(project, snapshot))
        .cloned()
        .collect();
    if pinned.is_empty() {
        projects
    } else {
        pinned
    }
}

fn selected_recent_project(snapshot: &HubSnapshot) -> Option<RecentProject> {
    let selected_path = snapshot.selected_project_path.as_ref()?;
    snapshot
        .recent_projects
        .iter()
        .find(|project| &project.path == selected_path)
        .cloned()
}

fn source_engine_rows_for_selection(
    snapshot: &HubSnapshot,
    selected_engine_id: Option<&str>,
) -> Vec<SourceEngineRowData> {
    let language = snapshot.settings.language;
    snapshot
        .engines
        .iter()
        .map(|engine| SourceEngineRowData {
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
            status: shared(if Some(engine.id.as_str()) == selected_engine_id {
                "Selected"
            } else {
                "Registered"
            }),
            active: Some(engine.id.as_str()) == selected_engine_id,
        })
        .collect()
}

fn shared_path_eq(left: &Path, right: &Path) -> bool {
    left == right
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
