use std::path::Path;

use slint::SharedString;

use crate::engines::SourceEngineInstall;
use crate::process::preferred_editor_executable_exists;
use crate::settings::HubLanguage;
use crate::state::{HubActionRecord, HubSnapshot, ProjectScope, SourceEngineScope};

use super::super::WorkspaceActionReadinessData;
use super::{build_context_engine, localization, path_text, shared};

pub(in crate::app) fn workspace_action_readiness(
    snapshot: &HubSnapshot,
) -> WorkspaceActionReadinessData {
    let language = snapshot.settings.language;
    let scope = snapshot.scope();
    let source_engine = build_context_engine(snapshot);
    let selected_project = selected_project_state(snapshot, language);
    let project_ready = selected_project.available && selected_project.root_valid;
    let source_ready = source_engine.is_some();
    let staged_ready = source_engine
        .map(|engine| preferred_editor_executable_exists(engine.staged_engine_dir()))
        .unwrap_or(false);
    let output_ready = source_engine
        .map(|engine| path_configured(&engine.output_dir))
        .unwrap_or_else(|| path_configured(&snapshot.settings.default_build_output_dir));
    let device_ready = path_configured(&snapshot.settings.default_device_install_dir);

    let build_enabled = project_ready && source_ready;
    let open_editor_enabled = project_ready && source_ready && staged_ready;
    let open_output_enabled = output_ready;
    let package_enabled = project_ready && output_ready;
    let install_enabled = package_enabled && device_ready;

    let source_engine_status = match &scope.source_engine {
        SourceEngineScope::ProjectBound(_) => {
            localization::text(language, "Project-bound Source Engine", "项目绑定源码引擎")
        }
        SourceEngineScope::Active(_) => localization::text(
            language,
            "Active Source Engine fallback",
            "当前源码引擎回退",
        ),
        SourceEngineScope::ProjectUnbound { .. } => localization::text(
            language,
            "Project has no bound Source Engine",
            "项目未绑定源码引擎",
        ),
        SourceEngineScope::ProjectEngineUnavailable { .. } => localization::text(
            language,
            "Project-bound Source Engine is unavailable",
            "项目绑定源码引擎不可用",
        ),
        SourceEngineScope::None => {
            localization::text(language, "No Source Engine configured", "未配置源码引擎")
        }
    };

    WorkspaceActionReadinessData {
        selected_project_title: selected_project.title,
        selected_project_path: selected_project.path,
        selected_project_status: selected_project.status,
        source_engine_title: source_engine_title(source_engine, language),
        source_engine_status,
        source_build_summary: source_build_summary(source_engine, language),
        operation_summary: operation_summary(snapshot.action_history.first(), language),
        build_enabled,
        build_disabled_reason: disabled_reason(
            build_enabled,
            selected_project.build_reason.as_ref(),
            source_disabled_reason(&scope.source_engine, language).as_ref(),
            None,
        ),
        open_editor_enabled,
        open_editor_disabled_reason: disabled_reason(
            open_editor_enabled,
            selected_project.open_reason.as_ref(),
            source_disabled_reason(&scope.source_engine, language).as_ref(),
            staged_disabled_reason(staged_ready, language).as_ref(),
        ),
        open_output_enabled,
        open_output_disabled_reason: disabled_reason(
            open_output_enabled,
            None,
            None,
            output_disabled_reason(output_ready, language).as_ref(),
        ),
        package_enabled,
        package_disabled_reason: disabled_reason(
            package_enabled,
            selected_project.package_reason.as_ref(),
            None,
            output_disabled_reason(output_ready, language).as_ref(),
        ),
        install_enabled,
        install_disabled_reason: disabled_reason(
            install_enabled,
            selected_project.install_reason.as_ref(),
            None,
            output_disabled_reason(output_ready, language)
                .or_else(|| device_disabled_reason(device_ready, language))
                .as_ref(),
        ),
    }
}

struct SelectedProjectReadiness {
    title: SharedString,
    path: SharedString,
    status: SharedString,
    available: bool,
    root_valid: bool,
    build_reason: Option<SharedString>,
    open_reason: Option<SharedString>,
    package_reason: Option<SharedString>,
    install_reason: Option<SharedString>,
}

fn selected_project_state(
    snapshot: &HubSnapshot,
    language: HubLanguage,
) -> SelectedProjectReadiness {
    match snapshot.scope().project {
        ProjectScope::Selected(project) => {
            let path = path_text(&project.path, language);
            let root_valid = project.path.exists();
            let missing_root = (!root_valid).then(|| {
                localization::text(
                    language,
                    "Selected project root is missing",
                    "所选项目根目录缺失",
                )
            });
            let status = if root_valid {
                localization::text(language, "Selected project ready", "所选项目就绪")
            } else {
                missing_root.clone().unwrap_or_else(|| {
                    localization::text(language, "Selected project unavailable", "所选项目不可用")
                })
            };
            SelectedProjectReadiness {
                title: shared(project.display_name),
                path: shared(path),
                status,
                available: true,
                root_valid,
                build_reason: missing_root.clone(),
                open_reason: missing_root.clone(),
                package_reason: missing_root.clone(),
                install_reason: missing_root,
            }
        }
        ProjectScope::StaleSelection { requested_path } => {
            let reason = localization::text(
                language,
                "Selected project is no longer available",
                "所选项目不再可用",
            );
            SelectedProjectReadiness {
                title: localization::text(language, "Stale project selection", "过期项目选择"),
                path: shared(path_text(&requested_path, language)),
                status: reason.clone(),
                available: false,
                root_valid: false,
                build_reason: Some(action_reason(
                    language,
                    "Selected project is no longer available to build",
                    "所选项目不再可构建",
                )),
                open_reason: Some(action_reason(
                    language,
                    "Selected project is no longer available to open",
                    "所选项目不再可打开",
                )),
                package_reason: Some(action_reason(
                    language,
                    "Selected project is no longer available to package",
                    "所选项目不再可打包",
                )),
                install_reason: Some(action_reason(
                    language,
                    "Selected project is no longer available to install",
                    "所选项目不再可安装",
                )),
            }
        }
        ProjectScope::LatestRecent(_) | ProjectScope::None => SelectedProjectReadiness {
            title: localization::text(language, "No project selected", "未选择项目"),
            path: SharedString::default(),
            status: localization::text(
                language,
                "Select a project before using workspace actions",
                "请先选择项目再使用工作区操作",
            ),
            available: false,
            root_valid: false,
            build_reason: Some(action_reason(
                language,
                "Select a project before building",
                "构建前请选择项目",
            )),
            open_reason: Some(action_reason(
                language,
                "Select a project before opening",
                "打开前请选择项目",
            )),
            package_reason: Some(action_reason(
                language,
                "Select a project before packaging",
                "打包前请选择项目",
            )),
            install_reason: Some(action_reason(
                language,
                "Select a project before installing",
                "安装前请选择项目",
            )),
        },
    }
}

fn action_reason(language: HubLanguage, english: &str, chinese: &str) -> SharedString {
    localization::text(language, english, chinese)
}

fn source_engine_title(
    engine: Option<&SourceEngineInstall>,
    language: HubLanguage,
) -> SharedString {
    engine
        .map(|engine| shared(&engine.display_name))
        .unwrap_or_else(|| localization::text(language, "No Source Engine", "无源码引擎"))
}

fn source_disabled_reason(
    source_engine: &SourceEngineScope,
    language: HubLanguage,
) -> Option<SharedString> {
    match source_engine {
        SourceEngineScope::ProjectBound(_) | SourceEngineScope::Active(_) => None,
        SourceEngineScope::ProjectUnbound { .. } => Some(localization::text(
            language,
            "Bind a Source Engine before building",
            "构建前请绑定源码引擎",
        )),
        SourceEngineScope::ProjectEngineUnavailable { .. } => Some(localization::text(
            language,
            "Project-bound Source Engine is unavailable",
            "项目绑定源码引擎不可用",
        )),
        SourceEngineScope::None => Some(localization::text(
            language,
            "Configure a Source Engine before building",
            "构建前请配置源码引擎",
        )),
    }
}

fn staged_disabled_reason(staged_ready: bool, language: HubLanguage) -> Option<SharedString> {
    (!staged_ready).then(|| {
        localization::text(
            language,
            "Staged editor executable is unavailable",
            "暂存编辑器可执行文件不可用",
        )
    })
}

fn output_disabled_reason(output_ready: bool, language: HubLanguage) -> Option<SharedString> {
    (!output_ready).then(|| {
        localization::text(
            language,
            "Configure a build output directory",
            "请配置构建输出目录",
        )
    })
}

fn device_disabled_reason(device_ready: bool, language: HubLanguage) -> Option<SharedString> {
    (!device_ready).then(|| {
        localization::text(
            language,
            "Configure a device install directory",
            "请配置设备安装目录",
        )
    })
}

fn disabled_reason(
    enabled: bool,
    project_reason: Option<&SharedString>,
    source_reason: Option<&SharedString>,
    path_reason: Option<&SharedString>,
) -> SharedString {
    if enabled {
        return SharedString::default();
    }
    project_reason
        .or(source_reason)
        .or(path_reason)
        .cloned()
        .unwrap_or_default()
}

fn source_build_summary(
    engine: Option<&SourceEngineInstall>,
    language: HubLanguage,
) -> SharedString {
    let Some(engine) = engine else {
        return localization::text(language, "No source build history", "没有源码构建历史");
    };
    engine
        .build_history
        .first()
        .map(|record| {
            shared(if record.detail.trim().is_empty() {
                record.status.clone()
            } else {
                record.detail.clone()
            })
        })
        .unwrap_or_else(|| localization::text(language, "Not built yet", "尚未构建"))
}

fn operation_summary(record: Option<&HubActionRecord>, language: HubLanguage) -> SharedString {
    record
        .map(|record| {
            if record.detail.trim().is_empty() {
                localization::text(language, record.action.label(), record.action.label())
            } else {
                shared(&record.detail)
            }
        })
        .unwrap_or_else(|| localization::text(language, "No operation timeline", "没有操作时间线"))
}

fn path_configured(path: &Path) -> bool {
    !path.as_os_str().is_empty()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::engines::{SourceBuildRecord, SourceEngineInstall};
    use crate::projects::{ProjectMetadata, ProjectMetadataMap, RecentProject};
    use crate::settings::HubSettings;
    use crate::state::{
        HubActionKind, HubActionRecord, HubActionStatus, HubPage, ProjectFilterMode,
        ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    fn snapshot_with_project(
        project_path: PathBuf,
        metadata: ProjectMetadataMap,
        engines: Vec<SourceEngineInstall>,
    ) -> HubSnapshot {
        HubSnapshot {
            selected_page: HubPage::Builds,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(project_path.clone()),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: vec![RecentProject::new("Game", project_path, 42)],
            project_metadata: metadata,
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            action_history: Vec::new(),
            engines,
            active_engine_id: Some("active-source-must-not-be-used".to_string()),
            settings: HubSettings::default(),
        }
    }

    #[test]
    fn workspace_action_readiness_requires_selected_project() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Builds,
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
            recent_projects: vec![RecentProject::new("Latest", "E:/Projects/Latest", 42)],
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let readiness = workspace_action_readiness(&snapshot);

        assert!(!readiness.build_enabled);
        assert!(!readiness.package_enabled);
        assert_eq!(readiness.selected_project_title, "No project selected");
        assert_eq!(
            readiness.build_disabled_reason,
            "Select a project before building"
        );
        assert_eq!(
            readiness.package_disabled_reason,
            "Select a project before packaging"
        );
    }

    #[test]
    fn workspace_action_readiness_reports_bound_engine_without_active_fallback() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-workspace-actions-{}",
            crate::projects::now_unix_ms()
        ));
        let project_path = root.join("game");
        let source_path = root.join("source");
        let output_path = root.join("out");
        let device_path = root.join("device");
        fs::create_dir_all(&project_path).expect("project root should exist");
        fs::create_dir_all(&source_path).expect("source root should exist");
        fs::create_dir_all(&output_path).expect("output root should exist");
        fs::create_dir_all(&device_path).expect("device root should exist");
        let mut metadata = ProjectMetadataMap::new();
        metadata.insert(
            crate::projects::project_metadata_key(&project_path),
            ProjectMetadata {
                engine_id: Some("bound".to_string()),
                ..ProjectMetadata::default()
            },
        );
        let mut snapshot = snapshot_with_project(
            project_path,
            metadata,
            vec![SourceEngineInstall {
                id: "bound".to_string(),
                display_name: "Bound Source".to_string(),
                source_dir: source_path,
                output_dir: output_path,
                last_build_unix_ms: Some(1),
                build_history: vec![SourceBuildRecord {
                    finished_unix_ms: 1,
                    status: "success".to_string(),
                    profile: "debug".to_string(),
                    jobs: Some(4),
                    output_dir: root.join("out"),
                    detail: "selected history".to_string(),
                    log_excerpt: String::new(),
                    command_line: Vec::new(),
                }],
            }],
        );
        snapshot.settings.default_device_install_dir = device_path;
        snapshot.action_history = vec![HubActionRecord {
            finished_unix_ms: 2,
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Success,
            target: "Bound Source".to_string(),
            detail: "last operation".to_string(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: Some(root.join("out")),
        }];

        let readiness = workspace_action_readiness(&snapshot);
        fs::remove_dir_all(root).expect("test roots should be removed");

        assert!(readiness.build_enabled);
        assert!(readiness.package_enabled);
        assert!(readiness.install_enabled);
        assert!(!readiness.open_editor_enabled);
        assert_eq!(readiness.source_engine_title, "Bound Source");
        assert_eq!(readiness.source_build_summary, "selected history");
        assert_eq!(readiness.operation_summary, "last operation");
        assert_eq!(
            readiness.open_editor_disabled_reason,
            "Staged editor executable is unavailable"
        );
    }

    #[test]
    fn workspace_action_readiness_reports_unbound_and_stale_selection_reasons() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-workspace-actions-unbound-{}",
            crate::projects::now_unix_ms()
        ));
        let project_path = root.join("game");
        fs::create_dir_all(&project_path).expect("project root should exist");
        let unbound = workspace_action_readiness(&snapshot_with_project(
            project_path.clone(),
            ProjectMetadataMap::new(),
            vec![SourceEngineInstall {
                id: "active".to_string(),
                display_name: "Active Source".to_string(),
                source_dir: root.join("source"),
                output_dir: root.join("out"),
                last_build_unix_ms: None,
                build_history: Vec::new(),
            }],
        ));
        fs::remove_dir_all(&root).expect("test root should be removed");

        assert!(!unbound.build_enabled);
        assert_eq!(
            unbound.build_disabled_reason,
            "Bind a Source Engine before building"
        );

        let mut stale = snapshot_with_project(
            PathBuf::from("E:/Projects/Missing"),
            ProjectMetadataMap::new(),
            Vec::new(),
        );
        stale.recent_projects.clear();
        let stale_readiness = workspace_action_readiness(&stale);

        assert!(!stale_readiness.build_enabled);
        assert_eq!(
            stale_readiness.build_disabled_reason,
            "Selected project is no longer available to build"
        );
        assert_eq!(
            stale_readiness.install_disabled_reason,
            "Selected project is no longer available to install"
        );
    }

    #[test]
    fn workspace_action_readiness_reports_source_build_and_operation_summaries() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-workspace-actions-summary-{}",
            crate::projects::now_unix_ms()
        ));
        let project_path = root.join("game");
        fs::create_dir_all(&project_path).expect("project root should exist");
        let mut metadata = ProjectMetadataMap::new();
        metadata.insert(
            crate::projects::project_metadata_key(&project_path),
            ProjectMetadata {
                engine_id: Some("bound".to_string()),
                ..ProjectMetadata::default()
            },
        );
        let mut snapshot = snapshot_with_project(
            project_path,
            metadata,
            vec![SourceEngineInstall {
                id: "bound".to_string(),
                display_name: "Bound Source".to_string(),
                source_dir: root.join("source"),
                output_dir: root.join("out"),
                last_build_unix_ms: None,
                build_history: vec![SourceBuildRecord {
                    finished_unix_ms: 9,
                    status: "failed".to_string(),
                    profile: "debug".to_string(),
                    jobs: Some(2),
                    output_dir: root.join("out"),
                    detail: String::new(),
                    log_excerpt: "compile error".to_string(),
                    command_line: vec!["python".to_string(), "tools/zircon_build.py".to_string()],
                }],
            }],
        );
        snapshot.action_history = vec![HubActionRecord {
            finished_unix_ms: 10,
            action: HubActionKind::OpenOutput,
            status: HubActionStatus::Failed,
            target: "Bound Source".to_string(),
            detail: String::new(),
            log_excerpt: "missing output".to_string(),
            recovery: Some("Configure output".to_string()),
            process_id: None,
            command_line: Vec::new(),
            output_dir: None,
        }];

        let readiness = workspace_action_readiness(&snapshot);
        fs::remove_dir_all(root).expect("test root should be removed");

        assert_eq!(readiness.source_build_summary, "failed");
        assert_eq!(readiness.operation_summary, "Open Output");
    }

    #[test]
    fn workspace_action_readiness_reports_missing_output_and_device_paths() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-workspace-actions-paths-{}",
            crate::projects::now_unix_ms()
        ));
        let project_path = root.join("game");
        fs::create_dir_all(&project_path).expect("project root should exist");
        let mut metadata = ProjectMetadataMap::new();
        metadata.insert(
            crate::projects::project_metadata_key(&project_path),
            ProjectMetadata {
                engine_id: Some("bound".to_string()),
                ..ProjectMetadata::default()
            },
        );
        let mut snapshot = snapshot_with_project(
            project_path,
            metadata,
            vec![SourceEngineInstall {
                id: "bound".to_string(),
                display_name: "Bound Source".to_string(),
                source_dir: root.join("source"),
                output_dir: PathBuf::new(),
                last_build_unix_ms: None,
                build_history: Vec::new(),
            }],
        );
        snapshot.settings.default_device_install_dir = PathBuf::new();

        let missing_output = workspace_action_readiness(&snapshot);
        assert!(!missing_output.open_output_enabled);
        assert!(!missing_output.package_enabled);
        assert_eq!(
            missing_output.open_output_disabled_reason,
            "Configure a build output directory"
        );
        assert_eq!(
            missing_output.package_disabled_reason,
            "Configure a build output directory"
        );

        snapshot.engines[0].output_dir = root.join("out");
        let missing_device = workspace_action_readiness(&snapshot);
        fs::remove_dir_all(root).expect("test root should be removed");

        assert!(missing_device.package_enabled);
        assert!(!missing_device.install_enabled);
        assert_eq!(
            missing_device.install_disabled_reason,
            "Configure a device install directory"
        );
    }
}
