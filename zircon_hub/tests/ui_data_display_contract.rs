//! Static contracts for React + Material UI Hub data-display semantics.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {path}: {error}");
        }),
    )
}

fn assert_contains_all(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_path} must contain React/MUI data-display snippet `{snippet}`"
        );
    }
}

fn assert_not_contains_any(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_path} must not contain obsolete React/MUI data-display snippet `{snippet}`"
        );
    }
}

#[test]
fn display_atoms_own_status_empty_and_metric_state() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/StatusBadge.tsx",
            vec![
                "StatusTone",
                "toneMap",
                "PlayArrowIcon",
                "CheckCircleIcon",
                "WarningIcon",
                "ErrorIcon",
                "height: 36",
                "minWidth: 112",
                "border: `1px solid ${toneStyle.border}`",
                "Typography variant=\"body2\"",
                "tone === \"running\"",
            ],
        ),
        (
            "web/src/components/data/EmptyStateBlock.tsx",
            vec![
                "EmptyStateBlockProps",
                "minHeight: 148",
                "placeItems: \"center\"",
                "border: `1px dashed",
                "textAlign: \"center\"",
                "Typography variant=\"body2\"",
                "Typography variant=\"caption\"",
            ],
        ),
        (
            "web/src/components/data/MetricCard.tsx",
            vec![
                "MetricCardProps",
                "tone?: \"neutral\" | \"accent\" | \"success\" | \"warning\" | \"error\"",
                "toneColor",
                "gridTemplateColumns: icon ? \"34px minmax(0, 1fr)\" : \"1fr\"",
                "hubTokens.radius.panel",
                "Typography variant=\"caption\" noWrap",
                "Typography variant=\"h6\" noWrap",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn project_cards_covers_and_tables_share_project_display_atoms() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/ProjectCover.tsx",
            vec![
                "coverById",
                "brandMark",
                "size?: \"card\" | \"thumb\"",
                "objectFit: \"cover\"",
                "pointerEvents: \"none\"",
                "!thumb ?",
            ],
        ),
        (
            "web/src/components/data/ProjectCard.tsx",
            vec![
                "CardActionArea",
                "HubProjectSummary",
                "ProjectCover",
                "selected ?",
                "onOpen?.(project)",
                "OpenInNewOutlinedIcon",
                "Chip",
                "chipSx",
            ],
        ),
        (
            "web/src/components/data/ProjectTable.tsx",
            vec![
                "Table size=\"small\"",
                "tableLayout: \"fixed\"",
                "HubRecentProject",
                "selectedProjectId",
                "ProjectCover",
                "onSelect?.(project)",
                "onOpenDetail?.(project)",
                "event.stopPropagation()",
                "HeaderCell",
                "BodyCell",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn row_like_components_keep_shared_material_row_contracts() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/HubList.tsx",
            vec![
                "ListItemButton",
                "const hasSelectHandler = Boolean(onSelect);",
                "const itemDisabled = item.disabled || !hasSelectHandler;",
                "selected={item.selected}",
                "disabled={itemDisabled}",
                "onSelect?.(item)",
                "cursor: hasSelectHandler && !item.disabled ? \"pointer\" : \"default\"",
                "ListItemIcon",
                "ListItemText",
                "Typography variant=\"body2\" noWrap",
                "Typography variant=\"caption\" noWrap",
                "item.meta",
            ],
        ),
        (
            "web/src/components/data/HubTreeView.tsx",
            vec![
                "useState",
                "new Set(defaultExpanded)",
                "Collapse in={open}",
                "depth * 2",
                "ExpandMoreIcon",
                "ChevronRightIcon",
                "const hasSelectHandler = Boolean(onSelect);",
                "const rowIsActionable = childCount > 0 || hasSelectHandler;",
                "disabled={!rowIsActionable}",
                "onSelect?.(node)",
            ],
        ),
        (
            "web/src/components/data/QuickActions.tsx",
            vec![
                "ButtonBase",
                "HubQuickAction",
                "actionIcons",
                "gridTemplateColumns: \"36px minmax(0, 1fr) 24px\"",
                "ChevronRightIcon",
                "onAction?.(action)",
                "Typography variant=\"body2\" noWrap",
                "Typography variant=\"caption\" noWrap",
            ],
        ),
        (
            "web/src/components/data/SourceEngineList.tsx",
            vec![
                "ButtonBase",
                "HubSourceEngineSummary",
                "emptyLabel: string;",
                "StatusBadge",
                "{emptyLabel}",
                "gridTemplateColumns: \"40px minmax(0, 1fr) auto\"",
                "engine.active ?",
                "const hasSelectHandler = Boolean(onSelect);",
                "disabled={!hasSelectHandler}",
                "cursor: hasSelectHandler ? \"pointer\" : \"default\"",
                "onSelect?.(engine)",
                "StatusBadge label={engine.status} tone={engine.active ? \"success\" : \"neutral\"}",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn panel_display_components_keep_reusable_slots_without_reference_samples() {
    for (source_path, snippets) in [(
        "web/src/components/data/HubPanel.tsx",
        vec![
            "HubPanelProps",
            "PropsWithChildren",
            "action?: ReactNode",
            "Card",
            "component=\"section\"",
            "overflow: \"hidden\"",
            "{children}",
        ],
    )] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
    let data_index = read_crate_file("web/src/components/data/index.ts");
    assert_not_contains_any(
        "web/src/components/data/index.ts",
        &data_index,
        &["ButtonStatesPanel"],
    );
}

#[test]
fn dashboard_browser_detail_and_catalog_pages_consume_display_atoms() {
    for (source_path, snippets) in [
        (
            "web/src/pages/ProjectsDashboard.tsx",
            vec![
                "from \"../components/data\"",
                "EmptyStateBlock",
                "ProjectCard",
                "ProjectTable",
                "QuickActions",
                "visibleRows",
                "dashboardProjects.map",
                "projects={visibleRows}",
                "projects={state.recentProjects}",
            ],
        ),
        (
            "web/src/pages/ProjectBrowserPage.tsx",
            vec![
                "from \"../components/data\"",
                "EmptyStateBlock",
                "ProjectTable",
                "QuickActions",
                "SourceEngineList",
                "browserProjects",
                "visibleRows",
            ],
        ),
        (
            "web/src/pages/ProjectDetailPage.tsx",
            vec![
                "from \"../components/data\"",
                "ProjectCover",
                "MetricCard",
                "HubList",
                "HubTreeView",
                "StatusBadge",
                "QuickActions",
                "SourceEngineList",
                "detailRows",
                "projectTree",
            ],
        ),
        (
            "web/src/pages/CatalogPage.tsx",
            vec![
                "from \"../components/data\"",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "StatusBadge",
                "EmptyStateBlock",
                "catalogRows",
                "filterRows",
                "groupBy",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
        assert_page_material_imports_do_not_include_data_primitives(source_path, &source);
    }
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    assert_not_contains_any(
        "web/src/pages/ProjectsDashboard.tsx",
        &dashboard,
        &["ButtonStatesPanel", "text.buttonStates"],
    );
}

#[test]
fn workspace_pages_share_history_status_and_tree_display_atoms() {
    for (source_path, snippets) in [
        (
            "web/src/pages/EditorPage.tsx",
            vec![
                "from \"../components/data\"",
                "EmptyStateBlock",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "StatusBadge",
                "editorActivity",
                "editorTree",
            ],
        ),
        (
            "web/src/pages/BuildsPage.tsx",
            vec![
                "from \"../components/data\"",
                "EmptyStateBlock",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "StatusBadge",
                "buildHistory",
                "historyRow",
                "BuildActionDetail",
            ],
        ),
        (
            "web/src/pages/CloudPage.tsx",
            vec![
                "from \"../components/data\"",
                "EmptyStateBlock",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "StatusBadge",
                "packageActions",
                "installActions",
                "serviceSlots",
            ],
        ),
        (
            "web/src/pages/TeamPage.tsx",
            vec![
                "from \"../components/data\"",
                "EmptyStateBlock",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "StatusBadge",
                "memberRows",
                "actionRows",
                "ActionDetail",
            ],
        ),
        (
            "web/src/pages/SettingsPage.tsx",
            vec![
                "from \"../components/data\"",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "SourceEngineList",
                "StatusBadge",
                "healthRows",
                "pathTree",
            ],
        ),
        (
            "web/src/pages/WorkspacePage.tsx",
            vec![
                "from \"../components/data\"",
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "settingsRows",
                "sourceTree",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
        assert_page_material_imports_do_not_include_data_primitives(source_path, &source);
    }
}

#[test]
fn backend_display_dtos_feed_shared_react_rows() {
    let types = read_crate_file("web/src/types/hub.ts");
    assert_contains_all(
        "web/src/types/hub.ts",
        &types,
        &[
            "export type StatusTone",
            "export interface HubTaskSummary",
            "export interface HubProjectSummary",
            "export interface HubRecentProject",
            "export interface HubProjectDetail",
            "export interface HubQuickAction",
            "export interface HubSourceEngineSummary",
            "export type HubActionHistoryKind",
            "export interface HubActionHistoryItem",
            "kind: HubActionHistoryKind;",
            "taskSummary: HubTaskSummary",
            "projects: HubProjectSummary[]",
            "browserProjects: HubRecentProject[]",
            "recentProjects: HubRecentProject[]",
            "quickActions: HubQuickAction[]",
            "sourceEngines: HubSourceEngineSummary[]",
            "actionHistory: HubActionHistoryItem[]",
        ],
    );
}

#[test]
fn data_display_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_data_display_contract.rs");
    let obsolete_suffix = [".", "slint"].concat();
    let obsolete_reader = ["read", "_ui", "_file"].concat();
    let obsolete_root_reader = ["ui", "_dir"].concat();
    let obsolete_app_path = ["src", "app"].join("/");

    for forbidden in [
        obsolete_suffix,
        obsolete_reader,
        obsolete_root_reader,
        obsolete_app_path,
    ] {
        assert!(
            !contract.contains(&forbidden),
            "React data-display contract must not keep old UI contract reference `{forbidden}`"
        );
    }
}

fn assert_page_material_imports_do_not_include_data_primitives(source_path: &str, source: &str) {
    let forbidden_material_imports = [
        "Card",
        "CardActionArea",
        "Chip",
        "List",
        "ListItemButton",
        "ListItemIcon",
        "ListItemText",
        "Table",
        "TableBody",
        "TableCell",
        "TableHead",
        "TableRow",
    ];

    for line in source.lines().filter(|line| line.contains("@mui/material")) {
        for forbidden in forbidden_material_imports {
            assert!(
                !line.contains(forbidden),
                "{source_path} must consume shared data-display components instead of importing raw Material data primitive `{forbidden}`"
            );
        }
    }
}
