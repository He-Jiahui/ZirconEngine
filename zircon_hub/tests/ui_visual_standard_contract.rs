//! Static contracts for the Hub React/MUI visual standard and reference artifacts.

use std::{
    fs,
    path::{Path, PathBuf},
};

const FINAL_VISUAL_ARTIFACTS: &[&str] = &[
    "hub.png",
    "hub-editor.png",
    "hub-builds.png",
    "hub-assets.png",
    "hub-plugins.png",
    "hub-cloud.png",
    "hub-team.png",
    "hub-learn.png",
    "hub-settings.png",
    "hub-projects-new.png",
    "hub-projects-browser.png",
    "hub-projects-detail.png",
    "hub-projects-browser-filter-menu.png",
    "hub-projects-browser-sort-menu.png",
    "hub-projects-detail-delete-confirm.png",
    "hub-source-engine-popup.png",
    "hub-user-menu.png",
    "hub-state-empty.png",
    "hub-state-loading.png",
    "hub-state-error.png",
];

const AI_DRAFT_ARTIFACTS: &[&str] = &[
    "hub-editor.png",
    "hub-builds.png",
    "hub-assets.png",
    "hub-plugins.png",
    "hub-cloud.png",
    "hub-team.png",
    "hub-learn.png",
    "hub-settings.png",
    "hub-projects-new.png",
    "hub-projects-browser.png",
    "hub-projects-detail.png",
    "hub-projects-browser-filter-menu.png",
    "hub-projects-browser-sort-menu.png",
    "hub-projects-detail-delete-confirm.png",
    "hub-source-engine-popup.png",
    "hub-user-menu.png",
    "hub-state-empty.png",
    "hub-state-loading.png",
    "hub-state-error.png",
];

const SUPPLEMENTAL_DESIGN_ARTIFACTS: &[&str] = &[
    "hub-design-structure-layout.png",
    "hub-design-structure-supplement.png",
    "hub-design-functional-details.png",
];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub must live under the repository root")
        .to_path_buf()
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

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read repository file {path}: {error}");
        }),
    )
}

fn assert_contains_all(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{label} must contain visual-standard snippet: {snippet}"
        );
    }
}

fn assert_not_contains_any(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{label} must not contain obsolete visual-standard snippet: {snippet}"
        );
    }
}

fn collect_files_with_extension(root: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to scan visual-standard source directory {}: {error}",
            root.display()
        );
    }) {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read visual-standard source entry: {error}"))
            .path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

fn contains_hex_color_literal(line: &str) -> bool {
    line.as_bytes()
        .windows(4)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}

#[test]
fn component_and_page_styles_do_not_bypass_visual_tokens() {
    let mut files = Vec::new();
    collect_files_with_extension(&crate_dir().join("web/src/components"), "tsx", &mut files);
    collect_files_with_extension(&crate_dir().join("web/src/pages"), "tsx", &mut files);

    for file in files {
        let source = normalize_newlines(fs::read_to_string(&file).unwrap_or_else(|error| {
            panic!(
                "failed to read visual-standard source file {}: {error}",
                file.display()
            );
        }));
        for (index, line) in source.lines().enumerate() {
            assert!(
                !contains_hex_color_literal(line),
                "{}:{} must use hubTokens instead of component-local hex colors: {line}",
                file.display(),
                index + 1
            );
            assert!(
                !line.contains("borderRadius: 999"),
                "{}:{} must use hubTokens.radius.pill instead of component-local pill radius",
                file.display(),
                index + 1
            );
        }
    }
}

#[test]
fn react_tokens_global_css_and_mui_theme_define_reference_visual_standard() {
    let tokens = read_crate_file("web/src/theme/tokens.ts");
    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    let styles = read_crate_file("web/src/styles.css");

    assert_contains_all(
        &tokens,
        &[
            "export const hubTokens =",
            "width: 1568",
            "height: 1003",
            "topBarHeight: 73",
            "sidebarWidth: 222",
            "sidebarCollapsedWidth: 78",
            "pagePaddingX: 30",
            "pagePaddingY: 28",
            "compact: 7",
            "panel: 8",
            "card: 8",
            "pill: 999",
            "background: \"#111212\"",
            "chrome: \"#151515\"",
            "panel: \"#202020\"",
            "panelLow: \"#1c1c1c\"",
            "panelHover: \"#292929\"",
            "line: \"rgba(255,255,255,0.10)\"",
            "lineStrong: \"rgba(255,255,255,0.16)\"",
            "text: \"#eeeeee\"",
            "textOnAccent: \"#eefefe\"",
            "textOnPrimary: \"#071515\"",
            "textSoft: \"#b9b9b9\"",
            "textMuted: \"#8d8d8d\"",
            "dangerText: \"#ffd8d5\"",
            "accent: \"#21d5cf\"",
            "accentDim: \"rgba(20, 121, 119, 0.72)\"",
            "success: \"#77d77a\"",
            "warning: \"#ffc24d\"",
            "error: \"#ef655e\"",
            "avatar: \"#4b4f52\"",
            "coverBackdrop: \"#141414\"",
            "tooltip: \"#242424\"",
            "gradients:",
            "window:",
            "panel: \"inset 0 0 0 1px rgba(255,255,255,0.04), 0 18px 42px rgba(0,0,0,0.28)\"",
            "accent: \"0 0 14px rgba(33,213,207,0.2)\"",
            "as const",
        ],
        "web/src/theme/tokens.ts",
    );
    assert_contains_all(
        &theme,
        &[
            "export const hubTheme = createTheme({",
            "mode: \"dark\"",
            "default: hubTokens.colors.background",
            "paper: hubTokens.colors.panel",
            "main: hubTokens.colors.accent",
            "main: hubTokens.colors.success",
            "main: hubTokens.colors.warning",
            "main: hubTokens.colors.error",
            "primary: hubTokens.colors.text",
            "secondary: hubTokens.colors.textSoft",
            "disabled: hubTokens.colors.textMuted",
            "divider: hubTokens.colors.line",
            "borderRadius: hubTokens.radius.compact",
            "fontFamily: 'Inter, Roboto, \"Segoe UI\", Arial, sans-serif'",
            "letterSpacing: 0",
            "textTransform: \"none\"",
            "MuiButton:",
            "height: 42",
            "whiteSpace: \"nowrap\"",
            "MuiCard:",
            "backgroundImage: \"none\"",
            "backgroundColor: hubTokens.colors.panel",
            "boxShadow: hubTokens.shadows.panel",
            "MuiIconButton:",
            "MuiMenu:",
            "MuiOutlinedInput:",
            "MuiSelect:",
            "MuiTooltip:",
        ],
        "web/src/theme/muiTheme.ts",
    );
    assert_contains_all(
        &styles,
        &[
            "html,",
            "body,",
            "#root",
            "width: 100%;",
            "min-width: 0;",
            "min-height: 100%;",
            "margin: 0;",
            "overflow: hidden;",
            "background: #090a0a;",
            "font-family: Inter, Roboto, \"Segoe UI\", Arial, sans-serif;",
            "button,",
            "input,",
            "textarea,",
            "select",
            "font: inherit;",
        ],
        "web/src/styles.css",
    );
}

#[test]
fn shell_chrome_drawer_topbar_and_popups_use_visual_tokens() {
    let window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");
    let popover = read_crate_file("web/src/components/overlays/HubPopover.tsx");
    let dialog = read_crate_file("web/src/components/overlays/HubDialog.tsx");
    let source_popup = read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");
    let user_popup = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");

    assert_contains_all(
        &window,
        &[
            "width: \"100vw\"",
            "height: \"100vh\"",
            "overflow: \"hidden\"",
            "color: hubTokens.colors.text",
            "background: hubTokens.gradients.window",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "borderRadius: \"10px\"",
            "height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`",
            "backgroundColor: \"rgba(17,17,17,0.55)\"",
        ],
        "HubWindow",
    );
    assert_contains_all(
        &topbar,
        &[
            "height: hubTokens.window.topBarHeight",
            "gridTemplateColumns: \"222px minmax(0, 1fr) auto\"",
            "borderBottom: `1px solid ${hubTokens.colors.line}`",
            "backgroundColor: \"rgba(17,17,17,0.96)\"",
            "gridTemplateColumns: \"78px minmax(0, 1fr) auto\"",
            "src={brandMark}",
            "textTransform: \"uppercase\"",
            "border: `1px solid ${engineAnchor ? \"rgba(45,212,207,0.48)\" : hubTokens.colors.lineStrong}`",
            "backgroundColor: engineAnchor ? \"rgba(18,82,80,0.38)\"",
            "state.taskStatus.map((status) =>",
            "Avatar sx={{ width: 36, height: 36, bgcolor: hubTokens.colors.avatar, fontSize: 14 }}",
            "SourceEnginePopover",
            "UserMenuPopover",
        ],
        "TopBar",
    );
    assert_contains_all(
        &drawer,
        &[
            "width: drawerWidth",
            "backgroundColor: \"rgba(16,16,16,0.96)\"",
            "borderRight: `1px solid ${hubTokens.colors.line}`",
            "transition: \"width 160ms ease\"",
            "borderRadius: `${hubTokens.radius.panel}px`",
            "backgroundColor: selected ? \"rgba(15,99,96,0.56)\" : \"transparent\"",
            "backgroundColor: \"rgba(32,32,32,0.62)\"",
            "const statusColor = activeEngine ? hubTokens.colors.success : hubTokens.colors.warning;",
            "backgroundColor: statusColor",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "disabled",
            "\"&.Mui-disabled\"",
            "const collapseLabel = collapsed ? text.expand : text.collapse;",
            "{collapseLabel}",
            "width: drawerWidth",
        ],
        "NavigationDrawer",
    );
    assert_contains_all(
        &popover,
        &[
            "width = 340",
            "maxWidth: \"calc(100vw - 32px)\"",
            "color: hubTokens.colors.text",
            "backgroundColor: \"rgba(25,29,29,0.98)\"",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "borderRadius: `${hubTokens.radius.panel}px`",
            "boxShadow: \"0 24px 60px rgba(0,0,0,0.46), 0 0 0 1px rgba(45,212,207,0.08)\"",
        ],
        "HubPopover",
    );
    assert_contains_all(
        &dialog,
        &[
            "Dialog",
            "DialogTitle",
            "DialogContent",
            "DialogActions",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "backgroundImage: \"none\"",
            "backgroundColor: \"rgba(28,28,28,0.98)\"",
        ],
        "HubDialog",
    );
    assert_contains_all(
        &source_popup,
        &[
            "HubPopover anchorEl={anchorEl} open={open} width={388}",
            "{text.activeEngine}",
            "{text.readyFallback}",
            "{text.localDefaults}",
            "gridTemplateColumns: \"34px minmax(0, 1fr) auto\"",
            "backgroundColor: active ? \"rgba(18,82,80,0.5)\"",
            "StatusBadge label={activeLabel} tone=\"success\"",
            "{text.manageEngines}",
        ],
        "SourceEnginePopover",
    );
    assert_contains_all(
        &user_popup,
        &[
            "HubPopover anchorEl={anchorEl} open={open} width={284} align=\"right\"",
            "gridTemplateColumns: \"42px minmax(0, 1fr)\"",
            "Avatar sx={{ width: 38, height: 38, bgcolor: hubTokens.colors.avatar, fontSize: 14 }}",
            "borderColor: hubTokens.colors.line",
            "color: isDisabled ? hubTokens.colors.textMuted : danger ? hubTokens.colors.error : hubTokens.colors.text",
            "backgroundColor: isDisabled ? \"transparent\" : danger ? \"rgba(105,31,29,0.24)\" : \"rgba(255,255,255,0.055)\"",
            "\"&.Mui-disabled\"",
        ],
        "UserMenuPopover",
    );
}

#[test]
fn shared_inputs_and_data_components_preserve_reference_density_and_states() {
    let button = read_crate_file("web/src/components/inputs/HubButton.tsx");
    let icon_button = read_crate_file("web/src/components/inputs/HubIconButton.tsx");
    let search = read_crate_file("web/src/components/inputs/HubSearchField.tsx");
    let select = read_crate_file("web/src/components/inputs/HubSelect.tsx");
    let toggle = read_crate_file("web/src/components/inputs/HubToggle.tsx");
    let panel = read_crate_file("web/src/components/data/HubPanel.tsx");
    let card = read_crate_file("web/src/components/data/ProjectCard.tsx");
    let cover = read_crate_file("web/src/components/data/ProjectCover.tsx");
    let badge = read_crate_file("web/src/components/data/StatusBadge.tsx");
    let metric = read_crate_file("web/src/components/data/MetricCard.tsx");
    let empty = read_crate_file("web/src/components/data/EmptyStateBlock.tsx");
    let table = read_crate_file("web/src/components/data/ProjectTable.tsx");
    let list = read_crate_file("web/src/components/data/HubList.tsx");
    let data_index = read_crate_file("web/src/components/data/index.ts");

    assert_contains_all(
        &button,
        &[
            "export type HubButtonTone = \"primary\" | \"secondary\" | \"tertiary\" | \"danger\";",
            "color: hubTokens.colors.textOnAccent",
            "backgroundColor: hubTokens.colors.accentDim",
            "borderColor: \"rgba(45, 212, 207, 0.48)\"",
            "backgroundColor: \"rgba(32,32,32,0.82)\"",
            "backgroundColor: hubTokens.colors.panelHover",
            "color: hubTokens.colors.accent",
            "color: hubTokens.colors.dangerText",
            "backgroundColor: \"rgba(120,25,25,0.54)\"",
            "variant=\"contained\"",
            "border: \"1px solid\"",
            "px: 2.5",
        ],
        "HubButton",
    );
    assert_contains_all(
        &icon_button,
        &[
            "Tooltip title={tooltip ?? label}",
            "width: 50",
            "height: 42",
            "color: selected ? hubTokens.colors.textOnAccent : hubTokens.colors.textSoft",
            "backgroundColor: selected ? \"rgba(9,94,91,0.56)\"",
            "border: `1px solid ${selected ? \"rgba(45,212,207,0.48)\" : hubTokens.colors.lineStrong}`",
            "hubTokens.colors.panelHover",
            "\"&.Mui-disabled\"",
        ],
        "HubIconButton",
    );
    assert_contains_all(
        &search,
        &[
            "width: compact ? 260 : 307",
            "height: compact ? 36 : 47",
            "borderColor: compact ? hubTokens.colors.lineStrong : \"rgba(45,212,207,0.92)\"",
            "boxShadow: compact ? \"none\" : hubTokens.shadows.accent",
            "color: hubTokens.colors.textMuted",
            "opacity: 1",
        ],
        "HubSearchField",
    );
    assert_contains_all(
        &select,
        &[
            "minWidth = 183",
            "height: 42",
            "color: hubTokens.colors.textSoft",
            "display: \"flex\"",
            "alignItems: \"center\"",
            "MenuItem",
        ],
        "HubSelect",
    );
    assert_contains_all(
        &toggle,
        &[
            "ToggleButtonGroup",
            "width: 50",
            "height: 42",
            "borderRadius: `${hubTokens.radius.compact}px !important`",
            "backgroundColor: \"rgba(31,31,31,0.72)\"",
            "\"&.Mui-selected\"",
            "backgroundColor: \"rgba(9,94,91,0.56)\"",
        ],
        "HubToggle",
    );
    assert_contains_all(
        &panel,
        &[
            "Card",
            "component=\"section\"",
            "p: 2",
            "overflow: \"hidden\"",
            "Typography variant=\"h6\"",
            "color: hubTokens.colors.textSoft",
        ],
        "HubPanel",
    );
    assert_contains_all(
        &card,
        &[
            "height: 251",
            "borderColor: selected ? \"rgba(45,212,207,0.44)\" : hubTokens.colors.lineStrong",
            "transition: \"border-color 140ms ease, transform 140ms ease\"",
            "transform: \"translateY(-1px)\"",
            "height: 112",
            "ProjectCover",
            "Chip label={project.engineVersion}",
            "Chip label={project.platform}",
        ],
        "ProjectCard",
    );
    assert_contains_all(
        &cover,
        &[
            "width: thumb ? 30 : \"100%\"",
            "height: thumb ? 30 : \"100%\"",
            "backgroundColor: hubTokens.colors.coverBackdrop",
            "objectFit: \"cover\"",
            "filter: \"saturate(0.98) contrast(0.98) brightness(0.98)\"",
            "linear-gradient(90deg, rgba(255,255,255,0.035)",
            "src={brandMark}",
            "backgroundColor: \"rgba(10,20,22,0.72)\"",
        ],
        "ProjectCover",
    );
    assert_contains_all(
        &badge,
        &[
            "const toneMap",
            "running:",
            "success:",
            "warning:",
            "error:",
            "neutral:",
            "height: 36",
            "minWidth: 112",
            "backgroundColor: toneStyle.background",
            "border: `1px solid ${toneStyle.border}`",
            "tone === \"running\"",
        ],
        "StatusBadge",
    );
    assert_contains_all(
        &metric,
        &[
            "minHeight: 86",
            "gridTemplateColumns: icon ? \"34px minmax(0, 1fr)\" : \"1fr\"",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "backgroundColor: \"rgba(32,32,32,0.62)\"",
            "Typography variant=\"caption\"",
            "Typography variant=\"h6\"",
        ],
        "MetricCard",
    );
    assert_contains_all(
        &empty,
        &[
            "minHeight: 148",
            "placeItems: \"center\"",
            "border: `1px dashed ${hubTokens.colors.lineStrong}`",
            "backgroundColor: \"rgba(28,28,28,0.42)\"",
            "textAlign: \"center\"",
        ],
        "EmptyStateBlock",
    );
    assert_contains_all(
        &table,
        &[
            "Table size=\"small\"",
            "tableLayout: \"fixed\"",
            "height: 36",
            "borderColor: \"rgba(255,255,255,0.075)\"",
            "backgroundColor: \"rgba(18,82,80,0.32)\"",
            "ProjectCover coverId={project.coverId} size=\"thumb\"",
            "fontSize: 12",
        ],
        "ProjectTable",
    );
    assert_contains_all(
        &list,
        &[
            "List dense",
            "gap: 0.7",
            "minHeight: item.secondaryDetail ? 64 : 48",
            "borderRadius: `${hubTokens.radius.compact}px`",
            "backgroundColor: item.selected ? \"rgba(18,82,80,0.38)\" : \"rgba(32,32,32,0.54)\"",
            "Typography variant=\"body2\"",
            "Typography variant=\"caption\"",
        ],
        "HubList",
    );
    assert_not_contains_any(&data_index, &["ButtonStatesPanel"], "components/data index");
}

#[test]
fn pages_keep_reference_responsive_density_and_state_surfaces() {
    for (page, snippets) in [
        (
            "ProjectsDashboard",
            &[
                "height: \"100%\"",
                "px: `${hubTokens.window.pagePaddingX}px`",
                "py: `${hubTokens.window.pagePaddingY}px`",
                "ProjectsToolbar",
                "ProjectCardRail",
                "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
                "EmptyStateBlock",
                "CreateProjectDialog",
            ][..],
        ),
        (
            "ProjectBrowserPage",
            &[
                "gridTemplateColumns: \"minmax(280px, 420px) 1fr auto auto auto\"",
                "gridTemplateColumns: \"minmax(0, 1fr) minmax(320px, 0.42fr)\"",
                "HubStatusBanner",
                "EmptyStateBlock title={text.noProjectsFound}",
                "SourceEngineList",
            ][..],
        ),
        (
            "ProjectDetailPage",
            &[
                "ProjectMetricsGrid",
                "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.4fr)\"",
                "ProjectCover",
                "StatusBadge",
                "EmptyStateBlock title={text.noProjectSelected}",
            ][..],
        ),
        (
            "SettingsPage",
            &[
                "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
                "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.42fr)\"",
                "MetricCard",
                "SettingsSection",
            ][..],
        ),
        (
            "BuildsPage",
            &[
                "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
                "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
                "LinearProgress",
                "EmptyStateBlock title={text.noBuildHistory}",
                "BuildActionDetail",
            ][..],
        ),
        (
            "CatalogPage",
            &[
                "width: 320",
                "HubSearchField",
                "MetricCard",
                "StatusBadge",
                "EmptyStateBlock title={text.noEntriesFound}",
                "HubTreeView",
            ][..],
        ),
        (
            "CloudPage",
            &[
                "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
                "HubSwitch",
                "HubCheckbox",
                "EmptyStateBlock title={text.noPackagesRecorded}",
                "StatusBadge label={state.taskSummary.label}",
            ][..],
        ),
        (
            "TeamPage",
            &[
                "StatusBadge label={state.taskSummary.label}",
                "MetricCard",
                "HubTreeView",
                "EmptyStateBlock title={text.noTeamMembersFound}",
                "ActionDetail",
            ][..],
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}.tsx"));
        assert_contains_all(&source, snippets, page);
        assert!(
            source.contains("@media (max-width:"),
            "{page} must keep responsive visual constraints"
        );
    }

    assert_contains_all(
        &read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx"),
        &[
            "gridTemplateColumns: \"minmax(260px, 307px) 1fr auto auto auto\"",
            "gridTemplateColumns: \"minmax(240px, 1fr) auto auto\"",
            "gridTemplateColumns: \"1fr\"",
        ],
        "ProjectsToolbar",
    );
    assert_contains_all(
        &read_crate_file("web/src/components/data/ProjectCardRail.tsx"),
        &["gridTemplateColumns: \"repeat(auto-fill, minmax(clamp(220px, 22vw, 296px), 1fr))\""],
        "ProjectCardRail",
    );
    assert_contains_all(
        &read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx"),
        &[
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
            "MetricCard",
        ],
        "ProjectMetricsGrid",
    );
    assert_contains_all(
        &read_crate_file("web/src/components/data/SettingsSection.tsx"),
        &[
            "LinearProgress",
            "StatusBadge label={draftSettings.health.label} tone={draftSettings.health.tone}",
            "HubComboBox",
            "HubTreeView",
        ],
        "SettingsSection",
    );
}

#[test]
fn runtime_visual_assets_are_react_assets_not_reference_screenshots() {
    let data = read_crate_file("web/src/data/hubData.ts");

    assert_contains_all(
        &data,
        &[
            "import brandMarkAsset from \"../../../assets/brand/zircon-mark.svg\";",
            "import elysiumCover from \"../../../assets/covers/reference/project-elysium.png\";",
            "import neonCover from \"../../../assets/covers/reference/project-neon-streets.png\";",
            "import sandsCover from \"../../../assets/covers/reference/project-sands-of-time.png\";",
            "import stellarCover from \"../../../assets/covers/reference/project-stellar-outpost.png\";",
            "import woodsCover from \"../../../assets/covers/reference/project-whispering-woods.png\";",
            "export const brandMark = brandMarkAsset;",
            "export const coverById: Record<string, string>",
        ],
        "web/src/data/hubData.ts",
    );
    assert_not_contains_any(
        &data,
        &[
            "docs/ui-and-layout/hub.png",
            "docs/ui-and-layout/hub-ai-drafts",
            "hub-web-reference-1568x1003.png",
            "hub-state-loading.png",
            "hub-state-error.png",
        ],
        "web/src/data/hubData.ts",
    );
}

#[test]
fn visual_reference_artifacts_manifest_and_web_reference_remain_available() {
    let manifest = read_repo_file("docs/ui-and-layout/hub-ai-reference-manifest.json");
    let registry = read_repo_file("docs/ui-and-layout/hub-web-reference/page-registry.mjs");
    let exporter = read_repo_file("docs/ui-and-layout/hub-web-reference/export-pages.mjs");
    let visual_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-visuals.mjs");
    let responsive_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-responsive.mjs");
    let interaction_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-interactions.mjs");
    let ui_index = read_repo_file("docs/ui-and-layout/index.md");

    assert_contains_all(
        &manifest,
        &[
            "\"source_reference\": \"docs/ui-and-layout/hub.png\"",
            "\"ai_draft_root\": \"docs/ui-and-layout/hub-ai-drafts\"",
            "\"final_source\": \"docs/ui-and-layout/hub-web-reference/index.html\"",
            "\"width\": 1568",
            "\"height\": 1003",
            "\"draft_kind\": \"overall-interaction-structure-layout\"",
            "\"draft_usage\": \"Overall interaction structure layout drafts for review; local functional-content callouts are secondary; not acceptance evidence.\"",
            "\"export_command\": \"node docs/ui-and-layout/hub-web-reference/export-pages.mjs\"",
            "\"visual_validation\": \"node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs\"",
            "\"interaction_validation\": \"node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs\"",
        ],
        "hub-ai-reference-manifest.json",
    );
    assert_contains_all(
        &registry,
        &[
            "export const CANVAS_WIDTH = 1568;",
            "export const CANVAS_HEIGHT = 1003;",
            "export const DASHBOARD_PAGE_ID = \"projects-dashboard\";",
            "export const DASHBOARD_CAPTURE_NAME = \"hub-web-reference-1568x1003.png\";",
            "export const EXPORTS_LIST = [",
        ],
        "hub-web-reference/page-registry.mjs",
    );
    assert_contains_all(
        &exporter,
        &[
            "EXPORTS_LIST",
            "selectedExports",
            "captureAll",
            "capture(pageId, target)",
        ],
        "hub-web-reference/export-pages.mjs",
    );
    assert_contains_all(
        &visual_validator,
        &[
            "validateExport",
            "validateRootPngInventory",
            "validateAiManifest",
            "decodePng",
            "CANVAS_WIDTH",
            "CANVAS_HEIGHT",
        ],
        "hub-web-reference/validate-visuals.mjs",
    );
    assert_contains_all(
        &responsive_validator,
        &[
            "const viewports = [",
            "[\"wide\", 1600, 1024]",
            "[\"desktop\", 1280, 900]",
            "[\"compact\", 1024, 720]",
            "responsiveAuditExpression(width, height)",
        ],
        "hub-web-reference/validate-responsive.mjs",
    );
    assert_contains_all(
        &interaction_validator,
        &[
            "knownPageIds",
            "[data-route]",
            "button.engine-select",
            ".quick-row:first-child",
        ],
        "hub-web-reference/validate-interactions.mjs",
    );
    assert_contains_all(
        &ui_index,
        &[
            "Hub Visual Artifact Matrix",
            "`hub.png` remains the Projects Dashboard pixel reference",
            "`docs/ui-and-layout/hub-ai-reference-manifest.json`",
            "`docs/ui-and-layout/hub-web-reference/export-pages.mjs`",
            "target/hub-visual-check-final/hub-projects-dashboard.png",
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-dashboard.png",
        ],
        "docs/ui-and-layout/index.md",
    );

    for artifact in FINAL_VISUAL_ARTIFACTS {
        let path = repo_dir().join("docs/ui-and-layout").join(artifact);
        assert_png_canvas(&path, (1568, 1003), "final visual reference");

        if *artifact != "hub.png" {
            let page_id = artifact.trim_end_matches(".png");
            assert!(
                manifest.contains(&format!("\"page_id\": \"{page_id}\"")),
                "manifest must include page id {page_id}"
            );
            assert!(
                manifest.contains(&format!("\"output\": \"{artifact}\"")),
                "manifest must include output {artifact}"
            );
            assert!(
                registry.contains(&format!("\"{artifact}\"")),
                "web reference registry must include output {artifact}"
            );
        }
        assert!(
            ui_index.contains(&format!("`{artifact}`")),
            "docs/ui-and-layout/index.md must document {artifact}"
        );
    }

    for artifact in AI_DRAFT_ARTIFACTS {
        let path = repo_dir()
            .join("docs/ui-and-layout/hub-ai-drafts")
            .join(artifact);
        assert_png_canvas(&path, (1024, 1024), "AI structure draft");
    }

    for artifact in SUPPLEMENTAL_DESIGN_ARTIFACTS {
        let path = repo_dir().join("docs/ui-and-layout").join(artifact);
        assert_png_canvas(&path, (1568, 1003), "supplemental design board");
        assert!(
            manifest.contains(artifact),
            "manifest must include supplemental design artifact {artifact}"
        );
    }
}

#[test]
fn visual_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let component_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        &shell_doc,
        &[
            "Zircon Hub Tauri React Shell",
            "web/src/theme/tokens.ts",
            "web/src/theme/muiTheme.ts",
            "components/inputs",
            "components/data",
            "components/feedback",
            "components/overlays",
            "components/shell",
            "ui_visual_standard_contract",
            "React/MUI visual standard",
        ],
        "tauri-react-shell.md",
    );
    assert_contains_all(
        &component_doc,
        &[
            "React/MUI visual standard",
            "`ui_visual_standard_contract.rs`",
            "tokens, MUI theme overrides, global CSS, shell chrome, drawer/topbar, shared data/input components",
            "design reference PNGs, AI drafts, and web-reference exporters remain comparison assets",
        ],
        "responsive-component-system.md",
    );
}

#[test]
fn visual_standard_contract_is_cut_over_to_react_sources() {
    let source = read_crate_file("tests/ui_visual_standard_contract.rs");
    let obsolete_ui_suffix = [".", "slint"].concat();
    let obsolete_reader = ["read", "_ui", "_file"].concat();
    let obsolete_root_reader = ["ui", "_dir"].concat();
    let obsolete_app_path = ["src", "app"].join("/");

    for obsolete in [
        obsolete_ui_suffix.as_str(),
        obsolete_reader.as_str(),
        obsolete_root_reader.as_str(),
        obsolete_app_path.as_str(),
    ] {
        assert!(
            !source.contains(obsolete),
            "visual-standard contract must not inspect removed UI-file or app-module surfaces: {obsolete}"
        );
    }

    assert_contains_all(
        &source,
        &[
            "web/src/theme/tokens.ts",
            "web/src/theme/muiTheme.ts",
            "web/src/styles.css",
            "HubWindow.tsx",
            "TopBar.tsx",
            "NavigationDrawer.tsx",
            "HubButton.tsx",
            "HubIconButton.tsx",
            "HubSearchField.tsx",
            "ProjectCard.tsx",
            "ProjectCover.tsx",
            "StatusBadge.tsx",
            "MetricCard.tsx",
            "EmptyStateBlock.tsx",
            "docs/ui-and-layout/hub-ai-reference-manifest.json",
            "docs/ui-and-layout/hub-web-reference/page-registry.mjs",
        ],
        "visual-standard contract",
    );
}

fn assert_png_canvas(path: &Path, expected: (u32, u32), label: &str) {
    assert!(path.exists(), "missing {label} PNG {}", path.display());
    let actual = png_dimensions(path);
    assert_eq!(
        actual,
        expected,
        "{} must have the expected {label} canvas",
        path.display()
    );
    let metadata = fs::metadata(path)
        .unwrap_or_else(|error| panic!("failed to stat {}: {error}", path.display()));
    assert!(
        metadata.len() > 16_384,
        "{} should be a rendered {label}, not an empty placeholder",
        path.display()
    );
}

fn png_dimensions(path: &Path) -> (u32, u32) {
    let bytes = fs::read(path)
        .unwrap_or_else(|error| panic!("failed to read PNG {}: {error}", path.display()));
    assert!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "{} is not a PNG file",
        path.display()
    );
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    (width, height)
}
