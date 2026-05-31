//! Static contracts for the scope-model prerequisite of selected-project UI state.
//!
//! This file intentionally avoids command execution, persistence, action history,
//! scoped catalog refresh, restart restore, and process-launch assertions. Those
//! belong to later runtime-state child milestones.

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
fn scope_model_is_canonical_snapshot_projection() {
    let scope = read_crate_file("src/state/scope.rs");
    for snippet in [
        "pub struct HubScope",
        "pub enum ProjectScope",
        "Selected(ProjectScopeProject)",
        "StaleSelection { requested_path: PathBuf }",
        "LatestRecent(ProjectScopeProject)",
        "pub enum SourceEngineScope",
        "ProjectBound(SourceEngineScopeEngine)",
        "ProjectUnbound {",
        "ProjectEngineUnavailable {",
        "Active(SourceEngineScopeEngine)",
        "Source Engine scope is intentionally derived after project scope",
        "stale_selected_project_does_not_fallback_to_latest_recent",
        "selected_project_without_engine_binding_reports_project_unbound",
        "selected_project_with_missing_engine_reports_unavailable_binding",
        "active_engine_scope_falls_back_to_first_engine_then_none",
    ] {
        assert!(
            scope.contains(snippet),
            "HubScope must centralize project and Source Engine scope; missing {snippet}"
        );
    }

    let snapshot = read_crate_file("src/state/hub_snapshot.rs");
    for snippet in [
        "pub fn scope(&self) -> HubScope",
        "HubScope::resolve(",
        "self.selected_project_path.as_deref()",
        "&self.recent_projects",
        "&self.project_metadata",
        "&self.engines",
        "self.active_engine_id.as_deref()",
        "snapshot_scope_exposes_selected_project_without_latest_recent_fallback",
    ] {
        assert!(
            snapshot.contains(snippet),
            "HubSnapshot::scope must be the only low-level scope resolver; missing {snippet}"
        );
    }
}

#[test]
fn scope_derived_view_model_copy_exposes_disabled_reasons() {
    let view_model = read_crate_file("src/app/view_model.rs");
    for snippet in [
        "match snapshot.scope().source_engine",
        "SourceEngineScope::ProjectBound(engine) | SourceEngineScope::Active(engine)",
        "SourceEngineScope::ProjectUnbound { .. }",
        "SourceEngineScope::ProjectEngineUnavailable { .. }",
        "SourceEngineScope::None",
        "fn build_context_engine(snapshot: &HubSnapshot) -> Option<&SourceEngineInstall>",
        "let scope = snapshot.scope();",
        "selected_project_status(&scope.project, language)",
        "source_engine_status(snapshot, &scope.source_engine)",
        "source_engine_data_prefers_selected_project_bound_engine",
        "source_engine_data_does_not_fallback_when_selected_project_is_unbound",
    ] {
        assert!(
            view_model.contains(snippet),
            "View model must copy canonical HubScope into DTOs; missing {snippet}"
        );
    }

    let projects = read_crate_file("src/app/view_model/projects.rs");
    for snippet in [
        "return empty_project_detail(snapshot.settings.language);",
        "fn empty_project_detail(language: HubLanguage) -> ProjectDetailData",
        "can_open: false,",
        "can_build: false,",
        "can_delete: false,",
        "can_build: row.can_build,",
        "fn project_can_build(",
        "can_open && project_engine(project, snapshot).is_some()",
        "project_paths_match(selected, &project.path)",
        ".find(|project| project_paths_match(&project.path, selected_path))",
    ] {
        assert!(
            projects.contains(snippet),
            "Project DTOs must distinguish selected/stale/no-project and buildable state; missing {snippet}"
        );
    }
    assert!(
        !projects.contains(".or(snapshot.active_engine_id.as_deref())"),
        "Project DTOs must not make active Source Engine look project-bound"
    );

    let workspace_actions = read_crate_file("src/app/view_model/workspace_actions.rs");
    for snippet in [
        "pub(in crate::app) fn workspace_action_readiness(",
        "let scope = snapshot.scope();",
        "ProjectScope::Selected(project)",
        "ProjectScope::StaleSelection { requested_path }",
        "ProjectScope::LatestRecent(_) | ProjectScope::None",
        "SourceEngineScope::ProjectBound(_) =>",
        "SourceEngineScope::Active(_) =>",
        "SourceEngineScope::ProjectUnbound { .. }",
        "SourceEngineScope::ProjectEngineUnavailable { .. }",
        "Select a project before building",
        "Selected project is no longer available to build",
        "Bind a Source Engine before building",
    ] {
        assert!(
            workspace_actions.contains(snippet),
            "Workspace readiness copy must come from HubScope disabled states; missing {snippet}"
        );
    }
}

#[test]
fn slint_consumes_scope_dtos_passively() {
    let shared = read_ui_file("shared.slint");
    let app = read_ui_file("app.slint");
    let builds = read_ui_file("builds.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let editor = read_ui_file("editor.slint");

    for snippet in [
        "export struct ProjectDetailData",
        "can-open: bool,",
        "can-build: bool,",
        "export struct QuickActionData",
        "detail: string,",
        "enabled: bool,",
        "export struct SourceEngineData",
        "export struct WorkspaceActionReadinessData",
        "build-disabled-reason: string,",
        "open-editor-disabled-reason: string,",
    ] {
        assert!(
            shared.contains(snippet),
            "shared.slint must expose scope-derived DTO fields; missing {snippet}"
        );
    }

    for snippet in [
        "in property <ProjectDetailData> project-detail;",
        "in property <[QuickActionData]> quick-actions;",
        "in property <SourceEngineData> source-engine;",
        "in property <WorkspaceActionReadinessData> workspace-action-readiness;",
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "readiness: root.workspace-action-readiness;",
        "has-selected-project: root.project-detail.selected;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must pass scope-derived DTOs into pages passively; missing {snippet}"
        );
    }

    for snippet in [
        "in property <WorkspaceActionReadinessData> readiness;",
        "root.readiness.build-enabled ? root.readiness.selected-project-title : root.readiness.build-disabled-reason",
        "action-enabled: root.readiness.build-enabled;",
        "root.readiness.open-editor-enabled ? root.readiness.selected-project-title : root.readiness.open-editor-disabled-reason",
    ] {
        assert!(
            builds.contains(snippet) || builds_components.contains(snippet) || editor.contains(snippet),
            "Builds/Editor surfaces must consume readiness DTO fields without recomputing scope; missing {snippet}"
        );
    }

    let binding = read_crate_file("src/app/binding.rs");
    for snippet in [
        "ui.set_project_detail(view_model::project_detail(snapshot));",
        "view_model::quick_actions(\n        snapshot, language,",
        "let source_engine = view_model::source_engine_data(snapshot);",
        "ui.set_source_engine(source_engine);",
        "ui.set_workspace_action_readiness(view_model::workspace_action_readiness(snapshot));",
    ] {
        assert!(
            binding.contains(snippet),
            "binding.rs must forward scope-derived DTOs in one snapshot pass; missing {snippet}"
        );
    }
}
