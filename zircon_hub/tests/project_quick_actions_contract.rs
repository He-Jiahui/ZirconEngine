//! Static contracts for scope-derived Hub quick-action copy.
//!
//! Command routing and runtime error handling are covered by focused runtime
//! contracts; these assertions lock the scope-model projection and shared page
//! header command surface.

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

#[test]
fn page_header_actions_route_through_shared_runtime_dispatch() {
    let app = read_crate_file("ui/app.slint");
    let quick_action = read_crate_file("src/app/quick_action.rs");
    let runtime = read_crate_file("src/app/runtime.rs");
    for snippet in [
        "private property <string> page-header-secondary-action-id:",
        "private property <string> page-header-primary-action-id:",
        "callback page-header-action(string);",
        "page-action(id) => {",
        "root.page-header-action(id);",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow should map shared header buttons to stable action IDs; missing {snippet}"
        );
    }

    for snippet in [
        "pub(super) enum HubPageHeaderAction",
        "\"refresh-sources\" => Some(Self::RefreshSources)",
        "\"refresh-assets\" => Some(Self::RefreshAssets)",
        "\"refresh-plugins\" => Some(Self::RefreshPlugins)",
        "\"refresh-learn\" => Some(Self::RefreshLearn)",
        "\"request-review\" => Some(Self::RequestReview)",
        "\"deploy-preview\" => Some(Self::DeployPreview)",
        "\"open-source-control\" => Some(Self::OpenSourceControl)",
        "\"add-asset\" => Some(Self::AddAsset)",
        "\"add-plugin\" => Some(Self::AddPlugin)",
        "\"add-guide\" => Some(Self::AddGuide)",
        "\"save-settings\" => Some(Self::SaveSettings)",
    ] {
        assert!(
            quick_action.contains(snippet),
            "HubPageHeaderAction should parse every shared page header command ID; missing {snippet}"
        );
    }

    for snippet in [
        "fn page_header_action(&mut self, ui: &HubWindow, action_id: &str)",
        "HubPageHeaderAction::from_id(action_id)",
        "Some(HubPageHeaderAction::RefreshSources) => self.refresh_sources_from_header(ui),",
        "Some(HubPageHeaderAction::RefreshAssets) => self.refresh_assets_from_header(ui),",
        "Some(HubPageHeaderAction::RefreshPlugins) => self.refresh_plugins_from_header(ui),",
        "Some(HubPageHeaderAction::RefreshLearn) => self.refresh_learn_from_header(ui),",
        "Some(HubPageHeaderAction::RequestReview) => self.request_review_from_header(ui),",
        "Some(HubPageHeaderAction::OpenSourceControl) => {",
        "self.open_source_control_from_header(ui)",
        "Some(HubPageHeaderAction::SaveSettings) => self.save_settings(ui),",
        "fn refresh_sources_from_header(&mut self, ui: &HubWindow)",
        "fn reserved_page_action(",
        "\"Deploy Preview reserved\"",
        "\"Add Asset reserved\"",
        "\"Add Plugin reserved\"",
        "\"Add Guide reserved\"",
        "fn wire_page_header_actions(ui: &HubWindow, runtime: Rc<RefCell<HubRuntime>>)",
        "ui.on_page_header_action(move |action_id|",
        "runtime.page_header_action(ui, &action_id)",
    ] {
        assert!(
            runtime.contains(snippet),
            "Runtime should execute or visibly report every shared page header action; missing {snippet}"
        );
    }
}
