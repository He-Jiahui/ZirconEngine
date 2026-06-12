//! Static contracts for shared React/MUI Hub table, list, and tree-view primitives.

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
            "{source_name} should contain table/list/tree snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete table/list/tree snippet {snippet:?}"
        );
    }
}

#[test]
fn data_barrel_exports_table_list_and_tree_component_family() {
    let data_index = read_crate_file("web/src/components/data/index.ts");

    assert_contains_all(
        "components/data/index.ts",
        &data_index,
        &[
            "export * from \"./HubList\";",
            "export * from \"./HubTreeView\";",
            "export * from \"./ProjectCover\";",
            "export * from \"./ProjectTable\";",
        ],
    );
}

#[test]
fn project_table_owns_material_column_model_selection_and_detail_action() {
    let table = read_crate_file("web/src/components/data/ProjectTable.tsx");
    let hub_types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "ProjectTable.tsx",
        &table,
        &[
            "import { Box, IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from \"@mui/material\";",
            "import type { HubRecentProject } from \"../../types/hub\";",
            "import { ProjectCover } from \"./ProjectCover\";",
            "export interface ProjectTableProps",
            "projects: HubRecentProject[];",
            "selectedProjectId: string | null;",
            "labels: {",
            "onSelect?: (project: HubRecentProject) => void;",
            "onOpenDetail?: (project: HubRecentProject) => void;",
            "<Box sx={{ overflowX: \"auto\", minWidth: 0 }}>",
            "<Table size=\"small\" sx={{ tableLayout: \"fixed\", minWidth: 560 }}>",
            "<TableHead>",
            "<HeaderCell width=\"32%\">{labels.name}</HeaderCell>",
            "<HeaderCell width=\"18%\">{labels.engineVersion}</HeaderCell>",
            "<HeaderCell width=\"16%\">{labels.lastModified}</HeaderCell>",
            "<HeaderCell>{labels.location}</HeaderCell>",
            "const selected = project.id === selectedProjectId;",
            "onClick={() => onSelect?.(project)}",
            "cursor: onSelect ? \"pointer\" : \"default\"",
            "\"&.Mui-selected, &.Mui-selected:hover\"",
            "<ProjectCover coverId={project.coverId} size=\"thumb\" />",
            "<BodyCell>{project.engineVersion}</BodyCell>",
            "<BodyCell>{project.modified}</BodyCell>",
            "<BodyCell>{project.location}</BodyCell>",
            "aria-label={`${labels.openDetails}: ${project.name}`}",
            "event.stopPropagation();",
            "onOpenDetail?.(project);",
            "function HeaderCell",
            "function BodyCell",
            "Typography variant=\"body2\" noWrap",
        ],
    );
    assert_contains_all(
        "hub.ts",
        &hub_types,
        &[
            "export interface HubRecentProject",
            "engineVersion: string;",
            "modified: string;",
            "location: string;",
            "coverId: string;",
            "recentProjects: HubRecentProject[];",
            "browserProjects: HubRecentProject[];",
        ],
    );
    assert_not_contains_any(
        "ProjectTable.tsx",
        &table,
        &["<Card", "<List", "DataGrid", "readUiFile", "HubTableView"],
    );
}

#[test]
fn hub_list_owns_dense_material_row_model_and_optional_slots() {
    let list = read_crate_file("web/src/components/data/HubList.tsx");

    assert_contains_all(
        "HubList.tsx",
        &list,
        &[
            "import { Box, List, ListItemButton, ListItemIcon, ListItemText, Typography } from \"@mui/material\";",
            "export interface HubListItem",
            "id: string;",
            "title: string;",
            "detail?: string;",
            "secondaryDetail?: string;",
            "meta?: string;",
            "icon?: ReactNode;",
            "selected?: boolean;",
            "disabled?: boolean;",
            "export interface HubListProps",
            "items: HubListItem[];",
            "onSelect?: (item: HubListItem) => void;",
            "const hasSelectHandler = Boolean(onSelect);",
            "const itemDisabled = item.disabled || !hasSelectHandler;",
            "<List dense sx={{ display: \"grid\", gap: 0.7, p: 0 }}>",
            "selected={item.selected}",
            "disabled={itemDisabled}",
            "onClick={() => onSelect?.(item)}",
            "cursor: hasSelectHandler && !item.disabled ? \"pointer\" : \"default\"",
            "minHeight: item.secondaryDetail ? 64 : 48",
            "borderRadius: `${hubTokens.radius.compact}px`",
            "item.selected ? \"rgba(45,212,207,0.34)\" : hubTokens.colors.lineStrong",
            "{item.icon ? <ListItemIcon",
            "<ListItemText",
            "primary={<Typography variant=\"body2\" noWrap>{item.title}</Typography>}",
            "item.detail || item.secondaryDetail ? (",
            "{item.secondaryDetail ? (",
            "{item.meta ? (",
        ],
    );
    assert_not_contains_any("HubList.tsx", &list, &["<Table", "<Card", "DataGrid"]);
}

#[test]
fn hub_tree_view_owns_recursive_disclosure_and_collapse_model() {
    let tree = read_crate_file("web/src/components/data/HubTreeView.tsx");

    assert_contains_all(
        "HubTreeView.tsx",
        &tree,
        &[
            "import { useState } from \"react\";",
            "import ChevronRightIcon from \"@mui/icons-material/ChevronRight\";",
            "import ExpandMoreIcon from \"@mui/icons-material/ExpandMore\";",
            "import { Box, Collapse, List, ListItemButton, Typography } from \"@mui/material\";",
            "export interface HubTreeNode",
            "label: string;",
            "detail?: string;",
            "children?: HubTreeNode[];",
            "export interface HubTreeViewProps",
            "nodes: HubTreeNode[];",
            "defaultExpanded?: string[];",
            "onSelect?: (node: HubTreeNode) => void;",
            "const hasSelectHandler = Boolean(onSelect);",
            "const [expanded, setExpanded] = useState(() => new Set(defaultExpanded));",
            "const hasChildren = (node.children?.length ?? 0) > 0;",
            "if (!hasSelectHandler && !hasChildren) {",
            "onSelect?.(node);",
            "const next = new Set(current);",
            "<TreeNode key={node.id} node={node} depth={0} expanded={expanded} hasSelectHandler={hasSelectHandler} onToggle={toggle} />",
            "function TreeNode",
            "const childCount = node.children?.length ?? 0;",
            "const open = expanded.has(node.id);",
            "const rowIsActionable = childCount > 0 || hasSelectHandler;",
            "const Icon = childCount > 0 && open ? ExpandMoreIcon : ChevronRightIcon;",
            "disabled={!rowIsActionable}",
            "pl: 0.8 + depth * 2",
            "cursor: rowIsActionable ? \"pointer\" : \"default\"",
            "<Collapse in={open} timeout={140} unmountOnExit>",
            "<TreeNode key={child.id} node={child} depth={depth + 1} expanded={expanded} hasSelectHandler={hasSelectHandler} onToggle={onToggle} />",
        ],
    );
    assert_not_contains_any("HubTreeView.tsx", &tree, &["<Table", "<Card", "TreeItem"]);
}

#[test]
fn routed_pages_consume_shared_table_list_and_tree_views() {
    for (page, snippets) in [
        (
            "ProjectsDashboard.tsx",
            vec![
                "ProjectTable",
                "projects={visibleRows}",
                "projects={state.recentProjects}",
                "onOpenDetail={(project) => void onAction(HUB_ACTION.openProjectDetail, project.id)}",
            ],
        ),
        (
            "ProjectBrowserPage.tsx",
            vec![
                "ProjectTable",
                "projects={visibleRows}",
                "onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}",
                "onOpenDetail={openDetail}",
            ],
        ),
        (
            "ProjectDetailPage.tsx",
            vec![
                "HubList items={detailRows}",
                "HubTreeView nodes={projectTree} defaultExpanded={[\"project-root\"]}",
            ],
        ),
        (
            "EditorPage.tsx",
            vec![
                "HubList",
                "items={editorPlugins.map((plugin) => ({",
                "HubTreeView nodes={editorTree} defaultExpanded={[\"editor-workspace\", \"source-engines\", \"editor-plugins\"]}",
            ],
        ),
        (
            "BuildsPage.tsx",
            vec![
                "items={workflowRows}",
                "void onAction(actionId, undefined, workflowProjectTarget);",
                "items={buildHistory.map(historyRow)}",
                "HubTreeView nodes={buildTree} defaultExpanded={[\"builds\", \"history\"]}",
            ],
        ),
        (
            "CatalogPage.tsx",
            vec![
                "HubList",
                "items={visibleRows.map((row) => ({",
                "HubTreeView nodes={treeNodes} defaultExpanded={[`${mode}-catalog`]}",
            ],
        ),
        (
            "CloudPage.tsx",
            vec![
                "HubList",
                "items={packageActions.map((action) => ({",
                "HubTreeView nodes={outputTree} defaultExpanded={[\"cloud\", \"services\"]}",
            ],
        ),
        (
            "TeamPage.tsx",
            vec![
                "HubList items={memberRows}",
                "HubList items={actionRows}",
                "HubTreeView nodes={teamTree} defaultExpanded={[\"repository\", \"identity\", \"contributors\"]}",
            ],
        ),
        (
            "SettingsPage.tsx",
            vec![
                "<SettingsSection",
                "healthRows={healthRows}",
                "pathTree={pathTree}",
                "() => draftSettings.health.rows.map((row) => ({ ...row, disabled: false }))",
            ],
        ),
        (
            "WorkspacePage.tsx",
            vec![
                "HubList items={settingsRows}",
                "HubTreeView nodes={sourceTree} defaultExpanded={[\"workspace\", \"source-engines\", \"paths\"]}",
            ],
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(page, &source, &snippets);
        assert_not_contains_any(
            page,
            &source,
            &[
                " Table,",
                " TableBody",
                " TableCell",
                " TableHead",
                " TableRow",
                " List,",
                " ListItemButton",
                " TreeItem",
                " DataGrid",
            ],
        );
    }

    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "HubList items={healthRows}",
            "HubTreeView nodes={pathTree} defaultExpanded={[\"settings-root\"]}",
        ],
    );
    assert_not_contains_any(
        "SettingsSection.tsx",
        &settings_section,
        &[
            " Table,",
            " TableBody",
            " TableCell",
            " TableHead",
            " TableRow",
        ],
    );
}

#[test]
fn table_view_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_table_view_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_table_view_contract",
            "## Table View Contract Cutover",
            "React/MUI table/list/tree view system",
            "web/src/components/data/ProjectTable.tsx",
            "web/src/components/data/HubList.tsx",
            "web/src/components/data/HubTreeView.tsx",
            "web/src/pages",
            "ProjectTable column model, HubList row model, and HubTreeView recursive tree model",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_table_view_contract.rs`",
            "React/MUI table/list/tree view system",
            "ProjectTable column model, HubList row model, and HubTreeView recursive tree model",
            "routed pages consume shared table/list/tree components",
        ],
    );
}

#[test]
fn table_view_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_table_view_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_table_view_contract.rs",
        &contract,
        &[
            "web/src/components/data/ProjectTable.tsx",
            "web/src/components/data/HubList.tsx",
            "web/src/components/data/HubTreeView.tsx",
            "web/src/components/data/SettingsSection.tsx",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/pages/WorkspacePage.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_table_view_contract.rs",
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
