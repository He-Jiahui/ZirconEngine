//! Static contracts for React + Material UI panel composition.

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
            "{source_name} should contain panel contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete or page-local panel snippet {snippet:?}"
        );
    }
}

#[test]
fn data_panel_primitives_are_reexported_and_own_shared_panel_chrome() {
    let index = read_crate_file("web/src/components/data/index.ts");
    let panel = read_crate_file("web/src/components/data/HubPanel.tsx");
    let empty_state = read_crate_file("web/src/components/data/EmptyStateBlock.tsx");
    let metric = read_crate_file("web/src/components/data/MetricCard.tsx");
    let project_card = read_crate_file("web/src/components/data/ProjectCard.tsx");

    assert_contains_all(
        "components/data/index.ts",
        &index,
        &[
            "HubPanel",
            "EmptyStateBlock",
            "MetricCard",
            "HubList",
            "HubTreeView",
            "ProjectTable",
            "QuickActions",
            "SourceEngineList",
            "StatusBadge",
            "ProjectCard",
        ],
    );
    assert_contains_all(
        "HubPanel.tsx",
        &panel,
        &[
            "PropsWithChildren",
            "ReactNode",
            "Card",
            "component=\"section\"",
            "HubPanelProps",
            "action?: ReactNode",
            "p: 2",
            "minWidth: 0",
            "overflow: \"hidden\"",
            "Typography variant=\"h6\"",
            "{children}",
        ],
    );
    assert_contains_all(
        "EmptyStateBlock.tsx",
        &empty_state,
        &[
            "EmptyStateBlockProps",
            "minHeight: 148",
            "display: \"grid\"",
            "placeItems: \"center\"",
            "border: `1px dashed",
            "Typography variant=\"body2\"",
            "Typography variant=\"caption\"",
        ],
    );
    assert_contains_all(
        "MetricCard.tsx",
        &metric,
        &[
            "MetricCardProps",
            "tone?: \"neutral\" | \"accent\" | \"success\" | \"warning\" | \"error\"",
            "gridTemplateColumns: icon ? \"34px minmax(0, 1fr)\" : \"1fr\"",
            "hubTokens.radius.panel",
            "Typography variant=\"h6\" noWrap",
        ],
    );
    assert_contains_all(
        "ProjectCard.tsx",
        &project_card,
        &[
            "CardActionArea",
            "height: 251",
            "ProjectCover",
            "selected ? \"rgba(45,212,207,0.44)\"",
            "transform: \"translateY(-1px)\"",
        ],
    );
}

#[test]
fn pages_route_repeated_panel_shells_through_hub_panel() {
    for (page, minimum_panel_count) in [
        ("BuildsPage.tsx", 7),
        ("CatalogPage.tsx", 5),
        ("CloudPage.tsx", 8),
        ("EditorPage.tsx", 7),
        ("ProjectBrowserPage.tsx", 3),
        ("ProjectDetailPage.tsx", 6),
        ("ProjectsDashboard.tsx", 3),
        ("SettingsPage.tsx", 7),
        ("TeamPage.tsx", 7),
        ("WorkspacePage.tsx", 7),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        let hub_panel_count = source.matches("<HubPanel").count();
        assert!(
            hub_panel_count >= minimum_panel_count,
            "{page} should compose repeated panel surfaces through HubPanel; expected at least {minimum_panel_count}, found {hub_panel_count}"
        );
        assert_contains_all(
            page,
            &source,
            &[
                "HubPanel",
                "display: \"grid\"",
                "gridTemplateColumns",
                "@media (max-width:",
            ],
        );
    }
}

#[test]
fn project_pages_keep_dashboard_browser_and_detail_panels_componentized() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "ProjectBrowserPage",
            "ProjectDetailPage",
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "ProjectCard",
            "ProjectTable",
            "HubPanel title={text.projectBrowser}",
            "title={text.recentProjects}",
            "HubPanel title={text.quickActions}",
            "HubDialog",
            "gridTemplateColumns: \"repeat(4, minmax(220px, 296px))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
        ],
    );
    assert_not_contains_any(
        "ProjectsDashboard.tsx",
        &dashboard,
        &["ButtonStatesPanel", "text.buttonStates"],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "ProjectTable",
            "QuickActions",
            "SourceEngineList",
            "HubPanel title={text.allProjects}",
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(320px, 0.42fr)\"",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "ProjectCover",
            "MetricCard",
            "HubList",
            "HubTreeView",
            "QuickActions",
            "SourceEngineList",
            "HubPanel title={text.projectOverview}",
            "HubPanel title={text.projectTree}",
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.4fr)\"",
        ],
    );
}

#[test]
fn workspace_pages_share_metric_rows_side_panels_and_empty_states() {
    for page in [
        "BuildsPage.tsx",
        "CatalogPage.tsx",
        "CloudPage.tsx",
        "EditorPage.tsx",
        "TeamPage.tsx",
        "WorkspacePage.tsx",
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(
            page,
            &source,
            &[
                "MetricCard",
                "HubList",
                "HubPanel",
                "HubTreeView",
                "QuickActions",
                "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            ],
        );
        assert!(
            source.contains("gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"")
                || source.contains("gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\""),
            "{page} should use the shared main/sidebar panel grid proportions"
        );
        assert!(
            source.contains("EmptyStateBlock") || page == "WorkspacePage.tsx",
            "{page} should use shared EmptyStateBlock for empty panel bodies unless it is the fallback workspace sample"
        );
    }

    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "MetricCard",
            "HubList",
            "HubPanel",
            "HubTreeView",
            "SourceEngineList",
            "StatusBadge",
            "HubCheckbox",
            "HubSwitch",
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.42fr)\"",
        ],
    );
}

#[test]
fn pages_do_not_import_raw_material_panel_or_data_container_primitives() {
    for page in [
        "BuildsPage.tsx",
        "CatalogPage.tsx",
        "CloudPage.tsx",
        "EditorPage.tsx",
        "ProjectBrowserPage.tsx",
        "ProjectDetailPage.tsx",
        "ProjectsDashboard.tsx",
        "SettingsPage.tsx",
        "TeamPage.tsx",
        "WorkspacePage.tsx",
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        for import_line in source.lines().filter(|line| line.contains("@mui/material")) {
            assert_not_contains_any(
                page,
                import_line,
                &[
                    "Card",
                    "Paper",
                    "Table",
                    "TableBody",
                    "TableCell",
                    "ListItemButton",
                    "ListItemText",
                    "Drawer",
                ],
            );
        }
        assert_not_contains_any(
            page,
            &source,
            &[
                "<Card",
                "<Paper",
                "<Table",
                "<TableBody",
                "<ListItemButton",
                "<Drawer",
            ],
        );
    }
}

#[test]
fn panel_slot_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_panel_slot_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_panel_slot_contract",
            "## Panel Slot Contract Cutover",
            "React/MUI panel composition",
            "web/src/components/data/HubPanel.tsx",
            "web/src/components/data/EmptyStateBlock.tsx",
            "web/src/components/data/MetricCard.tsx",
            "web/src/pages",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_panel_slot_contract.rs`",
            "React/MUI panel composition",
            "shared HubPanel shell",
            "responsive page grids",
            "pages do not import raw Material panel containers",
        ],
    );
}

#[test]
fn panel_slot_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_panel_slot_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");

    assert_contains_all(
        "ui_panel_slot_contract.rs",
        &contract,
        &[
            "web/src/components/data/HubPanel.tsx",
            "web/src/components/data/EmptyStateBlock.tsx",
            "web/src/components/data/MetricCard.tsx",
            "ProjectsDashboard.tsx",
            "ProjectBrowserPage.tsx",
            "ProjectDetailPage.tsx",
            "SettingsPage.tsx",
            "web/src/pages",
        ],
    );
    assert_not_contains_any(
        "ui_panel_slot_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
        ],
    );
}
