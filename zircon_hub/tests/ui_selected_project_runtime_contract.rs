//! Static contracts for selected-project runtime/action scope behavior.
use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_ui_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub UI file {name}: {error}");
        }),
    )
}

fn read_crate_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {name}: {error}");
        }),
    )
}
#[test]
fn selected_project_runtime_actions_use_project_scope() {
    let builds = read_ui_file("builds.slint");
    let shared = read_ui_file("shared.slint");
    let project_detail_data = shared
        .split("export struct ProjectDetailData")
        .nth(1)
        .and_then(|source| source.split("export struct QuickActionData").next())
        .expect("shared.slint must declare ProjectDetailData before QuickActionData");
    for snippet in ["can-open: bool,", "can-build: bool,"] {
        assert!(
            project_detail_data.contains(snippet),
            "ProjectDetailData must expose a separate build-ready flag so Builds can disable Build without disabling Open/Package/Install; missing {snippet}"
        );
    }
    for snippet in [
        "import { MutedText, ProjectDetailData, SourceBuildHistoryRowData, SourceEngineData, UiTextData } from \"shared.slint\";",
        "in property <ProjectDetailData> project;",
        "root.project.can-build ? root.project.title : (root.project.can-open ? root.project.engine-label : root.ui-text.select-project-before-building)",
        "action-enabled: root.project.can-build;",
        "root.project.can-open ? root.project.title : root.ui-text.select-project-before-opening",
        "root.project.project-path != \"\" ? root.project.project-path : root.ui-text.select-project-before-packaging",
        "callback package-project();",
        "callback install-device();",
        "id: \"package-project\"",
        "id: \"install-device\"",
        "action-title: root.ui-text.install-to-device;",
        "root.project.can-open ? root.project.title : root.ui-text.select-project-before-packaging",
        "root.project.can-open ? root.project.title : root.ui-text.select-project-before-installing",
        "action-enabled: root.project.can-open;",
    ] {
        assert!(
            builds.contains(snippet),
            "BuildsPage must surface selected-project build/package context; missing {snippet}"
        );
    }

    let app = read_ui_file("app.slint");
    assert!(
        app.contains("project: root.project-detail;"),
        "HubWindow must pass selected project detail into BuildsPage"
    );
    for snippet in [
        "callback launch-selected-project();",
        "callback build-selected-project-engine();",
        "callback package-selected-project();",
        "callback install-selected-project();",
        "build-engine => { root.build-selected-project-engine(); }",
        "launch-editor => { root.launch-selected-project(); }",
        "package-project => { root.package-selected-project(); }",
        "install-device => { root.install-selected-project(); }",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must route Builds page project actions through selected-project-only callbacks instead of Dashboard quick-action fallback; missing {snippet}"
        );
    }
    assert!(
        app.contains("has-selected-project: root.project-detail.selected;"),
        "HubWindow must pass selected project state into catalog/workspace page empty-state copy"
    );

    let project_projection = read_crate_file("src/app/view_model/projects.rs");
    for snippet in [
        "pub(in crate::app) fn project_create_enabled(snapshot: &HubSnapshot) -> bool",
        ".new_project_engine_id",
        ".is_some_and(|engine_id| snapshot.engines.iter().any(|engine| engine.id == engine_id))",
        "\"Selected Source Engine is unavailable\"",
    ] {
        assert!(
            project_projection.contains(snippet),
            "New Project create projection must require a selected Source Engine that still exists; missing {snippet}"
        );
    }
    let project_view_model = read_crate_file("src/app/view_model.rs");
    for snippet in [
        "project_create_requires_available_source_engine",
        "missing-source",
        "SharedString::from(\"Selected Source Engine is unavailable\")",
        "SharedString::from(\"Local Source\")",
    ] {
        assert!(
            project_view_model.contains(snippet),
            "Project view-model tests must cover unavailable New Project Source Engine selection; missing {snippet}"
        );
    }
    for snippet in [
        "return empty_project_detail(snapshot.settings.language);",
        "fn empty_project_detail(language: HubLanguage) -> ProjectDetailData",
        "fn project_detail_status_label(",
        "localization::text(language, \"Missing\", \"缺失\")",
        "localization::text(language, \"Ready\", \"就绪\")",
        "localization::text(language, \"Invalid\", \"无效\")",
        "\"No project selected\",",
        "\"Select a project to package or launch\",",
        "can_open: false,",
        "can_delete: false,",
    ] {
        assert!(
            project_projection.contains(snippet),
            "ProjectDetailData must remain an explicit no-selection state until selected_project_path resolves; missing {snippet}"
        );
    }
    assert!(
        !project_projection.contains(".or_else(|| project_browser_projects(snapshot).into_iter().next())"),
        "ProjectDetailData must not silently fall back to the first project; only Quick Actions use no-selection latest-recent fallback"
    );
    for snippet in [
        "can_build: row.can_build,",
        "let can_open = validation == crate::projects::ProjectValidation::Valid;",
        "can_build: project_can_build(project, can_open, snapshot),",
        "fn project_can_build(",
        "can_open && project_engine(project, snapshot).is_some()",
        "can_build: false,",
    ] {
        assert!(
            project_projection.contains(snippet),
            "ProjectDetailData must distinguish openable projects from buildable projects with a valid bound Source Engine; missing {snippet}"
        );
    }
    for snippet in [
        "project_detail_can_build_requires_available_bound_source_engine",
        "assert!(bound_detail.can_build)",
        "assert!(!unbound_detail.can_build)",
        "assert!(!stale_detail.can_build)",
        "assert!(bound_row.can_build)",
        "assert!(!unbound_row.can_build)",
        "assert!(!stale_row.can_build)",
        "active-source-must-not-be-used",
    ] {
        assert!(
            project_view_model.contains(snippet),
            "Project view-model tests must cover ProjectDetailData can_build for bound, unbound, and stale Source Engine states; missing {snippet}"
        );
    }
    for snippet in [
        "pub(super) fn source_engine_data(snapshot: &HubSnapshot) -> SourceEngineData",
        "source_engine_data_for_context(",
        "source_engine_data_for_missing_context(",
        "build_context_engine(snapshot)",
        "source_engine_data_prefers_selected_project_bound_engine",
        "source_engine_data_does_not_fallback_when_selected_project_is_unbound",
        "BuildContextEngine::NoSelectedProject => {",
        "BuildContextEngine::SelectedProjectUnbound",
        "BuildContextEngine::SelectedProjectUnavailable",
        "| BuildContextEngine::SelectedProjectUnavailable => None",
        "fn build_context_engine(snapshot: &HubSnapshot) -> Option<&SourceEngineInstall>",
        "fn selected_project_engine_context(snapshot: &HubSnapshot) -> BuildContextEngine<'_>",
        "fn selected_project_for_context(snapshot: &HubSnapshot) -> Option<&RecentProject>",
        ".find(|project| project_paths_match(&project.path, selected_path))",
        "metadata_for_path(&snapshot.project_metadata, &project.path)",
        "build_history_rows_prefer_selected_project_bound_engine_records",
        "SharedString::from(\"selected history\")",
        "SharedString::from(\"Bind project engine\")",
        "assert!(rows.is_empty())",
    ] {
        assert!(
            project_view_model.contains(snippet),
            "Build history projection must use active-engine fallback only without selected-project context; missing {snippet}"
        );
    }
    assert!(
        !project_projection.contains(".or(snapshot.active_engine_id.as_deref())"),
        "Project item engine/version projection must use per-project metadata instead of falling back to the active Source Engine"
    );
    for snippet in [
        "fn project_engine_id<'a>(project: &RecentProject, snapshot: &'a HubSnapshot) -> Option<&'a str>",
        "fn project_engine<'a>(",
        "metadata_for_path(&snapshot.project_metadata, &project.path)",
        "localization::text(snapshot.settings.language, \"No Source Engine\", \"无源码引擎\")",
        "\"Source Engine unavailable\"",
        "localization::text(language, \"Unavailable\", \"不可用\")",
    ] {
        assert!(
            project_projection.contains(snippet),
            "Project item projection must surface unbound and stale metadata-only engine binding states; missing {snippet}"
        );
    }
    for snippet in [
        "project_rows_use_bound_engine_metadata_without_active_fallback",
        "missing-source",
        "SharedString::from(\"Source Engine unavailable\")",
        "SharedString::from(\"Unavailable\")",
        "HubLanguage::Chinese",
        "SharedString::from(\"源码引擎不可用\")",
        "SharedString::from(\"不可用\")",
        "SharedString::from(\"缺失\")",
    ] {
        assert!(
            project_view_model.contains(snippet),
            "Project view-model tests must cover stale bound Source Engine metadata without active-engine fallback; missing {snippet}"
        );
    }
    for snippet in [
        "project_paths_match(selected, &project.path)",
        ".find(|project| project_paths_match(&project.path, selected_path))",
        "fn shared_path_eq(left: &Path, right: &Path) -> bool",
        "project_paths_match(left, right)",
    ] {
        assert!(
            project_projection.contains(snippet),
            "Project projection must use normalized selected-project path matching instead of direct PathBuf equality; missing {snippet}"
        );
    }

    let project_metadata = read_crate_file("src/projects/metadata.rs");
    for snippet in [
        "pub fn project_paths_match(left: impl AsRef<Path>, right: impl AsRef<Path>) -> bool",
        "project_metadata_key(left) == project_metadata_key(right)",
        "looks_like_windows_drive_path(&text)",
        "bytes[0].is_ascii_alphabetic() && bytes[1] == b':'",
    ] {
        assert!(
            project_metadata.contains(snippet),
            "Project path matching must reuse project metadata key normalization; missing {snippet}"
        );
    }

    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    let select_project_path = workspace
        .split("pub(super) fn select_project_path")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn view_all_projects").next())
        .expect("project_workspace.rs must declare select_project_path before view_all_projects");
    for snippet in [
        "self.selected_project_path = Some(project.path.clone());",
        "project_paths_match(&project.path, &path)",
        "let active_engine_before = self.config.active_engine_id.clone();",
        "self.activate_project_engine_for_path(&project.path);",
        "self.refresh_project_context_views(\n            true,\n            self.config.active_engine_id != active_engine_before,\n        )?;",
    ] {
        assert!(
            select_project_path.contains(snippet),
            "Selecting a project must immediately activate its bound Source Engine and refresh project/engine scoped views; missing {snippet}"
        );
    }
    let remember_project = workspace
        .split("pub(super) fn remember_project(&mut self, project: RecentProject)")
        .nth(1)
        .and_then(|source| source.split("fn refresh_selected_project_scoped_views").next())
        .expect("project_workspace.rs must declare remember_project before refresh_selected_project_scoped_views");
    for snippet in [
        "let active_engine_before = self.config.active_engine_id.clone();",
        "self.selected_project_path = Some(last_project_path.clone());",
        "self.activate_project_engine_for_path(&last_project_path);",
        "self.refresh_project_context_views(\n            true,\n            self.config.active_engine_id != active_engine_before,\n        )?;",
        "self.persist_with_last_project(Some(&last_project_path))",
    ] {
        assert!(
            remember_project.contains(snippet),
            "Remembering an opened or created project must activate its bound Source Engine before the next snapshot; missing {snippet}"
        );
    }
    let detail_engine = workspace
        .split("pub(super) fn select_project_detail_engine_by_id")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn toggle_selected_project_pin").next())
        .expect("project_workspace.rs must declare select_project_detail_engine_by_id before toggle_selected_project_pin");
    for snippet in [
        "self.config.active_engine_id = Some(engine_id.to_string());",
        "self.sync_settings_from_active_engine();",
        "self.refresh_project_context_views(\n            true,\n            self.config.active_engine_id != active_engine_before,\n        )?;",
    ] {
        assert!(
            detail_engine.contains(snippet),
            "Changing a project's bound Source Engine must make that engine current for Builds and scoped pages; missing {snippet}"
        );
    }

    let binding = read_crate_file("src/app/binding.rs");
    assert!(
        binding.contains("let source_engine = view_model::source_engine_data(snapshot);"),
        "binding.rs must project SourceEngineData from the full HubSnapshot so selected-project engine context can suppress active fallback when needed"
    );
    assert!(
        binding.contains("view_model::quick_actions(\n        snapshot, language,"),
        "binding.rs must derive QuickActionData from HubSnapshot so quick actions can describe selected/recent project scope"
    );

    let view_model = read_crate_file("src/app/view_model/quick_actions.rs");
    for snippet in [
        "quick_action_project_target(snapshot)",
        "QuickActionProjectTarget::LatestRecent",
        "Build latest recent project's Source Engine",
        "Select or add a project before building",
        "Package latest recent project",
        "Select or add a project before packaging",
        "quick_action_enabled(action, &project_target)",
        "use crate::projects::project_paths_match;",
        "use crate::projects::{metadata_for_path, RecentProject};",
        ".find(|project| project_paths_match(&project.path, selected_path))",
        "return QuickActionProjectTarget::StaleSelection;",
        "QuickActionProjectTarget::StaleSelection",
        "fn quick_actions_do_not_fallback_when_selected_project_is_stale()",
        "Selected project is no longer available to build",
        "Selected project unavailable; launch editor without a project",
        "enum ProjectSourceEngineState",
        "fn project_source_engine_state(",
        "metadata_for_path(&snapshot.project_metadata, &project.path)",
        "source_engine_state: project_source_engine_state(project, snapshot)",
        "HubQuickAction::BuildProject => target.has_source_engine()",
        "Bind a Source Engine before building selected project",
        "Bound Source Engine is unavailable for selected project",
        "Bind a Source Engine before building latest recent project",
        "Bound Source Engine is unavailable for latest recent project",
    ] {
        assert!(
            view_model.contains(snippet),
            "QuickActionData projection must describe selected/latest project targeting and disable project-only actions without a project; missing {snippet}"
        );
    }
    assert!(
        !view_model.contains(".find(|project| &project.path == selected_path)"),
        "QuickActionData projection must not use direct PathBuf equality for selected project labels"
    );
    assert!(
        !view_model.contains("HubQuickAction::BuildProject | HubQuickAction::PackageProject"),
        "Build Project quick action must not stay enabled solely because a project exists; it requires a bound Source Engine"
    );

    for snippet in [
        "let had_selected_project = self.selected_project_path.is_some();",
        "if had_selected_project {\n            return None;\n        }",
        "fn selected_or_latest_recent_project_for_named_action(",
        "Selected project is no longer available to build",
        "Selected project is no longer available to package",
        "Selected project is no longer available to install",
        "pub(super) fn selected_project_for_action(&mut self) -> Result<RecentProject, HubError>",
        "pub(super) fn selected_project_with_engine_for_action(",
        "pub(super) fn selected_or_latest_recent_project_with_engine_for_action(",
        "fn require_project_bound_engine(&self, project: &RecentProject) -> Result<(), HubError>",
        "Project has no bound Source Engine: {}",
        "Project bound Source Engine is unavailable: {} -> {}",
        "let selected_before = self.selected_project_path.clone();",
        "return Err(HubError::message(\"No project is selected\"));",
        "self.activate_project_engine_for_path(&project.path);",
        "selected_project_path_changed(\n            selected_before.as_deref(),\n            self.selected_project_path.as_deref(),\n        )",
        "fn refresh_project_context_views(",
        "self.refresh_source_scoped_views()",
        "pub(super) fn open_selected_project_in_editor(",
        "pub(super) fn package_selected_project(",
        "pub(super) fn install_selected_project_to_device(",
        "self.package_selected_project_to_output(ui)?",
    ] {
        assert!(
            workspace.contains(snippet),
            "Builds page actions must require an actual selected project while reusing package/install helpers; missing {snippet}"
        );
    }
    assert!(
        workspace.contains(
            "fn selected_project_path_changed(before: Option<&Path>, after: Option<&Path>) -> bool"
        ) && workspace
            .contains("(Some(before), Some(after)) => !project_paths_match(before, after)"),
        "Builds selected-project action refresh checks must use normalized project path matching"
    );
    assert!(
        !workspace.contains("self.selected_project_path != selected_before"),
        "Builds selected-project action refresh checks must not use direct Option<PathBuf> equality"
    );
    let selected_project_for_action = workspace
        .split("pub(super) fn selected_project_for_action")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn selected_project_with_engine_for_action").next())
        .expect(
            "project_workspace.rs must declare selected_project_for_action before selected_project_with_engine_for_action",
        );
    assert!(
        selected_project_for_action.contains("self.selected_recent_project()")
            && !selected_project_for_action.contains("selected_or_latest_recent_project"),
        "Builds selected-project actions must not use latest-recent fallback"
    );
    for snippet in [
        "pub(super) fn selected_or_latest_recent_project_for_action(",
        "let selected_before = self.selected_project_path.clone();",
        "let active_engine_before = self.config.active_engine_id.clone();",
        "let project = self.selected_or_latest_recent_project();",
        "self.activate_project_engine_for_path(&project.path);",
        "self.config.active_engine_id != active_engine_before",
        "self.refresh_project_context_views(",
    ] {
        assert!(
            workspace.contains(snippet),
            "Quick action latest-recent fallback must refresh selected-project scoped views and Source-scoped Learn when the active engine changes; missing {snippet}"
        );
    }
    assert!(
        workspace
            .matches("selected_or_latest_recent_project_for_action()?")
            .count()
            >= 2,
        "Package and Open Editor quick actions must use the refreshed selected/latest project helper"
    );
    let runtime = read_crate_file("src/app/runtime.rs");
    assert!(
        runtime.contains("fn quick_action_target_does_not_fallback_when_selected_project_is_stale()"),
        "Runtime unit tests must lock Quick Action fallback to no-selection only when selected_project_path becomes stale"
    );
    assert!(
        runtime.contains("fn quick_action_build_reports_stale_selected_project()"),
        "Runtime unit tests must keep stale selected-project quick-action errors explicit"
    );
    for snippet in [
        "load_editor_recent_project_session(&editor_config_path)?",
        "let last_project_path = editor_recent.last_project_path;",
        "startup_selected_project_path(last_project_path.as_deref(), &config.recent_projects)",
        "selected_project_path,",
        "runtime.activate_project_engine_for_path(&path);",
        "fn startup_selected_project_path(",
        "project_paths_match(&project.path, last_project_path)",
    ] {
        assert!(
            runtime.contains(snippet),
            "Hub startup must restore selected_project_path from the Editor last project when it matches a recent project; missing {snippet}"
        );
    }
    let editor_recent_sync = read_crate_file("src/projects/editor_recent_sync.rs");
    for snippet in [
        "use super::metadata::project_metadata_key;",
        "pub struct EditorRecentProjectSession",
        "pub last_project_path: Option<PathBuf>",
        "pub recent_projects: Vec<RecentProject>",
        "pub fn load_editor_recent_project_session(",
        "last_project_path: session.last_project_path.map(PathBuf::from)",
        "let key = project_metadata_key(&entry.path);",
    ] {
        assert!(
            editor_recent_sync.contains(snippet),
            "Editor recent loading must expose last_project_path separately from recent project rows; missing {snippet}"
        );
    }
    assert!(
        !editor_recent_sync.contains("fn recent_project_key("),
        "Editor recent merge must reuse the shared project metadata path key instead of keeping a second normalizer"
    );
    for snippet in [
        "Some(HubQuickAction::BuildProject) => self.build_selected_project_engine(ui)",
        "fn build_selected_project_engine(&mut self, ui: &HubWindow) -> Result<(), HubError>",
        "self.selected_or_latest_recent_project_with_engine_for_action()?",
        "self.refresh_source_scoped_views()?",
        "self.build_editor_runtime_after_sync(ui)",
    ] {
        assert!(
            runtime.contains(snippet),
            "Build Project quick action must target the selected/latest project's bound Source Engine before building; missing {snippet}"
        );
    }
    for snippet in [
        "fn build_selected_project_engine_only(&mut self, ui: &HubWindow) -> Result<(), HubError>",
        "self.selected_project_with_engine_for_action()?",
        "ui.on_build_selected_project_engine",
        "runtime.build_selected_project_engine_only(ui)",
    ] {
        assert!(
            runtime.contains(snippet),
            "Builds page Build action must require an actual selected project instead of using active-engine or latest-recent fallback; missing {snippet}"
        );
    }
    for snippet in [
        "ui.on_launch_selected_project",
        "runtime.open_selected_project_in_editor(ui)",
        "ui.on_package_selected_project",
        "runtime.package_selected_project(ui)",
        "ui.on_install_selected_project",
        "runtime.install_selected_project_to_device(ui)",
    ] {
        assert!(
            runtime.contains(snippet),
            "Runtime must wire Builds selected-project callbacks separately from quick actions; missing {snippet}"
        );
    }
    let open_editor = workspace
        .split("pub(super) fn open_selected_project_or_editor")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn create_project").next())
        .expect("project_workspace.rs must declare open_selected_project_or_editor before create_project");
    assert!(
        open_editor.contains("selected_or_latest_recent_project_for_action()?"),
        "Open Editor quick action must use the same selected-project/latest-recent target rule as package/install"
    );
    assert!(
        !open_editor.contains("selected_recent_project()"),
        "Open Editor quick action must use the selected/latest helper instead of a selected-only project lookup"
    );
    let open_project_path = workspace
        .split("pub(super) fn open_project_path")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn install_recent_project_to_device").next())
        .expect(
            "project_workspace.rs must declare open_project_path before install_recent_project_to_device",
        );
    for snippet in [
        "self.activate_project_engine_for_path(&project_path);",
        "self.remember_project(RecentProject::with_now(display_name.clone(), project_path))?;",
    ] {
        assert!(
            open_project_path.contains(snippet),
            "Opening an existing project must select/remember the project without losing existing engine binding behavior; missing {snippet}"
        );
    }
    assert!(
        !open_project_path.contains("remember_project_metadata_for_path(")
            && !open_project_path.contains("self.config.active_engine_id.clone()"),
        "Opening an existing project must not silently rebind Hub project metadata to the current active Source Engine"
    );
    let create_project = workspace
        .split("pub(super) fn create_project")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn select_new_project_engine_by_id").next())
        .expect("project_workspace.rs must declare create_project before select_new_project_engine_by_id");
    for snippet in [
        "No Source Engine selected for new project",
        "self.require_engine(&engine_id)?;",
        "self.config.active_engine_id = Some(engine_id.clone());",
        "self.sync_settings_from_active_engine();",
        "Some(engine_id)",
    ] {
        assert!(
            create_project.contains(snippet),
            "Creating a new project must require and activate a valid Source Engine before launching Editor; missing {snippet}"
        );
    }
    let confirm_delete = workspace
        .split("pub(super) fn confirm_delete_project")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn activate_project_engine_for_path")
                .next()
        })
        .expect("project_workspace.rs must declare confirm_delete_project before activate_project_engine_for_path");
    for snippet in [
        "recycle_delete_project(path.clone())?;",
        "self.remove_project_from_hub_path(&path);",
        "self.project_subpage = ProjectSubpage::ProjectBrowser;",
        "self.refresh_selected_project_scoped_views()?;",
    ] {
        assert!(
            confirm_delete.contains(snippet),
            "Confirm Delete must clear selected-project scoped Assets/Plugins/Team/Cloud views after removing the selected project; missing {snippet}"
        );
    }
    let recycle_index = confirm_delete
        .find("recycle_delete_project(path.clone())?;")
        .expect("confirm_delete_project must attempt Recycle Bin deletion");
    let remove_index = confirm_delete
        .find("self.remove_project_from_hub_path(&path);")
        .expect("confirm_delete_project must remove Hub metadata after deletion");
    assert!(
        recycle_index < remove_index,
        "Confirm Delete must not remove Hub metadata until the Recycle Bin command has succeeded"
    );
    let remove_project = workspace
        .split("pub(super) fn remove_project_from_hub_path")
        .nth(1)
        .and_then(|source| source.split("fn require_engine").next())
        .expect(
            "project_workspace.rs must declare remove_project_from_hub_path before require_engine",
        );
    for snippet in [
        ".retain(|project| !project_paths_match(&project.path, path))",
        "self.pending_delete_project_path",
        "project_paths_match(pending, path)",
        "project_paths_match(selected, path)",
        "self.pending_delete_project_path = None;",
        "self.selected_project_path = None;",
    ] {
        assert!(
            remove_project.contains(snippet),
            "Removing a project from Hub must clear both selected-project and pending-delete state through normalized path matching; missing {snippet}"
        );
    }
}
