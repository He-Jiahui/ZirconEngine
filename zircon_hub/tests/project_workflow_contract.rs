//! Static contracts for cross-cutting Hub project workflow routing.
//!
//! Focused path-scope, Source Engine, page-copy, and quick-action assertions
//! live in companion integration tests. This file keeps the workflow seams that
//! span those companions from regressing back to broad fallback behavior.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .to_string_lossy()
            .into_owned()
    }))
}

fn read_crate_file(path: &str) -> String {
    fs::read_to_string(crate_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(crate_dir().join("..").join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn builds_page_selected_project_actions_do_not_use_latest_recent_fallback() {
    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");

    let selected_package = workspace
        .split("fn package_selected_project_to_output_with_messages")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn package_project_to_output").next())
        .expect("project_workspace.rs must keep selected-project package helper before package_project_to_output");
    for snippet in [
        "selected_project_for_named_action(missing_project_message, stale_project_message)",
        "Select an available project before packaging",
        "self.package_project_to_output(project)",
    ] {
        assert!(
            selected_package.contains(snippet),
            "Selected-project package path must resolve only the selected project and keep action diagnostics; missing {snippet}"
        );
    }
    assert!(
        !selected_package.contains("selected_or_latest_recent_project"),
        "Builds/Cloud selected-project package actions must not promote the newest recent project"
    );

    let selected_open = workspace
        .split("pub(super) fn open_selected_project_in_editor")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn open_selected_project_or_editor")
                .next()
        })
        .expect("project_workspace.rs must keep selected-only open before fallback open helper");
    for snippet in [
        "selected_project_for_named_action(",
        "Select a project before opening",
        "Selected project is no longer available to open",
        "self.open_project_path(ui, project.path, Some(display_name))",
    ] {
        assert!(
            selected_open.contains(snippet),
            "Builds selected-project open action must keep selected-only resolution; missing {snippet}"
        );
    }
    assert!(
        !selected_open.contains("selected_or_latest_recent_project"),
        "Builds selected-project open must not share Dashboard's no-selection latest-recent fallback"
    );

    let fallback_open = workspace
        .split("pub(super) fn open_selected_project_or_editor")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn create_project").next())
        .expect("project_workspace.rs must keep dashboard fallback open before create_project");
    for snippet in [
        "selected_or_latest_recent_project_for_action()",
        "return self.launch_editor_without_project(ui);",
    ] {
        assert!(
            fallback_open.contains(snippet),
            "Dashboard open-editor fallback remains explicit and separate from Builds selected-project actions; missing {snippet}"
        );
    }
}

#[test]
fn project_selection_and_action_resolution_refresh_scoped_views_before_snapshot() {
    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");

    let remember_project = workspace
        .split("pub(super) fn remember_project(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn refresh_selected_project_scoped_views")
                .next()
        })
        .expect("project_workspace.rs must declare remember_project before refresh helpers");
    for snippet in [
        "let active_engine_before = self.config.active_engine_id.clone();",
        "self.selected_project_path = Some(last_project_path.clone());",
        "self.activate_project_engine_for_path(&last_project_path);",
        "self.refresh_project_context_views(\n            true,\n            self.config.active_engine_id != active_engine_before,\n        )?;",
        "self.persist_with_last_project(Some(&last_project_path))",
    ] {
        assert!(
            remember_project.contains(snippet),
            "Remembering a project must select it, activate its bound engine, refresh scoped views, then persist last_project_path; missing {snippet}"
        );
    }

    let selected_action = workspace
        .split("pub(super) fn selected_project_for_named_action")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn selected_project_with_engine_for_named_action").next())
        .expect("project_workspace.rs must keep selected_project_for_named_action before engine-required helper");
    for snippet in [
        "let selected_before = self.selected_project_path.clone();",
        "let active_engine_before = self.config.active_engine_id.clone();",
        "let Some(project) = self.selected_recent_project() else",
        "self.refresh_project_context_views(selected_project_changed, false)?;",
        "self.activate_project_engine_for_path(&project.path);",
        "selected_project_path_changed(\n            selected_before.as_deref(),\n            self.selected_project_path.as_deref(),\n        )",
        "self.refresh_project_context_views(\n            selected_project_changed,\n            self.config.active_engine_id != active_engine_before,\n        )?;",
    ] {
        assert!(
            selected_action.contains(snippet),
            "Selected-project action resolution must refresh stale/changed project context before the next snapshot; missing {snippet}"
        );
    }
}

#[test]
fn command_action_records_use_single_persistence_helper() {
    let runtime = read_crate_file("src/app/runtime.rs");
    let complete_build = runtime
        .split("fn complete_editor_runtime_build")
        .nth(1)
        .and_then(|source| source.split("fn ensure_editor_available").next())
        .expect("runtime.rs must keep build completion before editor availability checks");

    assert_eq!(
        complete_build
            .matches("self.record_action_and_persist(HubActionRecord")
            .count(),
        3,
        "build success, build command failure, and non-zero build results must all save action history through record_action_and_persist"
    );
    assert!(
        !complete_build.contains("self.record_action(HubActionRecord"),
        "build command handlers must not bypass the persistence helper when creating HubActionRecord rows"
    );
    assert!(
        !complete_build.contains("self.persist_hub_config()?;")
            && !complete_build.contains("self.persist()?;"),
        "build command handlers should not split action recording from action-history persistence"
    );

    for snippet in [
        "command_line.clone()",
        "log_excerpt: report.log_excerpt()",
        "output_dir: Some(output_dir)",
        "TaskStatus::error(",
        "TaskStatus::success(",
    ] {
        assert!(
            complete_build.contains(snippet),
            "Build command outcomes must preserve command metadata, visible status, and output paths; missing {snippet}"
        );
    }
}

#[test]
fn runtime_state_persists_through_hub_config_not_editor_recent_state() {
    let config = read_crate_file("src/settings/hub_config.rs");
    for snippet in [
        "pub runtime: HubRuntimeState,",
        "pub struct HubRuntimeState",
        "pub selected_page: HubPage,",
        "pub project_subpage: ProjectSubpage,",
        "pub selected_project_path: Option<PathBuf>,",
        "pub new_project_engine_id: Option<String>,",
        "pub fn normalize(&mut self)",
    ] {
        assert!(
            config.contains(snippet),
            "HubConfig must own restart-visible Hub runtime state; missing {snippet}"
        );
    }

    let persistence = read_crate_file("src/app/runtime/persistence.rs");
    for snippet in [
        "config.runtime = self.runtime_state_for_config();",
        "fn runtime_state_for_config(&self) -> HubRuntimeState",
        "runtime_state.selected_page",
        "runtime_state.project_subpage",
        "runtime_state.selected_project_path.as_deref()",
        "unwrap_or_else(|| path.to_path_buf())",
    ] {
        assert!(
            persistence.contains(snippet),
            "Runtime persistence must save and restore Hub-owned page/selection state without editor recent fallback drift; missing {snippet}"
        );
    }

    let runtime = read_crate_file("src/app/runtime.rs");
    for snippet in [
        "match action(&mut runtime, &ui)",
        "Ok(()) => {",
        "runtime.persist_hub_config()",
        "\"State save failed\"",
    ] {
        assert!(
            runtime.contains(snippet),
            "Successful UI callbacks must persist page/subpage/selection state through HubConfig; missing {snippet}"
        );
    }
    assert!(
        !runtime.contains("set_selected_page_after_reload"),
        "Background build reload must not overwrite the latest persisted selected page with a stale pre-build page"
    );

    let editor_recent = read_crate_file("src/projects/editor_recent_sync.rs");
    assert!(
        editor_recent.contains("editor_recent_writer_does_not_emit_hub_project_metadata"),
        "Editor recent sync must remain an import/export bridge and not become the Hub runtime state owner"
    );
}

#[test]
fn create_project_binds_requested_engine_before_remembering_project() {
    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    let create_project = workspace
        .split("pub(super) fn create_project")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn select_new_project_engine_by_id").next())
        .expect("project_workspace.rs must declare create_project before select_new_project_engine_by_id");

    for snippet in [
        "request\n            .validate_launch_fields()\n            .map_err(HubError::message)?;",
        "let root = request.target_root();",
        "let engine_id = self\n            .new_project_engine_id\n            .clone()",
        "self.require_engine(&engine_id)?;",
        "self.config.active_engine_id = Some(engine_id.clone());",
        "self.sync_settings_from_active_engine();",
        "self.remember_project_metadata_for_path(\n            &root,\n            Some(engine_id),\n            Some(self.selected_template_id.clone()),\n        );",
        "self.remember_project(RecentProject::with_now(display_name.clone(), root))?;",
    ] {
        assert!(
            create_project.contains(snippet),
            "Create Project must bind the requested Source Engine/template metadata before remember_project refreshes selected-project context; missing {snippet}"
        );
    }
    let metadata_index = create_project
        .find("self.remember_project_metadata_for_path")
        .expect("create_project must remember Hub project metadata");
    let remember_index = create_project
        .find("self.remember_project(RecentProject::with_now")
        .expect("create_project must remember the created project");
    assert!(
        metadata_index < remember_index,
        "Create Project metadata binding must happen before remember_project so selected-project refresh sees the bound engine"
    );

    let create_request = read_crate_file("src/projects/create_project_request.rs");
    for snippet in [
        "pub fn validate_launch_fields(&self) -> Result<(), &'static str>",
        "Project name is required",
        "Project location is required",
        "pub fn target_root(&self) -> PathBuf",
    ] {
        assert!(
            create_request.contains(snippet),
            "Create Project request validation should own reusable UI/runtime preflight helpers; missing {snippet}"
        );
    }
}

#[test]
fn project_lifecycle_docs_record_split_plans_and_validation_gate() {
    let lifecycle =
        fs::read_to_string(crate_dir().join("../docs/zircon_hub/projects/lifecycle-workflows.md"))
            .expect("docs/zircon_hub/projects/lifecycle-workflows.md should be readable");

    for snippet in [
        "related_code:",
        "implementation_files:",
        "plan_sources:",
        "tests:",
        ".opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-actions-model/plan.md",
        ".opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-create-import-templates/plan.md",
        ".opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-dashboard-detail-ui/plan.md",
        ".opencode/workflows/20260528_190023_866_继续完善hub/hub-projects-editor-launch-contracts/plan.md",
        "## Canonical project action model",
        "## Create, import, and template flow",
        "## Slint projections and page behavior",
        "## Editor launch contracts",
        "cargo test -p zircon_hub --test project_workflow_contract --locked -- --nocapture",
        "cargo build -p zircon_hub --bin zircon_hub --locked",
    ] {
        assert!(
            lifecycle.contains(snippet),
            "Project lifecycle docs should record split plans, owners, and validation gate; missing {snippet}"
        );
    }
}

#[test]
fn runtime_state_docs_record_split_child_evidence_and_dto_flow() {
    let foundations = read_repo_file("docs/zircon_hub/state/foundations.md");
    for snippet in [
        "## Canonical scope",
        "## Command action runtime scope",
        "## Scoped catalog snapshots",
        "## Restart persistence",
        "### 2026-05-30 contract-docs evidence",
        "hub-runtime-state-integration-scope-model/review-surface.md",
        "hub-runtime-state-integration-command-actions/review-surface.md",
        "hub-runtime-state-integration-scoped-snapshots/review-surface.md",
        "hub-runtime-state-integration-restart-persistence/review-surface.md",
        "hub-runtime-state-integration-contract-docs/review-surface.md",
        "HubConfig.runtime",
        "record_action_and_persist()",
        "source_scoped_views.rs",
    ] {
        assert!(
            foundations.contains(snippet),
            "Runtime-state foundations docs must record split child evidence and review boundaries; missing {snippet}"
        );
    }

    let responsive = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");
    for snippet in [
        "## Runtime-State Data Flow",
        "HubRuntime -> HubSnapshot -> HubScope -> view-model builders -> binding::apply_snapshot() -> Slint DTO props",
        "Slint pages consume projected DTOs",
        "has-selected-project",
        "OperationTimelinePanel",
        "record_action_and_persist()",
        "HubConfig.runtime",
        "Dashboard Quick Actions are the only visible surface that may use latest-recent fallback",
        "project_path_scope_contract.rs",
    ] {
        assert!(
            responsive.contains(snippet),
            "Responsive component docs must describe the runtime-state DTO flow instead of allowing page-local scope recomputation; missing {snippet}"
        );
    }

    let index = read_repo_file("docs/zircon_hub/index.md");
    for snippet in [
        "## Runtime-State Integration Split",
        "hub-runtime-state-integration-scope-model",
        "hub-runtime-state-integration-command-actions",
        "hub-runtime-state-integration-scoped-snapshots",
        "hub-runtime-state-integration-restart-persistence",
        "hub-runtime-state-integration-contract-docs",
        "hub-docs-contract-refresh",
        "hub-acceptance-validation",
        "visual-design artifact validation separate from the runtime-state scope/command/catalog/persistence contracts",
    ] {
        assert!(
            index.contains(snippet),
            "Hub index docs must link the runtime-state split and follow-on quality milestones; missing {snippet}"
        );
    }
}
