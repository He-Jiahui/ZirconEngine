//! Static contracts for scope-derived Hub quick-action copy.
//!
//! Command routing and runtime error handling are covered by later runtime-state
//! milestones; this contract only locks the scope-model projection.

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

#[test]
fn quick_actions_copy_comes_from_hub_scope() {
    let quick_actions = read_crate_file("src/app/view_model/quick_actions.rs");
    for snippet in [
        "use crate::state::{HubSnapshot, ProjectEngineScopeState, ProjectScope};",
        "let project_target = quick_action_project_target(snapshot);",
        "quick_action_detail(action, &project_target, language)",
        "quick_action_enabled(action, &project_target)",
        "match snapshot.scope().project",
        "ProjectScope::Selected(project) => QuickActionProjectTarget::Selected",
        "ProjectScope::StaleSelection { .. } => QuickActionProjectTarget::StaleSelection",
        "ProjectScope::LatestRecent(project) => QuickActionProjectTarget::LatestRecent",
        "ProjectScope::None => QuickActionProjectTarget::None",
        "source_engine_state: project_source_engine_state(project.engine_state)",
    ] {
        assert!(
            quick_actions.contains(snippet),
            "QuickActionData must derive target/copy from HubScope; missing {snippet}"
        );
    }
}

#[test]
fn quick_actions_explain_selected_latest_stale_and_empty_project_targets() {
    let quick_actions = read_crate_file("src/app/view_model/quick_actions.rs");
    for snippet in [
        "Build selected project {name}",
        "Build latest recent project {name}",
        "Bind a Source Engine to {name} before building",
        "Bind a Source Engine to latest recent project {name} before building",
        "Bound Source Engine for {name} is unavailable",
        "Bound Source Engine for latest recent project {name} is unavailable",
        "Selected project is no longer available",
        "Select a project with a bound Source Engine before building",
        "Open Editor without a project",
        "HubQuickAction::BuildProject => target.has_source_engine()",
        "HubQuickAction::PackageProject | HubQuickAction::InstallToDevice => target.has_project()",
        "HubQuickAction::OpenEditor => true",
        "fn quick_actions_do_not_fallback_when_selected_project_is_stale()",
        "fn quick_actions_describe_no_selection_and_latest_recent_scope()",
        "fn build_action_disables_unbound_selected_project()",
        "fn build_action_explains_unavailable_bound_source_engine()",
    ] {
        assert!(
            quick_actions.contains(snippet),
            "Quick action scope copy must distinguish selected/latest/stale/empty targets; missing {snippet}"
        );
    }
}

#[test]
fn hub_snapshot_scope_is_the_single_projection_source() {
    let snapshot = read_crate_file("src/state/hub_snapshot.rs");
    for snippet in [
        "pub fn scope(&self) -> HubScope",
        "HubScope::resolve(",
        "self.selected_project_path.as_deref()",
    ] {
        assert!(
            snapshot.contains(snippet),
            "HubSnapshot should expose canonical HubScope for project/source-engine projection; missing {snippet}"
        );
    }

    let scope = read_crate_file("src/state/scope.rs");
    for snippet in [
        "pub struct HubScope",
        "pub enum ProjectScope",
        "StaleSelection { requested_path: PathBuf }",
        "pub enum SourceEngineScope",
        "ProjectBound(SourceEngineScopeEngine)",
        "ProjectUnbound {",
        "ProjectEngineUnavailable {",
        "Active(SourceEngineScopeEngine)",
        "stale_selected_project_does_not_fallback_to_latest_recent",
        "selected_project_without_engine_binding_reports_project_unbound",
        "selected_project_with_missing_engine_reports_unavailable_binding",
        "active_engine_scope_falls_back_to_first_engine_then_none",
    ] {
        assert!(
            scope.contains(snippet),
            "HubScope should centralize selected project, fallback project, and Source Engine state; missing {snippet}"
        );
    }
}
