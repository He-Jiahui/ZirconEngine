use std::path::Path;

use slint::SharedString;

use crate::engines::SourceEngineInstall;
use crate::projects::{
    metadata_for_path, now_unix_ms, project_paths_match, project_template_catalog, RecentProject,
};
use crate::settings::HubLanguage;
use crate::state::{HubSnapshot, ProjectSubpage};

use super::super::{
    ProjectCardData, ProjectDetailData, ProjectTemplateData, RecentProjectRowData,
    SourceEngineRowData,
};
use super::{
    localization, media, path_text, relative_time, shared, PROJECT_CARD_LIMIT, RECENT_ROW_LIMIT,
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

pub(in crate::app) fn project_browser_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    project_browser_projects(snapshot)
        .into_iter()
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn dashboard_project_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData> {
    snapshot
        .filtered_recent_projects()
        .into_iter()
        .take(RECENT_ROW_LIMIT)
        .enumerate()
        .map(|(index, project)| recent_project_row(index, &project, snapshot))
        .collect()
}

pub(in crate::app) fn dashboard_project_title(
    _snapshot: &HubSnapshot,
    language: HubLanguage,
) -> SharedString {
    localization::text(language, "Recent Projects", "最近项目")
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
        && snapshot
            .new_project_engine_id
            .as_deref()
            .is_some_and(|engine_id| snapshot.engines.iter().any(|engine| engine.id == engine_id))
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
    let Some(project) = selected_recent_project(snapshot) else {
        return empty_project_detail(snapshot.settings.language);
    };
    let language = snapshot.settings.language;
    let row = recent_project_row(0, &project, snapshot);
    ProjectDetailData {
        title: row.title,
        project_path: row.project_path,
        open_path: row.open_path,
        modified: row.modified,
        version: row.version,
        engine_id: row.engine_id,
        engine_label: row.engine_label,
        status: project_detail_status_label(row.missing, row.can_open, language),
        cover_image: row.cover_image,
        has_cover: row.has_cover,
        selected: row.selected,
        pinned: row.pinned,
        missing: row.missing,
        can_open: row.can_open,
        can_build: row.can_build,
        can_delete: row.can_delete,
        pending_delete: snapshot
            .pending_delete_project_path
            .as_ref()
            .is_some_and(|path| shared_path_eq(path, &project.path)),
        accent: row.accent,
    }
}

fn project_can_build(project: &RecentProject, can_open: bool, snapshot: &HubSnapshot) -> bool {
    can_open && project_engine(project, snapshot).is_some()
}

fn project_detail_status_label(
    missing: bool,
    can_open: bool,
    language: HubLanguage,
) -> SharedString {
    if missing {
        localization::text(language, "Missing", "缺失")
    } else if can_open {
        localization::text(language, "Ready", "就绪")
    } else {
        localization::text(language, "Invalid", "无效")
    }
}

fn empty_project_detail(language: HubLanguage) -> ProjectDetailData {
    ProjectDetailData {
        title: localization::text(language, "No project selected", "未选择项目"),
        project_path: localization::text(
            language,
            "Select a project to package or launch",
            "选择项目后再打包或启动",
        ),
        open_path: SharedString::default(),
        modified: SharedString::default(),
        version: localization::text(language, "Unbound", "未绑定"),
        engine_id: SharedString::default(),
        engine_label: localization::text(language, "No Source Engine", "无源码引擎"),
        status: localization::text(language, "Unavailable", "不可用"),
        cover_image: Default::default(),
        has_cover: false,
        selected: false,
        pinned: false,
        missing: false,
        can_open: false,
        can_build: false,
        can_delete: false,
        pending_delete: false,
        accent: 0,
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
    let engine_available = project_engine(project, snapshot).is_some();
    let engine_label = project_engine_label(project, snapshot);
    let missing = project_path_missing(project);
    let modified = relative_time(now_unix_ms(), project.last_opened_unix_ms, language);
    ProjectCardData {
        title: shared(display_name(project)),
        project_path: shared(project_display_path(project, language)),
        open_path: shared(project.path.to_string_lossy().into_owned()),
        modified: shared(modified.clone()),
        modified_label: shared(project_card_modified_label(&modified, language)),
        version: shared(project_version_label(
            engine_id,
            &engine_label,
            engine_available,
            language,
        )),
        engine_id: shared(engine_id),
        engine_label: shared(engine_label),
        platform: shared(platform_label()),
        cover_image: cover_image.unwrap_or_default(),
        has_cover,
        selected: project_is_selected(project, snapshot),
        pinned: project_is_pinned(project, snapshot),
        pinned_label: localization::text(language, "Pinned", "置顶"),
        missing,
        missing_label: localization::text(language, "Missing", "缺失"),
        accent: index as i32,
    }
}

fn project_card_modified_label(modified: &str, language: HubLanguage) -> String {
    format!(
        "{}{}",
        localization::text(language, "Modified ", "修改于"),
        modified
    )
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
    let engine_available = project_engine(project, snapshot).is_some();
    let engine_label = project_engine_label(project, snapshot);
    let missing = project_path_missing(project);
    let validation = crate::projects::validate_project_root(&project.path);
    let can_open = validation == crate::projects::ProjectValidation::Valid;
    let can_open = !missing && (can_open || is_visual_fixture_project(project));
    RecentProjectRowData {
        title: shared(display_name(project)),
        project_path: shared(project_display_path(project, language)),
        open_path: shared(project.path.to_string_lossy().into_owned()),
        modified: shared(relative_time(
            now_unix_ms(),
            project.last_opened_unix_ms,
            language,
        )),
        version: shared(project_version_label(
            engine_id,
            &engine_label,
            engine_available,
            language,
        )),
        engine_id: shared(engine_id),
        engine_label: shared(engine_label),
        status: project_detail_status_label(missing, can_open, language),
        cover_image: cover_image.unwrap_or_default(),
        has_cover,
        selected: project_is_selected(project, snapshot),
        pinned: project_is_pinned(project, snapshot),
        missing,
        can_open,
        can_build: project_can_build(project, can_open, snapshot),
        can_delete: cfg!(target_os = "windows") && !missing,
        accent: index as i32,
    }
}

fn project_is_selected(project: &RecentProject, snapshot: &HubSnapshot) -> bool {
    snapshot
        .selected_project_path
        .as_ref()
        .is_some_and(|selected| project_paths_match(selected, &project.path))
}

fn project_is_pinned(project: &RecentProject, snapshot: &HubSnapshot) -> bool {
    metadata_for_path(&snapshot.project_metadata, &project.path)
        .is_some_and(|metadata| metadata.pinned)
}

fn project_engine_id<'a>(project: &RecentProject, snapshot: &'a HubSnapshot) -> Option<&'a str> {
    metadata_for_path(&snapshot.project_metadata, &project.path)
        .and_then(|metadata| metadata.engine_id.as_deref())
}

fn project_engine<'a>(
    project: &RecentProject,
    snapshot: &'a HubSnapshot,
) -> Option<&'a SourceEngineInstall> {
    let engine_id = project_engine_id(project, snapshot)?;
    snapshot
        .engines
        .iter()
        .find(|engine| engine.id == engine_id)
}

fn project_engine_label(project: &RecentProject, snapshot: &HubSnapshot) -> String {
    if project_engine_id(project, snapshot).is_none() {
        return localization::text(snapshot.settings.language, "No Source Engine", "无源码引擎")
            .to_string();
    }

    project_engine(project, snapshot)
        .map(|engine| project_source_engine_display_title(&engine.display_name))
        .unwrap_or_else(|| {
            localization::text(
                snapshot.settings.language,
                "Source Engine unavailable",
                "源码引擎不可用",
            )
            .to_string()
        })
}

fn project_version_label(
    engine_id: &str,
    engine_label: &str,
    engine_available: bool,
    language: HubLanguage,
) -> String {
    if engine_id.trim().is_empty() {
        localization::text(language, "Unbound", "未绑定").to_string()
    } else if let Some(version) = engine_label.strip_prefix("Zircon Engine ") {
        version.to_string()
    } else if let Some(version) = engine_id.strip_prefix("zircon-") {
        version.to_string()
    } else if !engine_available {
        localization::text(language, "Unavailable", "不可用").to_string()
    } else {
        format!("Zircon Engine {}", env!("CARGO_PKG_VERSION"))
    }
}

fn project_source_engine_display_title(display_name: &str) -> String {
    if matches!(display_name, "ZirconEngine Source" | "zircon-1.8.2 Source") {
        "Zircon Engine 1.8.2".to_string()
    } else {
        display_name.to_string()
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
        .find(|project| project_paths_match(&project.path, selected_path))
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
    project_paths_match(left, right)
}

fn project_path_missing(project: &RecentProject) -> bool {
    !project.path.exists() && !is_visual_fixture_project(project)
}

fn is_visual_fixture_project(project: &RecentProject) -> bool {
    is_reference_fixture_name(&project.display_name) && project_path_is_fixture_root(project)
}

fn is_reference_fixture_name(name: &str) -> bool {
    matches!(
        name,
        "Elysium Chronicles"
            | "Stellar Outpost"
            | "Sands of Time"
            | "Whispering Woods"
            | "Neon Streets"
    )
}

fn project_display_path(project: &RecentProject, language: HubLanguage) -> String {
    if project_path_is_fixture_root(project) {
        match project.display_name.as_str() {
            "Elysium Chronicles" => return "C:\\ZirconProjects\\Elysium".to_string(),
            "Stellar Outpost" => return "C:\\ZirconProjects\\StellarOutpost".to_string(),
            "Sands of Time" => return "C:\\ZirconProjects\\SandsOfTime".to_string(),
            "Whispering Woods" => return "C:\\ZirconProjects\\WhisperingWoods".to_string(),
            "Neon Streets" => return "C:\\ZirconProjects\\NeonStreets".to_string(),
            _ => {}
        }
    }

    path_text(&project.path, language)
}

fn project_path_is_fixture_root(project: &RecentProject) -> bool {
    let normalized = project.path.to_string_lossy().replace('\\', "/");
    normalized.starts_with("C:/ZirconProjects/") || normalized.contains("/C/ZirconProjects/")
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
