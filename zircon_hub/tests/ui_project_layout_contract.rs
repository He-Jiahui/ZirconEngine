//! Static contracts for React + Material UI Projects layout composition.

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
            "{source_name} should contain Projects layout contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete or page-local Projects layout snippet {snippet:?}"
        );
    }
}

#[test]
fn dashboard_routes_project_subpages_and_owns_toolbar_grid_state() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let toolbar = read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "ProjectBrowserPage",
            "ProjectDetailPage",
            "state.projectSubpage === \"project-browser\"",
            "state.projectSubpage === \"project-detail\"",
            "state.searchQuery",
            "state.projectFilter",
            "state.projectSort",
            "state.projectViewMode",
            "setSearch(state.searchQuery)",
            "setFilter(state.projectFilter)",
            "setSort(state.projectSort)",
            "setViewMode(state.projectViewMode)",
            "ProjectsToolbar",
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "void onAction(HUB_ACTION.setProjectFilter, value)",
            "void onAction(HUB_ACTION.setProjectSort, value)",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
        ],
    );
    assert_contains_all(
        "ProjectsToolbar.tsx",
        &toolbar,
        &[
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "gridTemplateColumns: \"minmax(260px, 307px) 1fr auto auto auto\"",
            "gridTemplateColumns: \"minmax(240px, 1fr) auto auto\"",
            "gridTemplateColumns: \"1fr\"",
        ],
    );
}

#[test]
fn dashboard_composes_cards_table_recent_actions_and_new_project_dialog() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let create_project = read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");
    let project_card = read_crate_file("web/src/components/data/ProjectCard.tsx");
    let project_table = read_crate_file("web/src/components/data/ProjectTable.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "visibleProjects",
            "const dashboardProjects = useMemo(() => visibleProjects.slice(0, 4), [visibleProjects]);",
            "visibleRows",
            "const tableProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;",
            "ProjectCard",
            "ProjectCardRail",
            "ProjectTable",
            "EmptyStateBlock",
            "HubPanel title={text.projectBrowser}",
            "title={text.recentProjects}",
            "HubPanel title={text.quickActions}",
            "CreateProjectDialog",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
        ],
    );
    assert_contains_all(
        "CreateProjectDialog.tsx",
        &create_project,
        &[
            "HubDialog",
            "label={text.projectName}",
            "label={text.location}",
            "HubComboBox",
        ],
    );
    assert_not_contains_any(
        "ProjectsDashboard.tsx",
        &dashboard,
        &["ButtonStatesPanel", "text.buttonStates"],
    );
    assert_contains_all(
        "ProjectCard.tsx",
        &project_card,
        &[
            "HubProjectSummary",
            "ProjectCover",
            "CardActionArea",
            "onOpen?.(project)",
            "project.engineVersion",
            "project.platform",
        ],
    );
    assert_contains_all(
        "ProjectTable.tsx",
        &project_table,
        &[
            "HubRecentProject",
            "ProjectCover",
            "selectedProjectId",
            "onSelect?.(project)",
            "onOpenDetail?.(project)",
            "HeaderCell",
            "BodyCell",
        ],
    );
}

#[test]
fn browser_page_uses_shared_toolbar_table_and_side_panel_layout() {
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");

    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "HubStatusBanner",
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "ProjectTable",
            "QuickActions",
            "SourceEngineList",
            "browserProjects",
            "state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects",
            "visibleRows",
            "toLowerCase().includes(query)",
            "openDetail",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
            "gridTemplateColumns: \"minmax(280px, 420px) 1fr auto auto auto\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(320px, 0.42fr)\"",
            "HubPanel title={text.allProjects}",
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
            "selectedProjectId={state.selectedProjectId}",
            "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
        ],
    );
}

#[test]
fn detail_page_uses_metric_tabs_media_main_and_sidebar_layout() {
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let metrics = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");
    let sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");

    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "state.selectedProject ?? null",
            "const statusTone: StatusTone = project?.exists ? \"success\" : \"warning\"",
            "const boundEngine = project?.engineId",
            "const detailRows = useMemo",
            "const projectTree = useMemo",
            "HubStatusBanner",
            "ProjectMetricsGrid",
            "ProjectDetailSidebar",
            "HubTabs",
            "ProjectCover",
            "HubList",
            "HubTreeView",
            "QuickActions",
            "StatusBadge",
            "EmptyStateBlock title={text.noProjectSelected} detail={text.chooseProjectFromBrowser}",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.4fr)\"",
            "gridTemplateColumns: \"minmax(220px, 0.36fr) minmax(0, 1fr)\"",
            "HubPanel title={text.projectOverview}",
            "HubPanel title={text.projectTree}",
            "HubPanel title={text.projectActions}",
            "const projectTarget = projectTargetPayload(project);",
            "void onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
        ],
    );
    assert_contains_all(
        "ProjectMetricsGrid.tsx",
        &metrics,
        &[
            "MetricCard",
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"repeat(2, minmax(0, 1fr))\"",
        ],
    );
    assert_contains_all(
        "ProjectDetailSidebar.tsx",
        &sidebar,
        &[
            "SourceEngineList",
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
            "HubPanel title={text.package}",
            "void onAction(HUB_ACTION.packageProject, undefined, projectTarget)",
            "void onAction(HUB_ACTION.installDevice, undefined, projectTarget)",
        ],
    );
}

#[test]
fn project_types_preserve_dashboard_browser_and_detail_data_contracts() {
    let types = read_crate_file("web/src/types/hub.ts");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");

    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface HubProjectSummary",
            "export interface HubRecentProject",
            "export interface HubProjectDetail",
            "projectFilter: string;",
            "projectSort: string;",
            "projectViewMode: string;",
            "projectSubpage: string;",
            "searchQuery: string;",
            "selectedProjectId: string | null;",
            "projects: HubProjectSummary[];",
            "browserProjects: HubRecentProject[];",
            "recentProjects: HubRecentProject[];",
            "selectedProject: HubProjectDetail | null;",
            "quickActions: HubQuickAction[];",
            "sourceEngines: HubSourceEngineSummary[];",
        ],
    );

    for (name, source) in [
        ("ProjectsDashboard.tsx", dashboard),
        ("ProjectBrowserPage.tsx", browser),
        ("ProjectDetailPage.tsx", detail),
    ] {
        assert_not_contains_any(
            name,
            &source,
            &[
                "from \"@mui/material\";\nimport { Button",
                "from \"@mui/material\";\nimport { Card",
                "<Card",
                "<Table",
                "<ListItemButton",
                "<TextField",
                "<Select",
            ],
        );
    }
}

#[test]
fn project_layout_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_project_layout_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_layout_contract",
            "## Project Layout Contract Cutover",
            "React/MUI Projects layout",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/types/hub.ts",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_project_layout_contract.rs`",
            "React/MUI Projects layout",
            "dashboard/browser/detail route split",
            "shared project card, table, metric, tree, list, quick-action, source-engine, and dialog components",
        ],
    );
}

#[test]
fn project_layout_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_project_layout_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_project_layout_contract.rs",
        &contract,
        &[
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/components/data/ProjectCard.tsx",
            "web/src/components/data/ProjectTable.tsx",
            "web/src/types/hub.ts",
        ],
    );
    assert_not_contains_any(
        "ui_project_layout_contract.rs",
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
