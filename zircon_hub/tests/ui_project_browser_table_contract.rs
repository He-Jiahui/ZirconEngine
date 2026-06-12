//! Static contracts for the React/MUI Projects Browser table layout.

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
            "{source_name} should contain Project Browser table snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete Project Browser table snippet {snippet:?}"
        );
    }
}

#[test]
fn project_table_owns_recent_project_columns_selection_and_detail_action() {
    let table = read_crate_file("web/src/components/data/ProjectTable.tsx");

    assert_contains_all(
        "ProjectTable.tsx",
        &table,
        &[
            "import { Box, IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from \"@mui/material\";",
            "export interface ProjectTableProps",
            "projects: HubRecentProject[];",
            "selectedProjectId: string | null;",
            "onSelect?: (project: HubRecentProject) => void;",
            "onOpenDetail?: (project: HubRecentProject) => void;",
            "<Box sx={{ overflowX: \"auto\", minWidth: 0 }}>",
            "<Table size=\"small\" sx={{ tableLayout: \"fixed\", minWidth: 560 }}>",
            "<HeaderCell width=\"32%\">{labels.name}</HeaderCell>",
            "<HeaderCell width=\"18%\">{labels.engineVersion}</HeaderCell>",
            "<HeaderCell width=\"16%\">{labels.lastModified}</HeaderCell>",
            "<HeaderCell>{labels.location}</HeaderCell>",
            "<HeaderCell width={42} />",
            "const selected = project.id === selectedProjectId;",
            "selected={selected}",
            "onClick={() => onSelect?.(project)}",
            "ProjectCover coverId={project.coverId} size=\"thumb\"",
            "{project.name}",
            "<BodyCell>{project.engineVersion}</BodyCell>",
            "<BodyCell>{project.modified}</BodyCell>",
            "<BodyCell>{project.location}</BodyCell>",
            "aria-label={`${labels.openDetails}: ${project.name}`}",
            "event.stopPropagation();",
            "onOpenDetail?.(project);",
        ],
    );
    assert_not_contains_any(
        "ProjectTable.tsx",
        &table,
        &[
            "event.clientX",
            "event.offsetX",
            "getBoundingClientRect",
            "position: \"absolute\"",
        ],
    );
}

#[test]
fn project_browser_page_feeds_table_from_filtered_browser_projects() {
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");

    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "const [search, setSearch] = useState(state.searchQuery);",
            "const [filter, setFilter] = useState(state.projectFilter);",
            "const [sort, setSort] = useState(state.projectSort);",
            "const [viewMode, setViewMode] = useState(state.projectViewMode);",
            "setSearch(state.searchQuery);",
            "setFilter(state.projectFilter);",
            "setSort(state.projectSort);",
            "setViewMode(state.projectViewMode);",
            "const browserProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;",
            "const visibleRows = useMemo(() => {",
            "const query = search.trim().toLowerCase();",
            "return browserProjects.filter((project) => `${project.name} ${project.location}`.toLowerCase().includes(query));",
            "const openDetail = (project: HubRecentProject) => {",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
            "ProjectTable",
            "projects={visibleRows}",
            "selectedProjectId={state.selectedProjectId}",
            "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
            "onOpenDetail={openDetail}",
        ],
    );
}

#[test]
fn dashboard_table_rows_consume_backend_recent_project_dtos_without_language_parsing() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "const tableProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;",
            "const visibleRows = useMemo<HubRecentProject[]>(() => {",
            "return tableProjects.filter((project) => `${project.name} ${project.location}`.toLowerCase().includes(query));",
            "projects={visibleRows}",
        ],
    );
    assert_not_contains_any(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            ".replace(/^Modified",
            "Modified\\s+",
            "location: project.path",
        ],
    );
}

#[test]
fn project_browser_toolbar_and_panel_layout_stay_responsive_and_componentized() {
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");

    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "HubButton startIcon={<DashboardCustomizeOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.showProjectSubpage, \"dashboard\")}",
            "HubButton tone=\"primary\" startIcon={<AddIcon />} onClick={() => void onAction(HUB_ACTION.newProject)}",
            "HubStatusBanner task={state.taskSummary}",
            "HubSearchField",
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "HubSelect",
            "void onAction(HUB_ACTION.setProjectFilter, value)",
            "void onAction(HUB_ACTION.setProjectSort, value)",
            "HubToggle",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
            "gridTemplateColumns: \"minmax(280px, 420px) 1fr auto auto auto\"",
            "gridTemplateColumns: \"minmax(240px, 1fr) auto auto\"",
            "gridTemplateColumns: \"1fr\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(320px, 0.42fr)\"",
            "HubPanel title={text.allProjects}",
            "EmptyStateBlock title={text.noProjectsFound} detail={text.noRecentProjectMatches}",
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
            "QuickActions actions={state.quickActions}",
            "SourceEngineList engines={state.sourceEngines}",
        ],
    );
    assert_not_contains_any(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "<Table",
            "<TableRow",
            "<TableCell",
            "<IconButton",
            "from \"@mui/material\";\nimport { Table",
            "void onAction(HUB_ACTION.searchProjects, value)",
        ],
    );
}

#[test]
fn view_model_and_types_project_browser_rows_remain_camel_case_dtos() {
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub browser_projects: Vec<HubRecentProject>",
            "pub recent_projects: Vec<HubRecentProject>",
            "#[serde(rename_all = \"camelCase\")]",
            "pub(crate) struct HubRecentProject",
            "pub engine_version: String",
            "pub location: String",
            "pub cover_id: String",
            "browser_projects: filtered_projects",
            ".map(|project| recent_project_row(snapshot, project))",
            "recent_projects: filtered_projects",
            ".take(RECENT_ROW_LIMIT)",
            "fn recent_project_row(snapshot: &HubSnapshot, project: &RecentProject) -> HubRecentProject",
            "location: summary.path",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface HubRecentProject",
            "id: string;",
            "name: string;",
            "engineVersion: string;",
            "modified: string;",
            "location: string;",
            "coverId: string;",
            "browserProjects: HubRecentProject[];",
            "recentProjects: HubRecentProject[];",
            "selectedProjectId: string | null;",
        ],
    );
}

#[test]
fn project_browser_table_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_project_browser_table_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_browser_table_contract",
            "## Project Browser Table Contract Cutover",
            "React/MUI Project Browser table",
            "web/src/components/data/ProjectTable.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "src/tauri_app/view_model.rs",
            "web/src/types/hub.ts",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_project_browser_table_contract.rs`",
            "React/MUI Project Browser table",
            "ProjectTable column model, row selection, detail icon action, ProjectBrowserPage filters, ProjectsDashboard table rows, and `browserProjects`/`recentProjects` DTOs",
        ],
    );
}

#[test]
fn project_browser_table_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_project_browser_table_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_project_browser_table_contract.rs",
        &contract,
        &[
            "web/src/components/data/ProjectTable.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "src/tauri_app/view_model.rs",
            "web/src/types/hub.ts",
        ],
    );
    assert_not_contains_any(
        "ui_project_browser_table_contract.rs",
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
