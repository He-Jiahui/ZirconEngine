//! Static contracts for React/MUI project scope projection.

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
            "{source_name} should contain project-scope snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete project-scope snippet {snippet:?}"
        );
    }
}

#[test]
fn rust_state_modules_own_project_scope_metadata_and_runtime_persistence() {
    let scope = read_crate_file("src/state/scope.rs");
    let snapshot = read_crate_file("src/state/hub_snapshot.rs");
    let state_mod = read_crate_file("src/state/mod.rs");
    let config = read_crate_file("src/settings/hub_config.rs");
    let metadata = read_crate_file("src/projects/metadata.rs");

    assert_contains_all(
        "scope.rs",
        &scope,
        &[
            "pub struct HubScope",
            "pub enum ProjectScope",
            "ProjectScopeProject",
            "engine_id: Option<String>",
            "ProjectEngineScopeState",
            "MissingBinding",
            "Unavailable",
            "pub fn selected_project(&self) -> Option<&ProjectScopeProject>",
            "pub fn selected_or_latest_project(&self) -> Option<&ProjectScopeProject>",
            "pub fn can_build(&self) -> bool",
        ],
    );
    assert_contains_all(
        "hub_snapshot.rs",
        &snapshot,
        &[
            "pub selected_project_path: Option<PathBuf>",
            "pub selected_template_id: String",
            "pub new_project_engine_id: Option<String>",
            "pub project_metadata: ProjectMetadataMap",
            "pub fn scope(&self) -> HubScope",
            "pub fn filtered_recent_projects(&self) -> Vec<RecentProject>",
            "project_matches_query(project, &query)",
        ],
    );
    assert_contains_all(
        "state/mod.rs",
        &state_mod,
        &[
            "pub use scope::{",
            "HubScope",
            "ProjectEngineScopeState",
            "ProjectScope",
            "SourceEngineScope",
        ],
    );
    assert_contains_all(
        "hub_config.rs",
        &config,
        &[
            "pub struct HubRuntimeState",
            "pub selected_project_path: Option<PathBuf>",
            "pub selected_template_id: String",
            "pub new_project_engine_id: Option<String>",
            "pub fn normalize(&mut self)",
            "self.selected_project_path = None;",
            "self.selected_template_id = default_selected_template_id();",
            "runtime_state_normalizes_empty_persisted_inputs",
        ],
    );
    assert_contains_all(
        "metadata.rs",
        &metadata,
        &[
            "pub type ProjectMetadataMap = BTreeMap<String, ProjectMetadata>;",
            "pub struct ProjectMetadata",
            "pub pinned: bool",
            "pub engine_id: Option<String>",
            "pub last_selected_template: Option<String>",
            "pub fn project_metadata_key",
            "pub fn project_paths_match",
            "pub fn metadata_for_path",
            "pub fn metadata_for_path_mut",
        ],
    );
}

#[test]
fn tauri_view_model_exposes_project_scope_dtos_and_visible_labels() {
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let quick_actions = read_crate_file("src/tauri_app/view_model/quick_actions.rs");
    let view_model_tests = read_crate_file("src/tauri_app/view_model/tests.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub projects: Vec<HubProjectSummary>",
            "pub browser_projects: Vec<HubRecentProject>",
            "pub recent_projects: Vec<HubRecentProject>",
            "pub selected_project: Option<HubProjectDetail>",
            "pub quick_actions: Vec<HubQuickAction>",
            "pub(crate) struct HubProjectSummary",
            "pub(crate) struct HubProjectDetail",
            "pub(crate) struct HubQuickAction",
            "projects: filtered_projects",
            "browser_projects: filtered_projects",
            "recent_projects: filtered_projects",
            "selected_project: selected_project_detail(snapshot)",
            "quick_actions: quick_actions(snapshot)",
            "fn project_summary(",
            "path: path_text(&project.path, language)",
            "relative_time(now_unix_ms(), project.last_opened_unix_ms, language)",
            "cover_id: project_cover_id(&name)",
            "fn project_detail_from_parts(",
            "pinned: metadata.is_some_and(|metadata| metadata.pinned)",
            "engine_id: metadata.and_then(|metadata| metadata.engine_id.clone())",
            "template_id: metadata.and_then(|metadata| metadata.last_selected_template.clone())",
            "template_label: project_template_label(",
            "text.pair(\"Available\", \"可用\")",
            "text.pair(\"Missing\", \"缺失\")",
            "use display::{",
        ],
    );
    assert_contains_all(
        "view_model/quick_actions.rs",
        &quick_actions,
        &[
            "fn quick_actions(snapshot: &HubSnapshot) -> Vec<HubQuickAction>",
            "HubActionId::BuildProject.as_str()",
            "HubActionId::InstallDevice.as_str()",
            "HubActionId::PackageProject.as_str()",
            "HubActionId::OpenEditor.as_str()",
        ],
    );
    assert_contains_all(
        "view_model/tests.rs",
        &view_model_tests,
        &["relative_time_uses_compact_labels"],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface HubProjectSummary",
            "path: string;",
            "modified: string;",
            "engineVersion: string;",
            "platform: string;",
            "coverId: string;",
            "export interface HubProjectDetail",
            "pinned: boolean;",
            "engineId: string | null;",
            "templateId: string | null;",
            "templateLabel: string;",
            "exists: boolean;",
            "status: string;",
            "export interface HubQuickAction",
            "detail: string;",
            "icon: string;",
            "enabled: boolean;",
        ],
    );
}

#[test]
fn project_cards_and_detail_pages_render_project_scope_dtos_passively() {
    let project_card = read_crate_file("web/src/components/data/ProjectCard.tsx");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let metrics = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");

    assert_contains_all(
        "ProjectCard.tsx",
        &project_card,
        &[
            "import type { HubProjectSummary } from \"../../types/hub\";",
            "export interface ProjectCardProps",
            "project: HubProjectSummary;",
            "selected?: boolean;",
            "onOpen?: (project: HubProjectSummary) => void;",
            "openDetailsLabel: string;",
            "borderColor: selected ? \"rgba(45,212,207,0.44)\" : hubTokens.colors.lineStrong",
            "onClick={() => onOpen?.(project)}",
            "{project.name}",
            "{project.path}",
            "{project.modified}",
            "<Chip label={project.engineVersion}",
            "<Chip label={project.platform}",
            "aria-label={`${openDetailsLabel}: ${project.name}`}",
        ],
    );
    assert_not_contains_any(
        "ProjectCard.tsx",
        &project_card,
        &["menuLabel", "projectMenuLabel"],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "const visibleProjects = useMemo(() => {",
            "state.projects.filter((project) => `${project.name} ${project.path}`.toLowerCase().includes(query))",
            "const handleOpenProject = (project: HubProjectSummary) => {",
            "void onAction(HUB_ACTION.openProjectDetail, project.id);",
            "const tableProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;",
            "const visibleRows = useMemo<HubRecentProject[]>(",
            "return tableProjects;",
            "return tableProjects.filter((project) => `${project.name} ${project.location}`.toLowerCase().includes(query));",
            "selected={project.id === state.selectedProjectId}",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const project = state.selectedProject ?? null;",
            "const statusTone: StatusTone = project?.exists ? \"success\" : \"warning\";",
            "const boundEngine = project?.engineId",
            "title: text.location, detail: project.path",
            "title: text.sourceEngine, detail: boundEngine?.name ?? project.engineVersion",
            "title: text.template, detail: project.templateLabel",
            "meta: project.pinned ? text.pinned : undefined",
            "title: text.platform, detail: project.platform",
            "<ProjectMetricsGrid project={project} boundEngine={boundEngine} text={text} />",
            "HubList items={detailRows}",
            "HubTreeView nodes={projectTree}",
        ],
    );
    assert_contains_all(
        "ProjectMetricsGrid.tsx",
        &metrics,
        &[
            "import type { HubProjectsText, HubProjectDetail, HubSourceEngineSummary } from \"../../types/hub\";",
            "project: HubProjectDetail;",
            "boundEngine?: HubSourceEngineSummary;",
            "text: HubProjectsText;",
            "MetricCard label={text.status} value={project.status}",
            "detail={project.exists ? text.ready : text.pathUnavailable}",
            "value={project.engineVersion}",
            "detail={boundEngine?.status ?? text.projectBinding}",
            "value={project.pinned ? text.pinned : text.unpinned}",
            "detail={project.templateLabel}",
        ],
    );
    assert_not_contains_any(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "project_metadata_key",
            "metadata_for_path",
            "project_paths_match",
        ],
    );
}

#[test]
fn quick_actions_and_workspace_pages_pass_scope_targets_to_runtime() {
    let quick_actions = read_crate_file("web/src/components/data/QuickActions.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let detail_sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let quick_action_runtime = read_crate_file("src/tauri_app/runtime_state/quick_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");

    assert_contains_all(
        "QuickActions.tsx",
        &quick_actions,
        &[
            "import type { HubQuickAction } from \"../../types/hub\";",
            "actions: HubQuickAction[];",
            "onAction?: (action: HubQuickAction) => void;",
            "const actionIcons = {",
            "build: BuildIcon",
            "device: PhoneIphoneIcon",
            "package: Inventory2Icon",
            "editor: OpenInNewIcon",
            "disabled={!action.enabled}",
            "if (action.enabled) {",
            "onAction?.(action);",
            "{action.title}",
            "{action.detail}",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.openEditor, undefined, projectTarget)}",
            "<ProjectDetailSidebar",
            "projectTarget={projectTarget}",
            "quickActionProjectTarget={quickActionProjectTarget}",
        ],
    );
    assert_contains_all(
        "ProjectDetailSidebar.tsx",
        &detail_sidebar,
        &[
            "projectTarget?: ProjectTargetPayload;",
            "quickActionProjectTarget?: ProjectTargetPayload;",
            "QuickActions actions={quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, projectTarget)}",
            "onClick={() => void onAction(project.pinned ? HUB_ACTION.unpinProject : HUB_ACTION.pinProject, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.removeFromHub, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.requestDelete, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.cancelDelete, undefined, projectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.confirmDelete, undefined, projectTarget)}",
        ],
    );
    assert_contains_all(
        "EditorPage.tsx",
        &editor,
        &[
            "const project = state.selectedProject;",
            "const projectTarget = projectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.openEditor, undefined, projectTarget)}",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "HubSwitch checked={Boolean(project?.exists)} label={text.projectAvailable}",
            "title: common.template, detail: project.templateLabel",
            "detail={project?.path ?? common.noProjectSelected}",
        ],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const project = state.selectedProject;",
            "id: HUB_ACTION.buildProject",
            "id: HUB_ACTION.packageProject",
            "id: HUB_ACTION.installDevice",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.buildProject, undefined, workflowProjectTarget)}",
            "void onAction(actionId, undefined, workflowProjectTarget);",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const project = state.selectedProject;",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)}",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
        ],
    );
    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
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
        ],
    );
    assert_contains_all(
        "runtime_state/editor_launch_actions.rs",
        &editor_launch_actions,
        &[
            "pub(super) fn open_selected_project_or_editor(&mut self) -> Result<(), HubError>",
            "fn selected_or_latest_recent_project_for_action(",
            "self.refresh_project_context_views(",
            "pub(in crate::tauri_app) fn prepare_background_editor_launch",
        ],
    );
    assert_contains_all(
        "runtime_state/quick_actions.rs",
        &quick_action_runtime,
        &["pub(super) fn record_action_and_persist("],
    );
    assert_contains_all(
        "runtime_state/build_actions.rs",
        &build_actions,
        &[
            "pub(super) fn build_selected_project_engine(&mut self) -> Result<(), HubError>",
            "selected_or_latest_recent_project_with_engine_for_action",
            "fn require_project_bound_engine(&self, project: &RecentProject) -> Result<(), HubError>",
            "ProjectMessageId::NoBoundSourceEngine",
            "ProjectMessageId::BoundSourceEngineUnavailable",
        ],
    );
    assert_contains_all(
        "runtime_state/project_delivery_actions.rs",
        &project_delivery_actions,
        &[
            "pub(super) fn package_recent_project(&mut self) -> Result<(), HubError>",
            "pub(super) fn install_recent_project_to_device(&mut self) -> Result<(), HubError>",
            "prepare_background_project_package",
            "prepare_background_device_install",
        ],
    );
}

#[test]
fn project_scope_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_project_scope_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_scope_contract",
            "## Project Scope Contract Cutover",
            "React/MUI project scope projection",
            "src/state/scope.rs",
            "src/settings/hub_config.rs",
            "src/projects/metadata.rs",
            "src/tauri_app/view_model.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/build_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "web/src/components/data/ProjectCard.tsx",
            "web/src/components/data/ProjectDetailSidebar.tsx",
            "web/src/components/data/ProjectMetricsGrid.tsx",
            "web/src/components/data/QuickActions.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_project_scope_contract.rs`",
            "React/MUI project scope projection",
            "project cards, detail surfaces, quick actions, and workspace workflows consume DTOs instead of recomputing scope",
        ],
    );
}

#[test]
fn project_scope_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_project_scope_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_project_scope_contract.rs",
        &contract,
        &[
            "src/state/scope.rs",
            "src/state/hub_snapshot.rs",
            "src/settings/hub_config.rs",
            "src/projects/metadata.rs",
            "src/tauri_app/view_model.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/build_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "web/src/components/data/ProjectCard.tsx",
            "web/src/components/data/ProjectDetailSidebar.tsx",
            "web/src/components/data/ProjectMetricsGrid.tsx",
            "web/src/components/data/QuickActions.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_project_scope_contract.rs",
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
