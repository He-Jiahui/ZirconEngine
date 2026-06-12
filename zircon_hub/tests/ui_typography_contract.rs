//! Static contracts for the Hub React/MUI typography system.

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
            "{source_name} should contain typography contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete typography snippet {snippet:?}"
        );
    }
}

#[test]
fn mui_theme_and_global_css_define_shared_typography_scale() {
    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    let styles = read_crate_file("web/src/styles.css");

    assert_contains_all(
        "muiTheme.ts",
        &theme,
        &[
            "export const hubTheme = createTheme({",
            "typography: {",
            "fontFamily: 'Inter, Roboto, \"Segoe UI\", Arial, sans-serif'",
            "h4: {",
            "fontSize: 28",
            "lineHeight: 1.2",
            "fontWeight: 700",
            "h6: {",
            "fontSize: 16",
            "lineHeight: 1.25",
            "body1: {",
            "fontSize: 14",
            "body2: {",
            "fontSize: 13",
            "caption: {",
            "fontSize: 12",
            "button: {",
            "fontWeight: 500",
            "letterSpacing: 0",
            "textTransform: \"none\"",
            "MuiButton",
            "whiteSpace: \"nowrap\"",
            "MuiTooltip",
        ],
    );

    let letter_spacing_zero_count = theme.matches("letterSpacing: 0").count();
    assert!(
        letter_spacing_zero_count >= 6,
        "MUI typography variants should explicitly avoid negative or viewport-scaled letter spacing"
    );

    assert_contains_all(
        "styles.css",
        &styles,
        &[
            "font-family: Inter, Roboto, \"Segoe UI\", Arial, sans-serif;",
            "button,",
            "input,",
            "textarea,",
            "select",
            "font: inherit;",
        ],
    );
}

#[test]
fn data_components_use_mui_typography_variants_and_truncation() {
    let hub_panel = read_crate_file("web/src/components/data/HubPanel.tsx");
    let empty_state = read_crate_file("web/src/components/data/EmptyStateBlock.tsx");
    let hub_list = read_crate_file("web/src/components/data/HubList.tsx");
    let hub_tree = read_crate_file("web/src/components/data/HubTreeView.tsx");
    let metric_card = read_crate_file("web/src/components/data/MetricCard.tsx");
    let project_card = read_crate_file("web/src/components/data/ProjectCard.tsx");
    let project_table = read_crate_file("web/src/components/data/ProjectTable.tsx");
    let quick_actions = read_crate_file("web/src/components/data/QuickActions.tsx");
    let source_engine_list = read_crate_file("web/src/components/data/SourceEngineList.tsx");
    let status_badge = read_crate_file("web/src/components/data/StatusBadge.tsx");

    assert_contains_all(
        "HubPanel.tsx",
        &hub_panel,
        &[
            "import { Box, Card, Typography } from \"@mui/material\";",
            "Typography variant=\"h6\"",
            "color: hubTokens.colors.textSoft",
        ],
    );
    assert_contains_all(
        "EmptyStateBlock.tsx",
        &empty_state,
        &[
            "Typography variant=\"body2\"",
            "fontWeight: 700",
            "Typography variant=\"caption\"",
            "color: hubTokens.colors.textMuted",
        ],
    );
    assert_contains_all(
        "HubList.tsx",
        &hub_list,
        &[
            "ListItemText",
            "primary={<Typography variant=\"body2\" noWrap>{item.title}</Typography>}",
            "secondary={",
            "item.detail || item.secondaryDetail ? (",
            "<Box sx={{ minWidth: 0, display: \"grid\", gap: 0.15 }}>",
            "{item.detail ? <Typography variant=\"caption\" noWrap>{item.detail}</Typography> : null}",
            "{item.secondaryDetail ? (",
            "Typography variant=\"caption\" noWrap",
        ],
    );
    assert_contains_all(
        "HubTreeView.tsx",
        &hub_tree,
        &[
            "Typography variant=\"body2\" noWrap",
            "Typography variant=\"caption\" noWrap",
            "color: hubTokens.colors.textMuted",
        ],
    );
    assert_contains_all(
        "MetricCard.tsx",
        &metric_card,
        &[
            "Typography variant=\"caption\" noWrap",
            "Typography variant=\"h6\" noWrap",
            "color: toneColor[tone]",
        ],
    );
    assert_contains_all(
        "ProjectCard.tsx",
        &project_card,
        &[
            "Typography variant=\"h6\" noWrap",
            "Typography variant=\"body2\" color=\"text.secondary\" noWrap",
            "Typography variant=\"body2\" color=\"text.disabled\" noWrap",
        ],
    );
    assert_contains_all(
        "ProjectTable.tsx",
        &project_table,
        &[
            "Typography component=\"div\" variant=\"body2\" noWrap",
            "fontSize: 12",
            "fontWeight: 500",
            "Typography variant=\"body2\" noWrap",
        ],
    );
    assert_contains_all(
        "QuickActions.tsx",
        &quick_actions,
        &[
            "Typography variant=\"body2\" noWrap",
            "fontWeight: 700",
            "Typography variant=\"caption\" noWrap",
        ],
    );
    assert_contains_all(
        "SourceEngineList.tsx",
        &source_engine_list,
        &[
            "Typography variant=\"body2\"",
            "Typography variant=\"body2\" noWrap",
            "Typography variant=\"caption\" noWrap",
        ],
    );
    assert_contains_all(
        "StatusBadge.tsx",
        &status_badge,
        &["Typography variant=\"body2\"", "fontWeight: 600"],
    );
    for (name, source) in [
        ("HubPanel.tsx", hub_panel),
        ("EmptyStateBlock.tsx", empty_state),
        ("HubList.tsx", hub_list),
        ("HubTreeView.tsx", hub_tree),
        ("MetricCard.tsx", metric_card),
        ("ProjectCard.tsx", project_card),
        ("ProjectTable.tsx", project_table),
        ("QuickActions.tsx", quick_actions),
        ("SourceEngineList.tsx", source_engine_list),
        ("StatusBadge.tsx", status_badge),
    ] {
        assert_not_contains_any(name, &source, &["fontFamily", "letterSpacing"]);
    }
}

#[test]
fn input_and_overlay_components_use_body_caption_label_typography() {
    let checkbox = read_crate_file("web/src/components/inputs/HubCheckbox.tsx");
    let switch = read_crate_file("web/src/components/inputs/HubSwitch.tsx");
    let select = read_crate_file("web/src/components/inputs/HubSelect.tsx");
    let menu = read_crate_file("web/src/components/overlays/HubMenu.tsx");
    let source_engine_popover =
        read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");
    let user_menu_popover = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");

    for (name, source) in [("HubCheckbox.tsx", &checkbox), ("HubSwitch.tsx", &switch)] {
        assert_contains_all(
            name,
            source,
            &[
                "FormControlLabel",
                "Typography variant=\"body2\" noWrap",
                "Typography variant=\"caption\" noWrap",
                "color: isDisabled ? hubTokens.colors.textMuted : hubTokens.colors.text",
            ],
        );
    }
    assert_contains_all(
        "HubSelect.tsx",
        &select,
        &[
            "renderValue={(selected) => (",
            "Typography variant=\"body2\" color=\"text.secondary\"",
            "IconComponent={ExpandMoreIcon}",
            "MenuItem",
        ],
    );
    assert_contains_all(
        "HubMenu.tsx",
        &menu,
        &["MenuItem", "Typography variant=\"body2\""],
    );
    assert_contains_all(
        "SourceEnginePopover.tsx",
        &source_engine_popover,
        &[
            "Typography variant=\"caption\" sx={sectionLabelSx}",
            "Typography variant=\"body2\" noWrap",
            "Typography variant=\"caption\" noWrap",
            "fontWeight: 700",
            "textTransform: \"uppercase\"",
        ],
    );
    assert_contains_all(
        "UserMenuPopover.tsx",
        &user_menu_popover,
        &[
            "Typography variant=\"body2\" noWrap",
            "fontWeight: 700",
            "Typography variant=\"caption\" noWrap",
        ],
    );

    for (name, source) in [
        ("HubCheckbox.tsx", checkbox),
        ("HubSwitch.tsx", switch),
        ("HubSelect.tsx", select),
        ("HubMenu.tsx", menu),
        ("SourceEnginePopover.tsx", source_engine_popover),
        ("UserMenuPopover.tsx", user_menu_popover),
    ] {
        assert_not_contains_any(name, &source, &["fontFamily", "letterSpacing"]);
    }
}

#[test]
fn shell_components_use_reference_title_status_and_navigation_typography() {
    let top_bar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        "TopBar.tsx",
        &top_bar,
        &[
            "Typography variant=\"h6\" noWrap sx={{ textTransform: \"uppercase\", lineHeight: 1 }}",
            "Typography variant=\"body2\" noWrap color=\"text.secondary\"",
            "Typography variant=\"body2\" noWrap",
            "StatusBadge key={status.id}",
            "SourceEnginePopover",
            "UserMenuPopover",
        ],
    );
    assert_contains_all(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "Typography",
            "variant=\"body2\"",
            "fontWeight: selected ? 700 : 500",
            "Typography variant=\"caption\"",
            "{text.engineStatus}",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "const collapseLabel = collapsed ? text.expand : text.collapse;",
            "{collapseLabel}",
        ],
    );

    for (name, source) in [("TopBar.tsx", top_bar), ("NavigationDrawer.tsx", drawer)] {
        assert_not_contains_any(name, &source, &["fontFamily", "letterSpacing"]);
    }
}

#[test]
fn routed_pages_use_page_title_subtitle_typography_and_shared_components() {
    let pages = [
        "ProjectsDashboard.tsx",
        "ProjectBrowserPage.tsx",
        "ProjectDetailPage.tsx",
        "EditorPage.tsx",
        "BuildsPage.tsx",
        "CatalogPage.tsx",
        "CloudPage.tsx",
        "TeamPage.tsx",
        "SettingsPage.tsx",
        "WorkspacePage.tsx",
    ];

    for page in pages {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(
            page,
            &source,
            &[
                "Typography",
                "Typography variant=\"h4\"",
                "Typography variant=\"body1\" color=\"text.secondary\"",
            ],
        );
        assert_not_contains_any(page, &source, &["fontFamily", "letterSpacing"]);
    }

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &read_crate_file("web/src/pages/ProjectsDashboard.tsx"),
        &[
            "ProjectCard",
            "ProjectTable",
            "QuickActions",
            "HubPanel title={text.projectBrowser}",
            "HubPanel title={text.quickActions}",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &read_crate_file("web/src/pages/ProjectDetailPage.tsx"),
        &[
            "ProjectMetricsGrid",
            "HubList",
            "HubTreeView",
            "StatusBadge",
            "HubTabs",
        ],
    );
    assert_contains_all(
        "ProjectMetricsGrid.tsx",
        &read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx"),
        &["MetricCard"],
    );
    assert_contains_all(
        "SettingsPage.tsx",
        &read_crate_file("web/src/pages/SettingsPage.tsx"),
        &["MetricCard", "HubTabs", "SettingsSection"],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &read_crate_file("web/src/components/data/SettingsSection.tsx"),
        &[
            "HubCheckbox",
            "HubSwitch",
            "SourceEngineList",
            "HubTreeView",
        ],
    );
}

#[test]
fn typography_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_typography_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_typography_contract",
            "## Typography Contract Cutover",
            "React/MUI typography system",
            "web/src/theme/muiTheme.ts",
            "web/src/styles.css",
            "web/src/components/data",
            "web/src/components/inputs",
            "web/src/components/overlays",
            "web/src/components/shell",
            "web/src/pages",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_typography_contract.rs`",
            "React/MUI typography system",
            "shared MUI theme scale, global CSS font inheritance",
            "data/input/overlay/shell Typography variants",
            "routed page title/subtitle usage",
        ],
    );
}

#[test]
fn typography_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_typography_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_material_typography = format!("Material{}", "Typography");

    assert_contains_all(
        "ui_typography_contract.rs",
        &contract,
        &[
            "web/src/theme/muiTheme.ts",
            "web/src/styles.css",
            "HubPanel.tsx",
            "EmptyStateBlock.tsx",
            "HubList.tsx",
            "MetricCard.tsx",
            "ProjectCard.tsx",
            "TopBar.tsx",
            "NavigationDrawer.tsx",
            "web/src/pages",
        ],
    );
    assert_not_contains_any(
        "ui_typography_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_material_typography.as_str(),
        ],
    );
}
