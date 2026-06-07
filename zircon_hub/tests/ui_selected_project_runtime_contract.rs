//! Static contracts for React/MUI selected-project runtime state.

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
            "{source_name} should contain selected-project runtime snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete selected-project runtime snippet {snippet:?}"
        );
    }
}

#[test]
fn scope_model_remains_canonical_project_and_source_engine_resolver() {
    let scope = read_crate_file("src/state/scope.rs");
    let snapshot = read_crate_file("src/state/hub_snapshot.rs");

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
            "Active(SourceEngineScopeEngine)",
            "pub enum ProjectEngineScopeState",
            "pub fn resolve(",
            "Source Engine scope is intentionally derived after project scope",
            "pub fn selected_project(&self) -> Option<&ProjectScopeProject>",
            "pub fn selected_or_latest_project(&self) -> Option<&ProjectScopeProject>",
            "pub fn has_stale_selected_project(&self) -> bool",
            "pub fn can_build(&self) -> bool",
            "pub fn engine_id(&self) -> Option<&str>",
            "stale_selected_project_does_not_fallback_to_latest_recent",
            "selected_project_without_engine_binding_reports_project_unbound",
            "selected_project_with_missing_engine_reports_unavailable_binding",
        ],
    );
    assert_contains_all(
        "hub_snapshot.rs",
        &snapshot,
        &[
            "pub fn scope(&self) -> HubScope",
            "HubScope::resolve(",
            "self.selected_project_path.as_deref()",
            "&self.recent_projects",
            "&self.project_metadata",
            "&self.engines",
            "self.active_engine_id.as_deref()",
            "snapshot_scope_exposes_selected_project_without_latest_recent_fallback",
        ],
    );
}

#[test]
fn tauri_runtime_persists_selected_project_and_refreshes_context() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "selected_project_path: Option<PathBuf>",
            "let selected_project_path = startup_selected_project_path(",
            "runtime_state.selected_project_path.as_deref()",
            "last_project_path.as_deref()",
            "if let Some(path) = session.selected_project_path.clone()",
            "session.activate_project_engine_for_path(&path);",
            "HubAction::SelectProject { target_id } => self.select_project_target(&target_id)?",
            "HubAction::OpenProjectDetail { target_id } => self.open_project_detail(&target_id)?",
            "fn select_project_target(&mut self, target: &str) -> Result<(), HubError>",
            "self.selected_project_path = Some(project.path.clone());",
            "self.activate_project_engine_for_path(&project.path);",
            "self.refresh_project_context_views(",
            "self.persist_with_last_project(Some(&project.path))",
            "fn open_project_detail(&mut self, target: &str) -> Result<(), HubError>",
            "self.project_subpage = ProjectSubpage::ProjectDetail;",
            "self.project_view_mode = ProjectViewMode::List;",
            "fn runtime_state_for_config(&self) -> HubRuntimeState",
            "selected_project_path: self.selected_project_path.clone(),",
            "fn selected_recent_project(&mut self) -> Option<RecentProject>",
            "self.selected_project_path = None;",
            "fn startup_selected_project_path(",
            "persisted_selected_project_path: Option<&Path>",
            "last_project_path: Option<&Path>",
            "unwrap_or_else(|| path.to_path_buf())",
            "startup_selection_preserves_persisted_stale_project_path",
            "load_from_paths_merges_repairs_registers_source_and_persists_runtime_state",
        ],
    );
}

#[test]
fn tauri_view_model_projects_selected_state_into_react_dtos() {
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub selected_project_id: Option<String>",
            "pub selected_project: Option<HubProjectDetail>",
            "pub(crate) struct HubProjectDetail",
            "pub engine_id: Option<String>",
            "pub template_id: Option<String>",
            "pub template_label: String",
            "pub exists: bool",
            "pub status: String",
            "let selected_project_id = snapshot",
            ".selected_project_path",
            "selected_project_id,",
            "selected_project: selected_project_detail(snapshot),",
            "fn selected_project_detail(snapshot: &HubSnapshot) -> Option<HubProjectDetail>",
            "Some(stale_project_detail(snapshot, selected_path))",
            "fn project_detail_from_recent(snapshot: &HubSnapshot, project: &RecentProject) -> HubProjectDetail",
            "fn stale_project_detail(snapshot: &HubSnapshot, path: &Path) -> HubProjectDetail",
            "fn project_detail_from_parts(",
            "let exists = path.exists();",
            "engine_id: metadata.and_then(|metadata| metadata.engine_id.clone())",
            "template_id: metadata.and_then(|metadata| metadata.last_selected_template.clone())",
            "template_label: project_template_label(",
            "text.pair(\"Available\", \"可用\")",
            "text.pair(\"Missing\", \"缺失\")",
            "fn source_engine_rows(snapshot: &HubSnapshot) -> Vec<HubSourceEngineSummary>",
            "let active = Some(engine.id.as_str()) == snapshot.active_engine_id.as_deref()",
            "view_model_projects_come_from_snapshot_filtering_and_state_ids",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "selectedProjectId?: string | null;",
            "selectedProject?: HubProjectDetail | null;",
            "export interface HubProjectDetail",
            "engineId?: string | null;",
            "templateId?: string | null;",
            "templateLabel: string;",
            "exists: boolean;",
            "status: string;",
            "activeSourceEngineId?: string | null;",
        ],
    );
}

#[test]
fn react_pages_consume_selected_project_state_passively() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "if (state.projectSubpage === \"project-browser\")",
            "if (state.projectSubpage === \"project-detail\")",
            "selectedProjectId={state.selectedProjectId}",
            "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
            "onOpenDetail={(project) => void onAction(HUB_ACTION.openProjectDetail, project.id)}",
            "selected={project.id === state.selectedProjectId}",
            "open={state.projectSubpage === \"new-project\"}",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "selectedProjectId={state.selectedProjectId}",
            "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
            "void onAction(HUB_ACTION.openProjectDetail, project.id);",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const project = state.selectedProject ?? null;",
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "const statusTone: StatusTone = project?.exists ? \"success\" : \"warning\";",
            "state.sourceEngines.find((engine) => engine.id === project.engineId)",
            "EmptyStateBlock title={text.noProjectSelected}",
            "HubList items={detailRows}",
            "HubTreeView nodes={projectTree} defaultExpanded={[\"project-root\"]}",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, projectTarget)}",
        ],
    );
    assert_contains_all(
        "EditorPage.tsx",
        &editor,
        &[
            "const project = state.selectedProject;",
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.openEditor, undefined, projectTarget)}",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "HubSwitch checked={Boolean(project?.exists)} label={text.projectAvailable}",
            "title: common.template, detail: project.templateLabel",
        ],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const project = state.selectedProject;",
            "const projectTarget = projectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.buildProject, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, projectTarget)}",
            "void onAction(actionId, undefined, projectTarget);",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const project = state.selectedProject;",
            "const projectTarget = projectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, projectTarget)}",
            "HubSwitch checked={Boolean(project?.exists)} label={state.ui.editor.projectAvailable}",
        ],
    );
}

#[test]
fn frontend_dispatch_uses_tauri_state_and_keeps_current_state_on_action_error() {
    let hub_api = read_crate_file("web/src/tauri/hubApi.ts");
    let app = read_crate_file("web/src/App.tsx");

    assert_contains_all(
        "hubApi.ts",
        &hub_api,
        &[
            "return await invoke<HubShellState>(\"hub_state\");",
            "export async function dispatchHubAction<TActionId extends HubActionId>(",
            "return await invoke<HubShellState>(\"hub_action\", {",
            "request: { actionId, targetId, payload },",
        ],
    );
    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const [state, setState] = useState<HubShellState>(fallbackShellState);",
            "loadHubState().then((nextState) => {",
            "const nextState = await dispatchHubAction(actionId, targetId, payload);",
            "setState(nextState);",
            "const shellText = stateRef.current.ui.shell;",
            "setState((current) => ({",
            "...current,",
            "label: shellText.actionFailed",
            "operation: shellText.actionFailed",
            "<HubWindow state={state} onAction={handleAction} />",
            "<HubSnackbar task={state.taskSummary}",
        ],
    );
}

#[test]
fn selected_project_runtime_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_selected_project_runtime_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_runtime_contract",
            "## Selected Project Runtime Contract Cutover",
            "React/MUI selected-project runtime scope",
            "src/state/scope.rs",
            "src/state/hub_snapshot.rs",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/view_model.rs",
            "web/src/types/hub.ts",
            "web/src/pages/ProjectDetailPage.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_selected_project_runtime_contract.rs`",
            "React/MUI selected-project runtime scope",
            "canonical HubScope resolver, Tauri runtime selected-project persistence, React selectedProject DTOs, and passive page consumption",
        ],
    );
}

#[test]
fn selected_project_runtime_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_selected_project_runtime_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_selected_project_runtime_contract.rs",
        &contract,
        &[
            "src/state/scope.rs",
            "src/state/hub_snapshot.rs",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/view_model.rs",
            "web/src/types/hub.ts",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/App.tsx",
            "web/src/tauri/hubApi.ts",
        ],
    );
    assert_not_contains_any(
        "ui_selected_project_runtime_contract.rs",
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
