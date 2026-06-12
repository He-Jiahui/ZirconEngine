//! Static contracts for React + Material UI Projects subpage navigation.

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
            "{source_name} should contain project-navigation contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete project-navigation snippet {snippet:?}"
        );
    }
}

#[test]
fn dashboard_routes_project_subpages_and_primary_project_commands() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let create_project = read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "state.projectSubpage === \"project-browser\"",
            "state.projectSubpage === \"project-detail\"",
            "<ProjectBrowserPage state={state} onAction={onAction} />",
            "<ProjectDetailPage state={state} onAction={onAction} />",
            "const handleOpenProject = (project: HubProjectSummary) => {",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
            "onOpen={handleOpenProject}",
            "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
            "onOpenDetail={(project) => void onAction(HUB_ACTION.openProjectDetail, project.id)}",
            "onClick={() => void onAction(HUB_ACTION.viewAllProjects)}",
            "onClick={() => void onAction(HUB_ACTION.newProject)}",
            "CreateProjectDialog",
            "open={state.projectSubpage === \"new-project\"}",
            "onClose={() => void onAction(HUB_ACTION.viewAllProjects)}",
        ],
    );
    assert_contains_all(
        "CreateProjectDialog.tsx",
        &create_project,
        &["HubDialog", "onCreate", "CreateProjectPayload"],
    );
}

#[test]
fn browser_page_keeps_dashboard_new_project_filter_and_detail_navigation() {
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");

    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects",
            "const openDetail = (project: HubRecentProject) => {",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
            "void onAction(HUB_ACTION.showProjectSubpage, \"dashboard\")",
            "void onAction(HUB_ACTION.newProject)",
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "void onAction(HUB_ACTION.setProjectFilter, value)",
            "void onAction(HUB_ACTION.setProjectSort, value)",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
            "selectedProjectId={state.selectedProjectId}",
            "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
            "onOpenDetail={openDetail}",
            "HubPanel title={text.allProjects}",
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
        ],
    );
}

#[test]
fn project_table_separates_row_selection_from_detail_icon_navigation() {
    let table = read_crate_file("web/src/components/data/ProjectTable.tsx");

    assert_contains_all(
        "ProjectTable.tsx",
        &table,
        &[
            "onSelect?: (project: HubRecentProject) => void;",
            "onOpenDetail?: (project: HubRecentProject) => void;",
            "const selected = project.id === selectedProjectId;",
            "selected={selected}",
            "onClick={() => onSelect?.(project)}",
            "cursor: onSelect ? \"pointer\" : \"default\"",
            "aria-label={`${labels.openDetails}: ${project.name}`}",
            "event.stopPropagation();",
            "onOpenDetail?.(project);",
        ],
    );
    assert_not_contains_any(
        "ProjectTable.tsx",
        &table,
        &["event.clientX", "event.offsetX", "getBoundingClientRect"],
    );
}

#[test]
fn project_cards_route_corner_action_to_detail_instead_of_empty_menu() {
    let card = read_crate_file("web/src/components/data/ProjectCard.tsx");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");

    assert_contains_all(
        "ProjectCard.tsx",
        &card,
        &[
            "onOpen?: (project: HubProjectSummary) => void;",
            "openDetailsLabel: string;",
            "CardActionArea",
            "onClick={() => onOpen?.(project)}",
            "selected ? \"rgba(45,212,207,0.44)\"",
            "aria-label={`${openDetailsLabel}: ${project.name}`}",
            "event.stopPropagation();",
            "onOpen?.(project);",
            "ProjectCover coverId={project.coverId}",
            "project.engineVersion",
            "project.platform",
        ],
    );
    assert_not_contains_any(
        "ProjectCard.tsx",
        &card,
        &["menuLabel", "Project menu", "MoreVertIcon"],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &["openDetailsLabel={text.openProjectDetailsLabel}"],
    );
    assert_not_contains_any(
        "ProjectsDashboard.tsx",
        &dashboard,
        &["menuLabel={text.projectMenuLabel}"],
    );
}

#[test]
fn detail_page_routes_back_to_browser_and_project_scoped_actions() {
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");

    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const project = state.selectedProject ?? null;",
            "const projectTarget = projectTargetPayload(project);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "void onAction(HUB_ACTION.viewAllProjects)",
            "void onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
            "HubTabs",
            "{ value: \"overview\", label: text.overview }",
            "{ value: \"files\", label: text.files }",
            "{ value: \"actions\", label: text.actions }",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "ProjectDetailSidebar",
            "sourceEngines={state.sourceEngines}",
            "emptyEngineLabel={state.ui.shell.noSourceEngineRegistered}",
            "onAction={onAction}",
            "EmptyStateBlock title={text.noProjectSelected}",
        ],
    );
    assert_contains_all(
        "ProjectDetailSidebar.tsx",
        &sidebar,
        &[
            "SourceEngineList engines={sourceEngines} emptyLabel={emptyEngineLabel} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)}",
            "void onAction(HUB_ACTION.packageProject, undefined, projectTarget)",
            "void onAction(HUB_ACTION.installDevice, undefined, projectTarget)",
        ],
    );
}

#[test]
fn tauri_runtime_preserves_project_navigation_state_transitions() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let hub_api = read_crate_file("web/src/tauri/hubApi.ts");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "HubAction::ShowProjectSubpage { target_id } =>",
            "HubAction::SearchProjects { query } => self.search_projects(&query)",
            "HubAction::SetProjectFilter { target_id } =>",
            "self.set_project_filter_by_id(&target_id)?",
            "HubAction::SetProjectSort { target_id } => self.set_project_sort_by_id(&target_id)?",
            "HubAction::SetProjectViewMode { target_id } =>",
            "HubAction::SelectProject { target_id } => self.select_project_target(&target_id)?",
            "HubAction::OpenProjectDetail { target_id } => self.open_project_detail(&target_id)?",
            "HubAction::ViewAllProjects => self.view_all_projects()",
            "HubAction::NewProject =>",
            "ProjectSubpage::NewProject.id()",
            "self.select_project_target(target)?;",
            "self.project_subpage = ProjectSubpage::ProjectDetail;",
            "self.project_view_mode = ProjectViewMode::List;",
            "self.pending_delete_project_path = None;",
            "self.search_query.clear();",
            "self.project_filter = ProjectFilterMode::All;",
            "self.project_subpage = ProjectSubpage::ProjectBrowser;",
        ],
    );
    assert_contains_all(
        "hubApi.ts",
        &hub_api,
        &[
            "export async function dispatchHubAction<TActionId extends HubActionId>(",
            "invoke<unknown>(\"hub_action\"",
            "request: { actionId, targetId, payload }",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "projectFilter: string;",
            "projectSort: string;",
            "projectViewMode: string;",
            "projectSubpage: string;",
            "searchQuery: string;",
            "selectedProjectId: string | null;",
            "browserProjects: HubRecentProject[];",
            "selectedProject: HubProjectDetail | null;",
        ],
    );
}

#[test]
fn project_navigation_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_project_navigation_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_navigation_contract",
            "## Project Navigation Contract Cutover",
            "React/MUI Projects navigation",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/components/data/ProjectTable.tsx",
            "web/src/components/data/ProjectCard.tsx",
            "web/src/tauri/hubApi.ts",
            "src/tauri_app/runtime_state.rs",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_project_navigation_contract.rs`",
            "React/MUI Projects navigation",
            "dashboard, browser, detail, and new-project subpage routing",
            "shared project table row selection and detail icon navigation",
            "project-card corner action opens Project Detail instead of an empty menu",
            "Tauri project action dispatch",
        ],
    );
}

#[test]
fn project_navigation_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_project_navigation_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_project_navigation_contract.rs",
        &contract,
        &[
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/components/data/ProjectTable.tsx",
            "web/src/components/data/ProjectCard.tsx",
            "web/src/tauri/hubApi.ts",
            "src/tauri_app/runtime_state.rs",
        ],
    );
    assert_not_contains_any(
        "ui_project_navigation_contract.rs",
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
