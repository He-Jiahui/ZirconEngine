//! Static contracts for React + Material UI data container components.

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
            "{source_path} must contain React data-container snippet `{snippet}`"
        );
    }
}

fn assert_not_contains_any(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_path} must not contain obsolete React data-container snippet `{snippet}`"
        );
    }
}

#[test]
fn data_components_are_reexported_from_the_react_barrel() {
    let index = read_crate_file("web/src/components/data/index.ts");
    assert_contains_all(
        "web/src/components/data/index.ts",
        &index,
        &[
            "EmptyStateBlock",
            "HubList",
            "HubPanel",
            "HubTreeView",
            "MetricCard",
            "ProjectCard",
            "ProjectCover",
            "ProjectTable",
            "QuickActions",
            "SourceEngineList",
            "StatusBadge",
        ],
    );
    assert_not_contains_any(
        "web/src/components/data/index.ts",
        &index,
        &["ButtonStatesPanel"],
    );
}

#[test]
fn list_tree_and_table_components_own_material_data_primitives() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/HubList.tsx",
            vec![
                "List,",
                "ListItemButton",
                "ListItemIcon",
                "ListItemText",
                "HubListItem",
                "selected?: boolean",
                "disabled?: boolean",
                "onSelect?.(item)",
                "hubTokens.radius.compact",
            ],
        ),
        (
            "web/src/components/data/HubTreeView.tsx",
            vec![
                "Collapse",
                "List",
                "ListItemButton",
                "HubTreeNode",
                "defaultExpanded",
                "new Set(defaultExpanded)",
                "TreeNode",
                "depth * 2",
                "onToggle(node)",
            ],
        ),
        (
            "web/src/components/data/ProjectTable.tsx",
            vec![
                "Table,",
                "TableBody",
                "TableCell",
                "TableHead",
                "TableRow",
                "HubRecentProject",
                "selectedProjectId",
                "onOpenDetail",
                "ProjectCover",
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
fn cards_metrics_badges_and_empty_states_are_shared_data_atoms() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/ProjectCard.tsx",
            vec![
                "Card,",
                "CardActionArea",
                "Chip",
                "IconButton",
                "HubProjectSummary",
                "ProjectCover",
                "chipSx",
            ],
        ),
        (
            "web/src/components/data/ProjectCover.tsx",
            vec![
                "coverById",
                "brandMark",
                "size?: \"card\" | \"thumb\"",
                "objectFit: \"cover\"",
                "pointerEvents: \"none\"",
            ],
        ),
        (
            "web/src/components/data/MetricCard.tsx",
            vec![
                "MetricCardProps",
                "tone?: \"neutral\" | \"accent\" | \"success\" | \"warning\" | \"error\"",
                "toneColor",
                "gridTemplateColumns",
                "hubTokens.radius.panel",
            ],
        ),
        (
            "web/src/components/data/StatusBadge.tsx",
            vec![
                "StatusTone",
                "toneMap",
                "PlayArrowIcon",
                "CheckCircleIcon",
                "WarningIcon",
                "ErrorIcon",
                "tone === \"running\"",
            ],
        ),
        (
            "web/src/components/data/EmptyStateBlock.tsx",
            vec![
                "EmptyStateBlockProps",
                "minHeight: 148",
                "border: `1px dashed",
                "placeItems: \"center\"",
                "hubTokens.colors.accent",
            ],
        ),
        (
            "web/src/components/data/HubPanel.tsx",
            vec![
                "Card",
                "component=\"section\"",
                "HubPanelProps",
                "action?: ReactNode",
                "Typography",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn interactive_data_rows_use_shared_row_components() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/QuickActions.tsx",
            vec![
                "ButtonBase",
                "HubQuickAction",
                "actionIcons",
                "gridTemplateColumns: \"36px minmax(0, 1fr) 24px\"",
                "ChevronRightIcon",
                "onAction?.(action)",
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
                "const hasSelectHandler = Boolean(onSelect);",
                "disabled={!hasSelectHandler}",
                "cursor: hasSelectHandler ? \"pointer\" : \"default\"",
                "onSelect?.(engine)",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn pages_compose_shared_data_components_instead_of_raw_data_material() {
    for (source_path, snippets) in [
        (
            "web/src/pages/ProjectsDashboard.tsx",
            vec![
                "EmptyStateBlock",
                "HubPanel",
                "ProjectCard",
                "ProjectTable",
                "QuickActions",
            ],
        ),
        (
            "web/src/pages/ProjectBrowserPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubPanel",
                "ProjectTable",
                "SourceEngineList",
            ],
        ),
        (
            "web/src/pages/ProjectDetailPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubList",
                "HubPanel",
                "HubTreeView",
                "ProjectCover",
                "ProjectDetailSidebar",
                "ProjectMetricsGrid",
                "StatusBadge",
            ],
        ),
        (
            "web/src/pages/CatalogPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubList",
                "HubPanel",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "StatusBadge",
            ],
        ),
        (
            "web/src/pages/SettingsPage.tsx",
            vec!["MetricCard", "SettingsSection"],
        ),
        (
            "web/src/pages/EditorPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubList",
                "HubPanel",
                "MetricCard",
                "SourceEngineList",
            ],
        ),
        (
            "web/src/pages/BuildsPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubList",
                "HubPanel",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
                "StatusBadge",
            ],
        ),
        (
            "web/src/pages/CloudPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubList",
                "HubPanel",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
            ],
        ),
        (
            "web/src/pages/TeamPage.tsx",
            vec![
                "EmptyStateBlock",
                "HubList",
                "HubPanel",
                "MetricCard",
                "QuickActions",
                "SourceEngineList",
            ],
        ),
        (
            "web/src/pages/WorkspacePage.tsx",
            vec![
                "HubList",
                "HubPanel",
                "HubTreeView",
                "MetricCard",
                "SourceEngineList",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
        assert_page_material_imports_do_not_include_data_primitives(source_path, &source);
        assert!(
            source.contains("gridTemplateColumns") || source.contains("@media (max-width:"),
            "{source_path} must keep responsive page composition around shared data components"
        );
    }
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    assert_not_contains_any(
        "web/src/pages/ProjectsDashboard.tsx",
        &dashboard,
        &["ButtonStatesPanel", "text.buttonStates"],
    );
}

#[test]
fn backend_types_feed_react_data_components() {
    let types = read_crate_file("web/src/types/hub.ts");
    assert_contains_all(
        "web/src/types/hub.ts",
        &types,
        &[
            "export interface HubProjectSummary",
            "export interface HubRecentProject",
            "export interface HubSourceEngineSummary",
            "export interface HubQuickAction",
            "export interface HubAssetItem",
            "export interface HubPluginItem",
            "export interface HubLearnItem",
            "export interface HubTeamSummary",
            "export type HubActionHistoryKind",
            "export interface HubActionHistoryItem",
            "kind: HubActionHistoryKind;",
            "projects: HubProjectSummary[]",
            "recentProjects: HubRecentProject[]",
            "browserProjects: HubRecentProject[]",
            "sourceEngines: HubSourceEngineSummary[]",
            "quickActions: HubQuickAction[]",
        ],
    );
}

#[test]
fn data_container_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_data_container_primitives_contract.rs");
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
            "React data-container contract must not keep old UI contract reference `{forbidden}`"
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
                "{source_path} must consume shared data components instead of importing raw Material data primitive `{forbidden}`"
            );
        }
    }
}
