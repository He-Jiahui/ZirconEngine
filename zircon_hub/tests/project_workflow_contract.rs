//! Static contracts for Tauri/React Hub project workflow routing.
//!
//! Focused path-scope, Source Engine, page-copy, and quick-action assertions
//! live in companion integration tests. This file keeps the workflow seams that
//! span those companions from regressing back to broad fallback behavior.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub crate should live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}")),
    )
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain project-workflow snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete project-workflow snippet {snippet:?}"
        );
    }
}

#[test]
fn bundled_hub_config_and_runtime_defaults_keep_chinese_as_first_launch_language() {
    let bundled_config = read_crate_file("hub.toml");
    let config = read_crate_file("src/settings/hub_config.rs");
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all("hub.toml", &bundled_config, &["language = \"Chinese\""]);
    assert_not_contains_any("hub.toml", &bundled_config, &["language = \"English\""]);
    assert_contains_all(
        "hub_config.rs",
        &config,
        &["English,", "#[default]\n    Chinese,"],
    );
    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &["first launch language stays Chinese by both Rust default and bundled `hub.toml`"],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &["first launch language stays Chinese by both Rust default and bundled `hub.toml`"],
    );
}

#[test]
fn tauri_runtime_routes_project_workflow_actions_and_persists_state() {
    let action_request = read_crate_file("src/tauri_app/action_request.rs");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let action_tasks = read_crate_file("src/tauri_app/runtime_state/action_tasks.rs");
    let project_actions = read_crate_file("src/tauri_app/runtime_state/project_actions.rs");
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");
    let output_actions = read_crate_file("src/tauri_app/runtime_state/output_actions.rs");
    let settings_actions = read_crate_file("src/tauri_app/runtime_state/settings_actions.rs");
    let settings_dto = read_crate_file("src/tauri_app/view_model/settings_dto.rs");
    let config = read_crate_file("src/settings/hub_config.rs");
    let commands = read_crate_file("src/tauri_app/commands.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let action_history_dto = read_crate_file("src/tauri_app/view_model/action_history.rs");

    assert_contains_all(
        "action_request.rs",
        &action_request,
        &[
            "pub(crate) struct HubActionRequest",
            "pub action_id: String",
            "pub target_id: Option<String>",
            "pub payload: Option<Value>",
            "pub(crate) enum HubAction",
            "CreateProjectActionPayload",
            "ImportProjectActionPayload",
            "BrowseSettingsFolderPayload",
            "OpenResourcePayload",
            "OpenOutputFolderPayload",
            "pub history_id: Option<String>",
            "pub settings: Option<HubSettingsPayload>",
            "pub(crate) fn parse(&self) -> Result<HubAction, HubError>",
            "\"show-page\" | \"page\" => Ok(HubAction::ShowPage",
            "\"select-project\" | \"open-project\" => Ok(HubAction::SelectProject",
            "\"save-settings\" => Ok(HubAction::SaveSettings",
            "payload: settings_payload_from_value(self.payload.as_ref())?",
            "\"create-project\" => Ok(HubAction::CreateProject",
            "payload: create_project_payload_from_value(self.payload.as_ref())?",
            "\"open-resource\" => Ok(HubAction::OpenResource",
            "payload: open_resource_payload_from_value(self.payload.as_ref())?",
            "\"open-output-folder\" => Ok(HubAction::OpenOutputFolder",
            "payload: open_output_folder_payload_from_value(self.payload.as_ref())?",
            "\"cancel-delete\" => Ok(HubAction::CancelDelete",
            "parses_cancel_delete_project_target_payload",
            "parses_create_project_payload_for_create_project_action",
            "parses_browse_settings_folder_payload_for_folder_action",
            "parses_open_output_folder_wrapped_payload_for_output_action",
            "unknown_action_is_rejected_before_runtime_routing",
        ],
    );
    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "pub(super) fn apply_action(",
            "request: HubActionRequest",
            "match request.parse()? {",
            "HubAction::ShowPage { target_id } => self.select_page_by_id(&target_id)?",
            "HubAction::ShowProjectSubpage { target_id } =>",
            "HubAction::SearchProjects { query } => self.search_projects(&query)",
            "HubAction::SetProjectFilter { target_id } =>",
            "HubAction::SetProjectSort { target_id } => self.set_project_sort_by_id(&target_id)?",
            "HubAction::SetProjectViewMode { target_id } =>",
            "HubAction::SelectProject { target_id } => self.select_project_target(&target_id)?",
            "HubAction::OpenProjectDetail { target_id } => self.open_project_detail(&target_id)?",
            "HubAction::ViewAllProjects => self.view_all_projects()",
            "HubAction::NewProject =>",
            "HubAction::SelectEngine { target_id } => self.select_engine_by_id(&target_id)?",
            "HubAction::SaveSettings { payload } => self.save_settings_from_action(payload)?",
            "HubAction::BrowseSettingsFolder { target_id, payload } =>",
            "self.browse_settings_folder(target_id.as_deref(), payload)?",
            "HubAction::CreateProject { payload } => self.create_project_from_payload(payload)?",
            "HubAction::ImportProject { target_id, payload } =>",
            "HubAction::CancelDelete { target_id, payload } =>",
            "self.cancel_project_delete(target_id.as_deref(), payload.as_ref())?",
            "HubAction::OpenResource { target_id, payload } =>",
            "HubAction::OpenOutputFolder { target_id, payload } =>",
            "HubAction::BuildProject { target_id, payload } =>",
            "payload.as_ref(),",
            "\"build-project\",",
            "self.build_selected_project_engine()?",
            "HubAction::PackageProject { target_id, payload } =>",
            "\"package-project\",",
            "self.package_recent_project()?",
            "HubAction::InstallDevice { target_id, payload } =>",
            "\"install-device\",",
            "self.install_recent_project_to_device()?",
            "HubAction::OpenEditor { target_id, payload } =>",
            "\"open-editor\",",
            "self.open_selected_project_or_editor()?",
            "Ok(self.view_model())",
            "fn persist_hub_config(&self) -> Result<(), HubError>",
            "config.runtime = self.runtime_state_for_config();",
            "fn persist_with_last_project(&self, last_project_path: Option<&Path>) -> Result<(), HubError>",
            "save_editor_recent_projects_with_last_project(",
            "fn runtime_state_for_config(&self) -> HubRuntimeState",
            "self.register_source_engine_from_settings();",
            "self.refresh_source_scoped_views()?;",
            "save_settings_refreshes_source_scoped_catalogs_in_returned_view_model",
            "const VISUAL_TASK_STATE_ENV: &str = \"ZIRCON_HUB_VISUAL_TASK_STATE\";",
            "fn apply_visual_task_state_override_from_env(&mut self)",
            "TaskStatus::warning(",
        ],
    );
    assert_contains_all(
        "runtime_state/action_tasks.rs",
        &action_tasks,
        &[
            "enum BackgroundHubAction",
            "\"build-project\" => Some(Self::BuildProject)",
            "\"package-project\" => Some(Self::PackageProject)",
            "\"install-device\" => Some(Self::InstallDevice)",
            "\"open-editor\" => Some(Self::OpenEditor)",
            "TaskStatus::running_operation(",
            "pub(in crate::tauri_app) fn should_run_action_in_background",
            "pub(in crate::tauri_app) fn start_background_action_status",
            "background_worker_active",
            "background_action_queue",
            "pub(in crate::tauri_app) fn take_next_background_action",
            "pub(in crate::tauri_app) fn record_background_action_error",
        ],
    );
    assert_contains_all(
        "runtime_state/build_actions.rs",
        &build_actions,
        &[
            "pub(in crate::tauri_app) struct PendingEditorRuntimeBuild",
            "pub(in crate::tauri_app) fn prepare_background_editor_runtime_build",
            "pub(in crate::tauri_app) fn complete_background_editor_runtime_build",
            "let result = run_build_command(pending_build.command())",
            "record_active_build(",
            "background_build_prepares_command_without_running_or_recording_history",
            "background_build_completion_records_success_after_external_result",
        ],
    );
    assert_contains_all(
        "runtime_state/editor_launch_actions.rs",
        &editor_launch_actions,
        &[
            "pub(in crate::tauri_app) struct PendingEditorLaunch",
            "pub(super) fn open_selected_project_or_editor(&mut self) -> Result<(), HubError>",
            "pub(in crate::tauri_app) fn prepare_background_editor_launch",
            "pub(in crate::tauri_app) fn complete_background_editor_launch",
            "launch_editor(command)?",
            "Command::new(executable).spawn()?",
            "record_editor_launch_failure(",
            "background_editor_launch_prepare_records_missing_executable_failure_without_spawn",
            "background_editor_launch_completion_records_success_after_external_spawn",
        ],
    );
    assert_contains_all(
        "runtime_state/project_delivery_actions.rs",
        &project_delivery_actions,
        &[
            "pub(in crate::tauri_app) struct PendingProjectPackage",
            "pub(in crate::tauri_app) struct PendingDeviceInstall",
            "pub(in crate::tauri_app) fn prepare_background_project_package",
            "pub(in crate::tauri_app) fn complete_background_project_package",
            "pub(in crate::tauri_app) fn prepare_background_device_install",
            "pub(in crate::tauri_app) fn complete_background_device_install",
            "package_project(&self.request)",
            "install_package_to_device(&install_request)",
            "record_package_success(",
            "background_package_prepares_request_without_copying_or_recording_history",
            "background_package_completion_records_success_after_copy_result",
            "background_install_runs_package_then_device_copy_before_recording_history",
        ],
    );
    assert_contains_all(
        "runtime_state/project_actions.rs",
        &project_actions,
        &[
            "pub(super) fn import_project_from_action(",
            "CreateProjectActionPayload",
            "ImportProjectActionPayload",
            "FolderPickerRequest::new(",
            "HubTextBundle::new(self.config.settings.language)",
            "import_project_picker_title(text)",
            "fn import_project_picker_title(text: HubTextBundle) -> &'static str",
            "text.pair(\"Import Zircon Project\", \"导入 Zircon 项目\")",
            "import_project_folder_picker_title_uses_current_language",
        ],
    );
    assert_not_contains_any(
        "runtime_state/project_actions.rs",
        &project_actions,
        &["FolderPickerRequest::new(\n                \"Import Zircon Project\""],
    );
    assert_contains_all(
        "runtime_state/output_actions.rs",
        &output_actions,
        &[
            "pub(super) fn open_output_folder(",
            "OpenOutputFolderPayload",
            "if let Some(output_dir) = payload.output_dir.clone()",
            "if let Some(path) = payload.path.clone()",
            "action_history_id(record) == target",
            "OpenFolderCommand::new(output_dir.clone())",
            "HubActionKind::OpenOutput",
            "TaskStatus::success(\"Output folder opened\"",
            "TaskStatus::error(\"Open Output failed\"",
            "record.action.id()",
            "open_output_folder_resolves_record_id_before_path_fallback",
            "open_output_folder_prefers_typed_output_dir_over_legacy_path_payload",
            "open_output_folder_missing_directory_is_recoverable_status",
        ],
    );
    assert_not_contains_any(
        "runtime_state/output_actions.rs",
        &output_actions,
        &["payload.path.clone().or(payload.output_dir.clone())"],
    );
    assert_contains_all(
        "runtime_state/settings_actions.rs",
        &settings_actions,
        &[
            "pub(super) fn browse_settings_folder(",
            "pub(super) fn save_settings_from_action(",
            "BrowseSettingsFolderPayload",
            "settings_payload: Option<HubSettingsPayload>",
            "self.save_settings(settings_payload)",
            "record_settings_save_failure",
            "text.status_label(\"Save Settings failed\")",
            "text.status_detail(\"Check Settings values and save again\")",
            "FolderPickerRequest::new(",
            "field.picker_title(text)",
            "field.set_path(&mut self.settings_draft",
            "HubTextBundle::new(self.settings_draft.language)",
            "text.pair(\"Choose Default Project Directory\", \"选择默认项目目录\")",
            "text.status_label(\"Folder selected\")",
            "text.status_label(\"Folder selection cancelled\")",
            "text.status_label(\"Browse folder failed\")",
            "text.status_detail(\"Choose an existing local folder or type the path manually\")",
            "settings_draft_folder_changes_wait_for_save_settings",
            "settings_folder_picker_title_uses_current_language",
            "save_settings_validation_errors_return_localized_view_model",
        ],
    );
    assert_contains_all(
        "view_model/settings_dto.rs",
        &settings_dto,
        &[
            "pub(crate) struct HubSettingsHealthSummary",
            "pub(crate) struct HubSettingsHealthRow",
            "\"python-path\"",
            "&settings.python_path",
            "\"cargo-path\"",
            "&settings.cargo_path",
            "\"rustup-path\"",
            "&settings.rustup_path",
            "fn executable_row(",
            "path_command_exists(trimmed)",
            "fn path_command_exists(command: &str) -> bool",
            "env::split_paths(&path_var)",
            "fn path_command_extensions(has_extension: bool)",
            "env::var_os(\"PATHEXT\")",
            "fn directory_row(",
            "settings_health_includes_rustup_path_status",
            "settings_health_checks_path_command_availability",
        ],
    );
    assert_contains_all(
        "hub_config.rs",
        &config,
        &[
            "pub runtime: HubRuntimeState,",
            "pub struct HubRuntimeState",
            "pub selected_page: HubPage,",
            "pub project_subpage: ProjectSubpage,",
            "pub project_filter: ProjectFilterMode,",
            "pub project_sort: ProjectSortMode,",
            "pub project_view_mode: ProjectViewMode,",
            "pub search_query: String,",
            "pub selected_project_path: Option<PathBuf>,",
            "pub new_project_engine_id: Option<String>,",
            "pub fn normalize(&mut self)",
        ],
    );
    assert_contains_all(
        "commands.rs",
        &commands,
        &[
            "pub(super) fn hub_state(",
            "Ok(session.view_model())",
            "pub(super) fn hub_action(",
            "if HubRuntimeSession::should_run_action_in_background(&request)",
            "let should_spawn = session.start_background_action_or_record_error(&request)?;",
            "spawn_background_action(request, session_handle, app.clone());",
            "fn spawn_background_action(",
            "run_background_build_action(request, session_handle, app);",
            "run_background_package_action(request, session_handle, app);",
            "run_background_install_action(request, session_handle, app);",
            "run_background_editor_action(request, session_handle, app);",
            "fn run_background_build_action(",
            "session.prepare_background_editor_runtime_build()",
            "run_build_command(pending_build.command())",
            "session.complete_background_editor_runtime_build(",
            "fn run_background_package_action(",
            "session.prepare_background_project_package()",
            "pending_package.run()",
            "session.complete_background_project_package(",
            "fn run_background_install_action(",
            "session.prepare_background_device_install()",
            "pending_install.run()",
            "session.complete_background_device_install(",
            "fn run_background_editor_action(",
            "session.prepare_background_editor_launch()",
            "pending_launch.run()",
            "session.complete_background_editor_launch(",
            "emit_current_state_and_continue(session_handle, app);",
            "fn emit_and_continue(",
            "fn continue_background_queue(",
            "session.take_next_background_action()",
            "let view_model = match session.apply_action(request.clone())",
            "session.record_background_action_error(&request, error.to_string())",
            "app.emit(\"hub-state-changed\", &view_model)",
        ],
    );
    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "mod action_history;",
            "use action_history::{action_history_rows, HubActionHistoryItem};",
            "pub(crate) struct HubViewModel",
            "pub project_subpage: String",
            "pub quick_actions: Vec<HubQuickAction>",
            "pub action_history: Vec<HubActionHistoryItem>",
            "action_history: action_history_rows(",
            "snapshot.settings.language",
        ],
    );
    assert_contains_all(
        "view_model/action_history.rs",
        &action_history_dto,
        &[
            "pub(crate) struct HubActionHistoryItem",
            "pub kind: String",
            "record.action.id()",
            "kind: record.action.id().to_string()",
            "let text = HubTextBundle::new(language);",
            "action: text.action_label(record.action).to_string()",
            "status: text.action_status_label(record.status).to_string()",
            "let detail = text.status_detail(&record.detail);",
            "let log_excerpt = text.status_detail(&record.log_excerpt);",
            "let detail_rows = action_history_detail_rows(",
            ".map(|recovery| text.status_detail(recovery))",
            "let finished = relative_time(now_ms, record.finished_unix_ms, language);",
        ],
    );
}

#[test]
fn project_selection_detail_and_source_context_refresh_before_view_model() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let snapshot = read_crate_file("src/state/hub_snapshot.rs");
    let scope = read_crate_file("src/state/scope.rs");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "fn select_project_target(&mut self, target: &str) -> Result<(), HubError>",
            "let active_engine_before = self.config.active_engine_id.clone();",
            "self.selected_project_path = Some(project.path.clone());",
            "self.activate_project_engine_for_path(&project.path);",
            "self.refresh_project_context_views(",
            "self.config.active_engine_id != active_engine_before",
            "self.persist_with_last_project(Some(&project.path))",
            "fn open_project_detail(&mut self, target: &str) -> Result<(), HubError>",
            "self.select_project_target(target)?;",
            "self.project_subpage = ProjectSubpage::ProjectDetail;",
            "self.project_view_mode = ProjectViewMode::List;",
            "fn view_all_projects(&mut self)",
            "self.search_query.clear();",
            "self.project_subpage = ProjectSubpage::ProjectBrowser;",
            "fn refresh_project_context_views(",
            "if active_engine_changed {",
            "self.refresh_source_scoped_views()",
            "} else if selected_project_changed {",
            "self.refresh_selected_project_scoped_views()",
        ],
    );
    assert_contains_all(
        "hub_snapshot.rs",
        &snapshot,
        &[
            "pub fn scope(&self) -> HubScope",
            "HubScope::resolve(",
            "pub fn filtered_recent_projects(&self) -> Vec<RecentProject>",
            "self.project_filter.includes(project)",
            "project_matches_query(project, &query)",
        ],
    );
    assert_contains_all(
        "scope.rs",
        &scope,
        &[
            "pub struct HubScope",
            "pub enum ProjectScope",
            "Selected(ProjectScopeProject)",
            "StaleSelection { requested_path: PathBuf }",
            "LatestRecent(ProjectScopeProject)",
            "pub enum SourceEngineScope",
            "ProjectBound(SourceEngineScopeEngine)",
            "ProjectUnbound {",
            "ProjectEngineUnavailable {",
        ],
    );
}

#[test]
fn backend_workflow_actions_record_history_and_visible_task_status() {
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");
    let quick_actions = read_crate_file("src/tauri_app/runtime_state/quick_actions.rs");
    let output_actions = read_crate_file("src/tauri_app/runtime_state/output_actions.rs");
    let action_history = read_crate_file("src/state/action_history.rs");
    let task_status = read_crate_file("src/state/task_status.rs");

    assert_contains_all(
        "runtime_state/build_actions.rs",
        &build_actions,
        &[
            "pub(super) fn build_selected_project_engine(&mut self) -> Result<(), HubError>",
            "fn prepare_editor_runtime_build(&mut self) -> Result<PendingEditorRuntimeBuild, HubError>",
            "self.validate_active_source_engine_for_build(command_line.clone())?",
            "fn complete_editor_runtime_build(",
            "self.record_action_and_persist(HubActionRecord",
            "TaskStatus::running_operation(",
            "TaskStatus::error(",
            "TaskStatus::success(",
        ],
    );
    assert_contains_all(
        "runtime_state/quick_actions.rs",
        &quick_actions,
        &[
            "TaskStatus::error(",
            "pub(super) fn record_action_and_persist(",
            "open_editor_action_records_recoverable_failure_without_falling_back_to_demo_state",
        ],
    );
    assert_contains_all(
        "runtime_state/editor_launch_actions.rs",
        &editor_launch_actions,
        &[
            "pub(super) fn open_selected_project_or_editor(&mut self) -> Result<(), HubError>",
            "EditorLaunchCommand::from_preferred_engine(",
            "record_editor_launch_failure(",
            "TaskStatus::success(",
            "pub(in crate::tauri_app) fn prepare_background_editor_launch",
            "pub(in crate::tauri_app) fn complete_background_editor_launch",
            "background_editor_launch_completion_records_success_after_external_spawn",
        ],
    );
    assert_contains_all(
        "runtime_state/project_delivery_actions.rs",
        &project_delivery_actions,
        &[
            "pub(super) fn package_recent_project(&mut self) -> Result<(), HubError>",
            "ProjectPackageRequest::new(",
            "record_package_success(",
            "pub(super) fn install_recent_project_to_device(&mut self) -> Result<(), HubError>",
            "DeviceInstallRequest::new(",
            "HubActionKind::InstallProject",
            "record_project_action_failure(",
            "pub(in crate::tauri_app) fn prepare_background_project_package",
            "pub(in crate::tauri_app) fn complete_background_project_package",
            "pub(in crate::tauri_app) fn prepare_background_device_install",
            "pub(in crate::tauri_app) fn complete_background_device_install",
            "background_package_completion_records_success_after_copy_result",
            "background_install_runs_package_then_device_copy_before_recording_history",
        ],
    );
    assert_contains_all(
        "runtime_state/output_actions.rs",
        &output_actions,
        &[
            "open_folder(&command)",
            "process_id: Some(process_id)",
            "command_line,",
            "output_dir: Some(output_dir.clone())",
            "record_output_folder_failure(",
            "Open the folder manually from the file system and verify shell integration",
        ],
    );
    assert_contains_all(
        "action_history.rs",
        &action_history,
        &[
            "pub struct HubActionRecord",
            "pub enum HubActionKind",
            "BuildEditorRuntime",
            "OpenEditor",
            "PackageProject",
            "InstallProject",
            "pub fn label(self) -> &'static str",
            "pub fn push_action_record(history: &mut Vec<HubActionRecord>, record: HubActionRecord)",
        ],
    );
    assert_contains_all(
        "task_status.rs",
        &task_status,
        &[
            "pub struct TaskStatus",
            "pub fn running_operation(",
            "pub fn success(",
            "pub fn warning(",
            "pub fn error(",
            "pub fn with_operation(",
            "pub fn operation_summary(&self) -> String",
            "pub fn detail_with_recovery(&self) -> String",
        ],
    );
}

#[test]
fn react_pages_dispatch_project_workflows_through_single_action_api() {
    let app = read_crate_file("web/src/App.tsx");
    let hub_api = read_crate_file("web/src/tauri/hubApi.ts");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const handleAction: HubActionHandler = async (actionId, targetId, payload) =>",
            "const nextState = await dispatchHubAction(actionId, targetId, payload);",
            "setState(nextState);",
            "const shellText = stateRef.current.ui.shell;",
            "label: shellText.actionFailed",
            "detail: shellText.actionFailedDetail",
            "<HubWindow state={state} onAction={handleAction} />",
        ],
    );
    assert_contains_all(
        "hubApi.ts",
        &hub_api,
        &[
            "return await invoke<HubShellState>(\"hub_action\", {",
            "request: { actionId, targetId, payload },",
            "return await invoke<HubShellState>(\"hub_state\");",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "openOutputFolder: \"open-output-folder\"",
            "browseSettingsFolder: \"browse-settings-folder\"",
            "export type HubActionHistoryKind =",
            "kind: HubActionHistoryKind;",
            "export type HubSettingsFolderField =",
            "export interface SearchProjectsPayload",
            "export interface ProjectTargetPayload",
            "projectId?: string;",
            "projectPath?: string;",
            "export interface BrowseSettingsFolderPayload",
            "settings?: Partial<HubSettingsSummary>;",
            "export interface OpenOutputFolderPayload {\n  outputDir?: string;\n  historyId?: string;\n}",
            "historyId?: string;",
            "[HUB_ACTION.searchProjects]: SearchProjectsPayload;",
            "[HUB_ACTION.buildProject]: ProjectTargetPayload;",
            "[HUB_ACTION.pinProject]: ProjectTargetPayload;",
            "[HUB_ACTION.unpinProject]: ProjectTargetPayload;",
            "[HUB_ACTION.removeFromHub]: ProjectTargetPayload;",
            "[HUB_ACTION.requestDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.cancelDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.confirmDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.packageProject]: ProjectTargetPayload;",
            "[HUB_ACTION.installDevice]: ProjectTargetPayload;",
            "[HUB_ACTION.openEditor]: ProjectTargetPayload;",
            "[HUB_ACTION.browseSettingsFolder]: BrowseSettingsFolderPayload;",
            "[HUB_ACTION.openOutputFolder]: OpenOutputFolderPayload;",
            "settingsDraft?: HubSettingsSummary | null;",
        ],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "void onAction(HUB_ACTION.setProjectFilter, value);",
            "void onAction(HUB_ACTION.setProjectSort, value);",
            "void onAction(HUB_ACTION.setProjectViewMode, value);",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
            "void onAction(HUB_ACTION.newProject)",
            "state.projectSubpage === \"new-project\"",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "void onAction(HUB_ACTION.showProjectSubpage, \"dashboard\")",
            "void onAction(HUB_ACTION.newProject)",
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "void onAction(HUB_ACTION.selectProject, project.id)",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "void onAction(HUB_ACTION.viewAllProjects)",
            "void onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
            "void onAction(action.id, undefined, quickActionProjectTarget)",
            "void onAction(HUB_ACTION.packageProject, undefined, projectTarget)",
            "void onAction(HUB_ACTION.installDevice, undefined, projectTarget)",
            "void onAction(project.pinned ? HUB_ACTION.unpinProject : HUB_ACTION.pinProject, undefined, projectTarget)",
            "void onAction(HUB_ACTION.removeFromHub, undefined, projectTarget)",
            "void onAction(HUB_ACTION.requestDelete, undefined, projectTarget)",
            "void onAction(HUB_ACTION.cancelDelete, undefined, projectTarget)",
            "void onAction(HUB_ACTION.confirmDelete, undefined, projectTarget)",
        ],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const buildActionKinds: HubActionHistoryItem[\"kind\"][] =",
            "buildActionKinds.includes(action.kind)",
            "action.kind === \"install-project\"",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "void onAction(HUB_ACTION.buildProject, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)",
            "void onAction(actionId, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.openOutputFolder, item.id, { historyId: item.id })",
            "item.id === \"build-output-root\"",
            "state.settings.defaultBuildOutputDir",
            "state.settings.defaultDeviceInstallDir",
            "void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir });",
            "void onAction(action.id, undefined, quickActionProjectTarget)",
        ],
    );
    assert_not_contains_any(
        "BuildsPage.tsx",
        &builds,
        &["void onAction(HUB_ACTION.openOutputFolder, undefined, { path: item.detail })"],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "action.kind === \"package-project\"",
            "action.kind === \"install-project\"",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.openOutputFolder, item.id, { historyId: item.id })",
            "void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir: state.settings.defaultBuildOutputDir });",
            "void onAction(action.id, undefined, quickActionProjectTarget)",
        ],
    );
    assert_not_contains_any(
        "CloudPage.tsx",
        &cloud,
        &["void onAction(HUB_ACTION.openOutputFolder, undefined, { path: item.detail })"],
    );
    assert_contains_all(
        "EditorPage.tsx",
        &editor,
        &[
            "action.kind === \"open-editor\" || action.kind === \"build-editor-runtime\"",
            "const activeSourceEngine = state.sourceEngines.find((engine) => engine.active) ?? state.sourceEngines[0];",
            "const projectTarget = projectTargetPayload(project);",
            "void onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
            "void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir: activeSourceEngine?.outputPath })",
        ],
    );
    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "settingsDraftState(state)",
            "state.settingsDraft ?? state.settings",
            "void onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft });",
            "state.ui.actions.browseFolder",
            "settingsText.buildProfileOptions",
            "settingsText.languageOptions",
            "SettingsPathField",
            "HubIconButton",
        ],
    );
}

#[test]
fn project_search_uses_typed_payload_instead_of_target_id() {
    let action_request = read_crate_file("src/tauri_app/action_request.rs");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let types = read_crate_file("web/src/types/hub.ts");
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "action_request.rs",
        &action_request,
        &[
            "pub(crate) struct SearchProjectsPayload",
            "query: search_projects_payload_from_value(",
            "self.target_id.as_deref(),",
            "parses_search_projects_typed_payload_before_target_fallback",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface SearchProjectsPayload",
            "query: string;",
            "[HUB_ACTION.searchProjects]: SearchProjectsPayload;",
        ],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &["void onAction(HUB_ACTION.searchProjects, undefined, { query: value });"],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &["void onAction(HUB_ACTION.searchProjects, undefined, { query: value });"],
    );
    assert_not_contains_any(
        "project pages",
        &(dashboard + &browser),
        &["void onAction(HUB_ACTION.searchProjects, value);"],
    );
    assert_contains_all(
        "Hub docs",
        &(shell_doc + &responsive_doc),
        &["search-projects` sends typed `{ query }` payloads"],
    );
}

#[test]
fn project_workflow_documentation_records_tauri_react_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/project_workflow_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test project_workflow_contract",
            "## Project Workflow Contract Cutover",
            "React/MUI project workflow routing",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/runtime_state/action_tasks.rs",
            "src/tauri_app/runtime_state/project_actions.rs",
            "src/tauri_app/runtime_state/output_actions.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "src/tauri_app/runtime_state/settings_actions.rs",
            "localized Import Project folder picker",
            "web/src/tauri/hubApi.ts",
            "web/src/pages/ProjectsDashboard.tsx",
            "open-output-folder",
            "browse-settings-folder",
            "settingsDraft",
            "single action dispatcher and refreshed HubViewModel",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`project_workflow_contract.rs`",
            "React/MUI project workflow routing",
            "browse-settings-folder",
            "settingsDraft",
            "single action dispatcher and refreshed HubViewModel",
        ],
    );
}

#[test]
fn project_workflow_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/project_workflow_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "project_workflow_contract.rs",
        &contract,
        &[
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/runtime_state/action_tasks.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "src/tauri_app/runtime_state/settings_actions.rs",
            "src/tauri_app/commands.rs",
            "web/src/App.tsx",
            "web/src/tauri/hubApi.ts",
            "web/src/pages/ProjectsDashboard.tsx",
        ],
    );
    assert_not_contains_any(
        "project_workflow_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_taffy_name.as_str(),
        ],
    );
}
