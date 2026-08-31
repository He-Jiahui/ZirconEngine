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
    let action_id = read_crate_file("src/tauri_app/action_id.rs");
    let action_request = read_crate_file("src/tauri_app/action_request.rs");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let runtime_state_tests = read_crate_file("src/tauri_app/runtime_state/tests.rs");
    let action_tasks = read_crate_file("src/tauri_app/runtime_state/action_tasks.rs");
    let project_actions = read_crate_file("src/tauri_app/runtime_state/project_actions.rs");
    let project_action_tests =
        read_crate_file("src/tauri_app/runtime_state/project_actions/tests.rs");
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");
    let device_install = read_crate_file("src/projects/device_install.rs");
    let install_receipt = read_crate_file("src/projects/install_receipt.rs");
    let output_actions = read_crate_file("src/tauri_app/runtime_state/output_actions.rs");
    let settings_actions = read_crate_file("src/tauri_app/runtime_state/settings_actions.rs");
    let settings_dto = read_crate_file("src/tauri_app/view_model/settings_dto.rs");
    let config = read_crate_file("src/settings/hub_config.rs");
    let commands = read_crate_file("src/tauri_app/commands.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let action_history_dto = read_crate_file("src/tauri_app/view_model/action_history.rs");

    assert_contains_all(
        "action_id.rs",
        &action_id,
        &[
            "pub(crate) enum HubActionId",
            "pub(crate) const ALL: [HubActionId; 31]",
            "Self::BuildProject => \"build-project\"",
            "Self::PackageProject => \"package-project\"",
            "Self::InstallDevice => \"install-device\"",
            "Self::OpenEditor => \"open-editor\"",
            "\"page\" => Some(Self::ShowPage)",
            "\"project-subpage\" => Some(Self::ShowProjectSubpage)",
            "\"open-project\" => Some(Self::SelectProject)",
            "every_action_id_round_trips_between_as_str_and_from_str",
        ],
    );
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
            "pub(crate) fn action(&self) -> Result<HubActionId, HubError>",
            "pub(in crate::tauri_app) fn parse_as(",
            "pub(crate) trait ValidatePayload",
            "fn parse_payload<T>(action: HubActionId, payload: Option<&Value>) -> Result<T, HubError>",
            "fn parse_optional_payload<T>(",
            "HubActionId::ShowPage => Ok(HubAction::ShowPage",
            "HubActionId::SelectProject => Ok(HubAction::SelectProject",
            "HubActionId::UpdateSettingsDraft => Ok(HubAction::UpdateSettingsDraft",
            "HubActionId::SaveSettings => Ok(HubAction::SaveSettings",
            "payload: parse_payload::<HubSettingsActionPayload>(action, self.payload.as_ref())?",
            "payload: parse_optional_payload::<HubSettingsActionPayload>(",
            "HubActionId::CreateProject => Ok(HubAction::CreateProject",
            "payload: parse_payload(action, self.payload.as_ref())?",
            "HubActionId::OpenResource => Ok(HubAction::OpenResource",
            "HubActionId::OpenOutputFolder => Ok(HubAction::OpenOutputFolder",
            "payload: parse_optional_payload(action, self.payload.as_ref())?",
            "impl ValidatePayload for CreateProjectActionPayload",
            "impl ValidatePayload for ProjectTargetActionPayload",
            "project_target_envelope_payload_is_rejected_after_hard_cutover",
            "missing_required_payload_is_rejected_with_action_id",
            "settings_payload_requires_settings_wrapper",
            "HubActionId::CancelDelete => Ok(HubAction::CancelDelete",
            "parses_cancel_delete_project_target_payload",
            "parses_create_project_payload_for_create_project_action",
            "parses_browse_settings_folder_payload_for_folder_action",
            "parses_open_output_folder_flat_payload_for_output_action",
            "unknown_action_is_rejected_before_runtime_routing",
        ],
    );
    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "pub(super) fn apply_action(",
            "request: HubActionRequest",
            "let action_id = request.action()?;",
            "request.parse_as(action_id)",
            "self.record_action_payload_failure(action_id, error)?;",
            "fn record_action_payload_failure(",
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
            "HubAction::UpdateSettingsDraft { payload } =>",
            "self.update_settings_draft_from_action(payload)?",
            "HubAction::SaveSettings { payload } => self.save_settings_from_action(payload)?",
            "HubAction::DiscardSettingsDraft => self.discard_settings_draft()",
            "HubAction::RestoreDefaultSettings => self.restore_default_settings()",
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
            "HubActionId::BuildProject,",
            "self.build_selected_project_engine()?",
            "HubAction::PackageProject { target_id, payload } =>",
            "HubActionId::PackageProject,",
            "self.package_recent_project()?",
            "HubAction::InstallDevice { target_id, payload } =>",
            "HubActionId::InstallDevice,",
            "self.install_recent_project_to_device()?",
            "HubAction::OpenEditor { target_id, payload } =>",
            "HubActionId::OpenEditor,",
            "self.open_selected_project_or_editor()?",
            "Ok(self.view_model())",
            "fn persist(&mut self) -> Result<(), HubError>",
            "config.runtime = self.runtime_state_for_config();",
            "fn persist_unchecked(&mut self) -> Result<(), HubError>",
            "reconcile_shared_recent_projects(",
            "shared_recent_projects_snapshot",
            "fn refresh_shared_recent_projects_on_focus(&mut self) -> Result<bool, HubError>",
            "fn runtime_state_for_config(&self) -> HubRuntimeState",
            "self.register_source_engine_from_settings()",
            "validate_settings_for_save(&settings)",
            "self.refresh_source_scoped_views()?;",
            "const VISUAL_TASK_STATE_ENV: &str = \"ZIRCON_HUB_VISUAL_TASK_STATE\";",
            "fn apply_visual_task_state_override_from_env(&mut self)",
            "TaskStatus::warning(",
        ],
    );
    assert_contains_all(
        "runtime_state/tests.rs",
        &runtime_state_tests,
        &[
            "save_settings_refreshes_source_scoped_catalogs_in_returned_view_model",
            "apply_action_records_payload_validation_failure_as_recoverable_status",
        ],
    );
    assert_contains_all(
        "runtime_state/action_tasks.rs",
        &action_tasks,
        &[
            "enum BackgroundHubAction",
            "HubActionId::BuildProject => Some(Self::BuildProject)",
            "HubActionId::PackageProject => Some(Self::PackageProject)",
            "HubActionId::InstallDevice => Some(Self::InstallDevice)",
            "HubActionId::OpenEditor => Some(Self::OpenEditor)",
            "TaskStatus::running_operation(",
            "pub(in crate::tauri_app) trait BackgroundTask",
            "pub(in crate::tauri_app) fn execute_background_task",
            "pub(in crate::tauri_app) fn dispatch_background_request",
            "pub(in crate::tauri_app) fn run_background_worker_loop",
            "pub(in crate::tauri_app) fn lock_session",
            "panic::catch_unwind",
            "pub(in crate::tauri_app) fn should_run_action_in_background",
            "pub(in crate::tauri_app) fn start_background_action_status",
            "background_worker_active",
            "background_action_queue",
            "pub(in crate::tauri_app) fn take_next_background_action",
            "pub(in crate::tauri_app) fn record_background_action_error",
            "pub(in crate::tauri_app) fn record_background_worker_panic",
        ],
    );
    assert_contains_all(
        "runtime_state/build_actions.rs",
        &build_actions,
        &[
            "pub(in crate::tauri_app) struct PendingEditorRuntimeBuild",
            "impl BackgroundTask for PendingEditorRuntimeBuild",
            "pub(in crate::tauri_app) fn prepare_background_editor_runtime_build",
            "pub(in crate::tauri_app) fn complete_background_editor_runtime_build",
            "let result = pending_build.run()",
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
            "report.receipt_path",
            "record_package_success(",
            "background_package_prepares_request_without_copying_or_recording_history",
            "background_package_completion_records_success_after_copy_result",
            "background_install_runs_package_then_device_copy_before_recording_history",
        ],
    );
    assert_contains_all(
        "projects/device_install.rs",
        &device_install,
        &[
            "pub receipt_path: PathBuf",
            "pub total_bytes: u64",
            "write_install_receipt(install_dir)?",
            "content_download_manifest",
            "project/zircon-project.toml",
        ],
    );
    assert_contains_all(
        "projects/install_receipt.rs",
        &install_receipt,
        &[
            "pub struct DeviceInstallReceipt",
            "pub content_download_manifest: HubContentDownloadManifest",
            "pub struct HubContentDownloadManifest",
            "pub struct HubContentDownloadChunk",
            "allow_range_resume: true",
            "fn sha256_hex(bytes: &[u8]) -> String",
            "sha256_hex_matches_known_vectors",
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
            "(self.folder_picker)(",
            "normalize_project_root(",
            "find_recent_project_by_filesystem_key(",
            "fn record_create_project_kept_folder_failure(",
            "(self.recycle_delete)(",
            "HubTextBundle::new(self.config.settings.language)",
            "import_project_picker_title(text)",
            "fn import_project_picker_title(text: HubTextBundle) -> &'static str",
            "text.pair(\"Import Zircon Project\", \"导入 Zircon 项目\")",
        ],
    );
    assert_contains_all(
        "runtime_state/project_actions/tests.rs",
        &project_action_tests,
        &["import_project_folder_picker_title_uses_current_language"],
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
            "\"Output folder opened\"",
            "TaskStatus::error(\"Open Output failed\"",
            "record.action.id()",
            "open_output_folder_resolves_record_id_before_path_fallback",
            "open_output_folder_prefers_typed_output_dir_over_archived_path_payload",
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
            "pub(super) fn update_settings_draft_from_action(",
            "pub(super) fn save_settings_from_action(",
            "pub(super) fn discard_settings_draft(",
            "pub(super) fn restore_default_settings(",
            "BrowseSettingsFolderPayload",
            "settings_payload: Option<HubSettingsPayload>",
            "self.settings_draft = draft;",
            "update_settings_draft_recomputes_health_without_persisting",
            "self.save_settings(settings_payload)",
            "record_settings_save_failure",
            "text.status_label(\"Save Settings failed\")",
            "HubMessage::new(HubMessageId::Settings(",
            "SettingsMessageId::CheckValuesAndSave",
            "FolderPickerRequest::new(",
            "field.picker_title(text)",
            "field.set_path(&mut draft",
            "HubTextBundle::new(self.settings_draft.language)",
            "text.pair(\"Choose Default Project Directory\", \"选择默认项目目录\")",
            "text.status_label(\"Folder selected\")",
            "text.status_label(\"Folder selection cancelled\")",
            "text.status_label(\"Browse folder failed\")",
            "SettingsMessageId::ChooseExistingFolderOrManual",
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
            "pub(crate) struct HubSettingsActionPayload",
            "pub(crate) settings: HubSettingsPayload",
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
            "let emit_state = |view_model: &HubViewModel|",
            "run_background_worker_loop(request, &session_handle, &emit_state);",
            "app.emit(\"hub-state-changed\", view_model)",
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
            "let detail = text.render_message(&record.detail);",
            "let log_excerpt = text.render_message(&record.log_excerpt);",
            "let detail_rows = action_history_detail_rows(",
            ".map(|recovery| text.render_message(recovery))",
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
            "self.persist(Some(&project.path))",
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
            "filtered_recent_projects_with_availability",
            "self.project_filter.includes(project, availability)",
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
            "DeliveryMessageId::OpenFolderManuallyRecovery",
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
            "pub task_id: u64",
            "pub fn with_task_id(",
            "pub fn running_operation(",
            "pub fn success(",
            "pub fn warning(",
            "pub fn error(",
            "pub fn with_operation(",
            "pub fn operation_summary(&self) -> String",
            "pub detail: HubMessage",
            "pub recovery: Option<HubMessage>",
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
    let detail_sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const handleAction: HubActionHandler = async (actionId, targetId, payload) =>",
            "const stateGenerationRef = useRef(0);",
            "const actionSequenceRef = useRef(0);",
            "function applyHubState(nextState: HubShellState) {",
            "stateGenerationRef.current += 1;",
            "const actionSequence = actionSequenceRef.current + 1;",
            "actionSequenceRef.current = actionSequence;",
            "const stateGenerationAtDispatch = stateGenerationRef.current;",
            "const nextState = await dispatchHubAction(actionId, targetId, payload);",
            "if (actionSequence === actionSequenceRef.current && stateGenerationRef.current === stateGenerationAtDispatch) {",
            "applyHubState(nextState);",
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
            "await invoke<unknown>(\"hub_action\", {",
            "request: { actionId, targetId, payload },",
            "return assertHubShellState(await invoke<unknown>(\"hub_state\"));",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "openOutputFolder: \"open-output-folder\"",
            "updateNewProjectDraft: \"update-new-project-draft\"",
            "updateSettingsDraft: \"update-settings-draft\"",
            "discardSettingsDraft: \"discard-settings-draft\"",
            "restoreDefaultSettings: \"restore-default-settings\"",
            "browseSettingsFolder: \"browse-settings-folder\"",
            "export type HubActionHistoryKind =",
            "kind: HubActionHistoryKind;",
            "export type HubSettingsFolderField =",
            "export interface SearchProjectsPayload",
            "export interface NewProjectDraftPayload",
            "export interface ProjectTargetPayload",
            "projectId?: string;",
            "projectPath?: string;",
            "export interface BrowseSettingsFolderPayload",
            "settings?: Partial<HubSettingsSummary>;",
            "export interface UpdateSettingsDraftPayload",
            "[HUB_ACTION.updateSettingsDraft]: UpdateSettingsDraftPayload;",
            "export interface OpenOutputFolderPayload {\n  outputDir?: string;\n  historyId?: string;\n}",
            "historyId?: string;",
            "[HUB_ACTION.searchProjects]: SearchProjectsPayload;",
            "[HUB_ACTION.updateNewProjectDraft]: NewProjectDraftPayload;",
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
            "settingsDraft: HubSettingsSummary | null;",
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
            "<ProjectDetailSidebar",
        ],
    );
    assert_contains_all(
        "ProjectDetailSidebar.tsx",
        &detail_sidebar,
        &[
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
            "void onAction(HUB_ACTION.updateSettingsDraft, undefined, { settings: nextDraft });",
            "void onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft });",
            "<SettingsSection",
        ],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
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
            "parse_payload::<SearchProjectsPayload>",
            "parses_search_projects_typed_payload",
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
            "update-settings-draft",
            "actionSequenceRef",
            "stateGenerationRef",
            "browse-settings-folder",
            "settingsDraft",
            "recomputes Configuration Health",
            "single action dispatcher and refreshed HubViewModel",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`project_workflow_contract.rs`",
            "React/MUI project workflow routing",
            "`update-settings-draft` health refresh",
            "`actionSequenceRef` and `stateGenerationRef`",
            "sends typed `{ settings: draft }` payloads on field edits",
            "browse-settings-folder",
            "settingsDraft",
            "single action dispatcher and refreshed HubViewModel",
        ],
    );
}

#[test]
fn hub_build_target_routes_through_tauri_nsis_bundler() {
    let build_entry = read_repo_file("tools/zircon_build.py");
    let build_tool = read_repo_file("tools/zircon_build_hub.py");
    let tauri_config = read_crate_file("tauri.conf.json");
    let actionable_doc = read_repo_file("docs/zircon_hub/pages/actionable-pages.md");

    assert_contains_all(
        "tools/zircon_build.py",
        &build_entry,
        &[
            "from .zircon_build_hub import build_hub",
            "build_hub(config)",
        ],
    );
    assert_contains_all(
        "tools/zircon_build_hub.py",
        &build_tool,
        &[
            "HUB_TAURI_BUNDLE_TARGET = \"nsis\"",
            "HUB_INSTALLERS_DIR_NAME = \"installers\"",
            "run_tauri_build(config, target_dir)",
            "stage_hub_tauri_outputs(config, target_dir)",
            "def run_tauri_build(config: object, target_dir: Path) -> None:",
            "str(tauri_cli_path(config))",
            "\"build\",",
            "\"--runner\",",
            "config.cargo,",
            "\"--bundles\",",
            "HUB_TAURI_BUNDLE_TARGET,",
            "\"--ci\",",
            "\"--no-sign\",",
            "command.append(\"--debug\")",
            "runner_args.append(\"--locked\")",
            "runner_args.extend([\"--jobs\", config.jobs])",
            "env[\"CARGO_TARGET_DIR\"] = str(target_dir)",
            "subprocess.run(command, cwd=config.repo_root / \"zircon_hub\", check=True, env=env)",
            "bundle_root = target_dir / config.profile_dir / \"bundle\" / HUB_TAURI_BUNDLE_TARGET",
            "installers_dir = config.engine_root / HUB_INSTALLERS_DIR_NAME",
        ],
    );
    assert_contains_all(
        "tauri.conf.json",
        &tauri_config,
        &[
            "\"beforeBuildCommand\": \"npm run build\"",
            "\"frontendDist\": \"web/dist\"",
            "\"active\": true",
            "\"targets\": [",
            "\"nsis\"",
        ],
    );
    assert_contains_all(
        "actionable-pages.md",
        &actionable_doc,
        &[
            "tools/zircon_build.py --targets hub",
            "tauri build --bundles nsis --ci --no-sign",
            "frontendDist",
            "web/dist",
            "ZirconEngine/installers",
            "Tauri has no ASAR-style archive layer",
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
