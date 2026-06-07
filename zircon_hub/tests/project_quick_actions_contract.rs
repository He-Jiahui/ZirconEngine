//! Static contracts for React/MUI scope-derived Hub quick actions.

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
            "{source_name} should contain quick-action snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete quick-action snippet {snippet:?}"
        );
    }
}

#[test]
fn quick_action_dtos_are_scope_derived_in_tauri_view_model() {
    let view_model = read_crate_file("src/tauri_app/view_model.rs");

    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub(crate) struct HubQuickAction",
            "pub enabled: bool",
            "quick_actions: quick_actions(snapshot)",
            "fn quick_actions(snapshot: &HubSnapshot) -> Vec<HubQuickAction>",
            "let project_target = quick_action_project_target(snapshot);",
            "enum QuickActionKind",
            "enum QuickActionProjectTarget",
            "enum QuickActionSourceEngineState",
            "fn quick_action_project_target(snapshot: &HubSnapshot) -> QuickActionProjectTarget",
            "match snapshot.scope().project",
            "ProjectScope::Selected(project)",
            "ProjectScope::LatestRecent(project)",
            "ProjectScope::StaleSelection { .. }",
            "ProjectScope::None",
            "fn quick_action_enabled(",
            "QuickActionKind::BuildProject => target.has_source_engine()",
            "QuickActionKind::PackageProject | QuickActionKind::InstallToDevice => target.has_project()",
            "QuickActionKind::OpenEditor => true",
            "Build selected project {name}",
            "Build latest recent project {name}",
            "Bind a Source Engine to {name} before building",
            "Bound Source Engine for {name} is unavailable",
            "Selected project is no longer available",
            "Open Editor without a project",
            "fn quick_actions_use_selected_project_scope_and_engine_binding()",
            "fn quick_actions_disable_unbound_or_stale_project_targets()",
            "fn quick_actions_use_latest_recent_only_when_no_project_is_selected()",
        ],
    );
}

#[test]
fn runtime_quick_actions_keep_fallback_and_persisted_history_separate_from_dto_copy() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let quick_actions = read_crate_file("src/tauri_app/runtime_state/quick_actions.rs");
    let action_tasks = read_crate_file("src/tauri_app/runtime_state/action_tasks.rs");
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");
    let commands = read_crate_file("src/tauri_app/commands.rs");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
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
        ],
    );
    assert_contains_all(
        "runtime_state/quick_actions.rs",
        &quick_actions,
        &[
            "pub(super) fn record_action_and_persist(",
            "self.config.action_history.insert(0, record);",
            "self.persist_hub_config()",
            "package_action_creates_project_package_and_records_success_history",
            "install_action_packages_project_then_copies_package_to_device_root",
            "open_editor_action_records_recoverable_failure_without_falling_back_to_demo_state",
        ],
    );
    assert_contains_all(
        "runtime_state/editor_launch_actions.rs",
        &editor_launch_actions,
        &[
            "pub(in crate::tauri_app) struct PendingEditorLaunch",
            "pub(in crate::tauri_app) fn run(&self) -> Result<EditorLaunchReport, HubError>",
            "pub(super) fn open_selected_project_or_editor(&mut self) -> Result<(), HubError>",
            "pub(in crate::tauri_app) fn prepare_background_editor_launch",
            "pub(in crate::tauri_app) fn complete_background_editor_launch",
            "selected_or_latest_recent_project_for_action()",
            "prepare_empty_editor_launch()",
            "launch_editor(command)?",
            "Command::new(executable).spawn()?",
            "record_editor_launch_failure(",
            "background_editor_launch_prepare_records_missing_executable_failure_without_spawn",
            "background_editor_launch_completion_records_success_after_external_spawn",
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
            "selected_or_latest_recent_project_with_engine_for_action()",
            "record_build_action_failure(",
            "Select a valid project with a bound Source Engine before building",
            "validate_active_source_engine_for_build",
            "record_active_build(",
            "background_build_prepares_command_without_running_or_recording_history",
            "background_build_completion_records_success_after_external_result",
        ],
    );
    assert_contains_all(
        "runtime_state/project_delivery_actions.rs",
        &project_delivery_actions,
        &[
            "pub(in crate::tauri_app) struct PendingProjectPackage",
            "pub(in crate::tauri_app) struct PendingDeviceInstall",
            "pub(super) fn package_recent_project(&mut self) -> Result<(), HubError>",
            "pub(super) fn install_recent_project_to_device(&mut self) -> Result<(), HubError>",
            "pub(in crate::tauri_app) fn prepare_background_project_package",
            "pub(in crate::tauri_app) fn complete_background_project_package",
            "pub(in crate::tauri_app) fn prepare_background_device_install",
            "pub(in crate::tauri_app) fn complete_background_device_install",
            "package_project(&self.request)",
            "install_package_to_device(&install_request)",
            "ProjectPackageRequest::new(",
            "DeviceInstallRequest::new(",
            "record_package_success(",
            "No recent project is available to package",
            "Selected project is no longer available to package",
            "Select a valid project and package it before installing to a device",
            "background_package_prepares_request_without_copying_or_recording_history",
            "background_package_completion_records_success_after_copy_result",
            "background_install_runs_package_then_device_copy_before_recording_history",
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
            "background_action_status_marks_build_running_without_executing_it",
            "background_actions_queue_while_worker_is_active_and_dequeue_fifo",
        ],
    );
    assert_contains_all(
        "commands.rs",
        &commands,
        &[
            "pub(super) fn hub_action(",
            "if HubRuntimeSession::should_run_action_in_background(&request)",
            "session.start_background_action_or_record_error(&request)?;",
            "spawn_background_action(request, session_handle, app.clone());",
            "fn spawn_background_action(",
            "thread::spawn(move ||",
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
            "Ok(view_model)",
        ],
    );
}

#[test]
fn react_quick_actions_consume_enabled_dtos_and_guard_disabled_clicks() {
    let types = read_crate_file("web/src/types/hub.ts");
    let fallback_data = read_crate_file("web/src/data/hubData.ts");
    let quick_actions = read_crate_file("web/src/components/data/QuickActions.tsx");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let team = read_crate_file("web/src/pages/TeamPage.tsx");
    let workspace = read_crate_file("web/src/pages/WorkspacePage.tsx");

    assert_contains_all("types/hub.ts", &types, &["enabled: boolean;"]);
    assert_contains_all(
        "hubData.ts",
        &fallback_data,
        &["quickActions: [", "enabled: true"],
    );
    assert_contains_all(
        "QuickActions.tsx",
        &quick_actions,
        &[
            "actions: HubQuickAction[];",
            "disabled={!action.enabled}",
            "if (action.enabled) {",
            "onAction?.(action);",
            "\"&.Mui-disabled\"",
        ],
    );
    let project_target = read_crate_file("web/src/tauri/projectTarget.ts");
    assert_contains_all(
        "projectTarget.ts",
        &project_target,
        &[
            "export function projectTargetPayload(project?: HubProjectDetail | null): ProjectTargetPayload | undefined",
            "export function workflowProjectTargetPayload(state: HubShellState): ProjectTargetPayload | undefined",
            "const target = workflowTargetProject(state);",
            "projectPath: workflowProjectPath(target),",
            "export function workflowTargetProject(state: HubShellState): HubProjectDetail | HubRecentProject | undefined",
            "export function workflowProjectPath(target: HubProjectDetail | HubRecentProject): string",
            "state.selectedProject ?? state.recentProjects[0]",
            "export function quickActionProjectTargetPayload(project?: HubProjectDetail | null): ProjectTargetPayload | undefined",
            "if (!project?.exists) {",
            "return projectTargetPayload(project);",
        ],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &[
            "const project = state.selectedProject;",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "EditorPage.tsx",
        &editor,
        &[
            "const project = state.selectedProject;",
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "TeamPage.tsx",
        &team,
        &[
            "const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    assert_contains_all(
        "WorkspacePage.tsx",
        &workspace,
        &[
            "const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />",
        ],
    );
}

#[test]
fn quick_action_documentation_records_tauri_react_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/project_quick_actions_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test project_quick_actions_contract",
            "## Project Quick Actions Contract Cutover",
            "React/MUI scope-derived quick-action DTOs",
            "src/tauri_app/view_model.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/build_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "web/src/components/data/QuickActions.tsx",
            "workflowProjectTargetPayload",
            "selected project first and the latest recent project second",
            "enabled quick-action DTO state",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`project_quick_actions_contract.rs`",
            "React/MUI scope-derived quick-action DTOs",
            "workflowProjectTargetPayload",
            "selected project first and the latest recent project second",
            "enabled quick-action DTO state",
        ],
    );
}

#[test]
fn quick_action_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/project_quick_actions_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "project_quick_actions_contract.rs",
        &contract,
        &[
            "src/tauri_app/view_model.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/build_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "web/src/components/data/QuickActions.tsx",
            "web/src/tauri/projectTarget.ts",
            "web/src/types/hub.ts",
        ],
    );
    assert_not_contains_any(
        "project_quick_actions_contract.rs",
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
