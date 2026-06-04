//! Static contracts for Hub Material component usage across shared wrappers and pages.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

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

fn read_ui_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub UI file {name}: {error}");
        }),
    )
}

fn read_crate_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {name}: {error}");
        }),
    )
}

fn read_repo_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read repository file {name}: {error}");
        }),
    )
}

#[test]
fn react_material_ui_packages_are_directly_composed_through_theme_and_components() {
    let package_json = read_crate_file("package.json");
    for snippet in [
        "\"@mui/material\": \"9.0.1\"",
        "\"@mui/icons-material\": \"9.0.1\"",
        "\"@emotion/react\": \"latest\"",
        "\"@emotion/styled\": \"latest\"",
        "\"@vitejs/plugin-react\": \"6.0.2\"",
    ] {
        assert!(
            package_json.contains(snippet),
            "Hub React frontend must depend on real Material UI and Vite React packages; missing {snippet}"
        );
    }

    let material_ui_package = read_repo_file("dev/material-ui/package.json");
    for snippet in [
        "\"name\": \"@mui/monorepo\"",
        "\"version\": \"9.0.1\"",
        "\"private\": true",
    ] {
        assert!(
            material_ui_package.contains(snippet),
            "the repository-local Material UI reference tree must remain available for Hub styling review; missing {snippet}"
        );
    }
    assert!(
        repo_dir()
            .join("dev/material-ui/packages/mui-material/src")
            .is_dir(),
        "the repository-local Material UI reference tree must expose packages/mui-material/src"
    );

    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    let tokens = read_crate_file("web/src/theme/tokens.ts");
    for snippet in [
        "createTheme",
        "MuiButton",
        "MuiCard",
        "MuiIconButton",
        "MuiMenu",
        "MuiOutlinedInput",
    ] {
        assert!(
            theme.contains(snippet),
            "Hub Material UI theme must centralize shared control styling; missing {snippet}"
        );
    }
    for snippet in [
        "topBarHeight: 73",
        "sidebarWidth: 222",
        "radius",
        "colors",
        "shadows",
    ] {
        assert!(
            tokens.contains(snippet),
            "Hub React tokens must own shared density, palette, and elevation values; missing {snippet}"
        );
    }

    for source_path in [
        "web/src/components/inputs/HubButton.tsx",
        "web/src/components/inputs/HubIconButton.tsx",
        "web/src/components/inputs/HubSearchField.tsx",
        "web/src/components/inputs/HubSelect.tsx",
        "web/src/components/inputs/HubToggle.tsx",
    ] {
        let source = read_crate_file(source_path);
        assert!(
            source.contains("@mui/material") || source.contains("@mui/icons-material"),
            "{source_path} must compose Material UI primitives directly"
        );
    }

    let app = read_crate_file("web/src/App.tsx");
    let main = read_crate_file("web/src/main.tsx");
    for snippet in ["ThemeProvider", "hubTheme", "CssBaseline", "<App />"] {
        assert!(
            app.contains(snippet) || main.contains(snippet),
            "React app root must install the shared Material UI theme before rendering Hub components; missing {snippet}"
        );
    }
}

#[test]
fn status_pill_uses_material_text_inside_reference_shaped_pill() {
    let shared = read_ui_file("shared.slint");
    let status_pill = shared
        .split("export component StatusPill")
        .nth(1)
        .and_then(|source| source.split("export component Panel").next())
        .expect("shared.slint must declare StatusPill before Panel");

    for snippet in [
        "border-radius: HubVisualSpec.compact-radius;",
        "in property <string> tone: \"custom\";",
        "root.tone == \"running\" ? HubVisualSpec.status-running-fill",
        "root.tone == \"running\" ? HubVisualSpec.status-running-stroke.with_alpha(0.34)",
        "root.tone == \"running\" ? HubVisualSpec.status-running-foreground",
        "root.tone == \"info\" ? HubVisualSpec.status-info-fill : root.accent-color.with-alpha(0.11)",
        "root.accent-color.with_alpha(0.36)",
        "HubVisualSpec.status-success-stroke.with_alpha(0.34)",
        "root.tone == \"warning\" ? HubVisualSpec.status-warning-stroke",
        "root.tone == \"error\" ? HubVisualSpec.status-error-stroke",
        "root.tone == \"neutral\" ? HubVisualSpec.status-neutral-stroke",
        "HorizontalLayout {",
        "reference-running.svg",
        "reference-success.svg",
        "reference-warning.svg",
        "reference-error.svg",
        "image-fit: contain;",
        "MaterialText {",
        "text: root.text;",
        "style: MaterialTypography.label_medium;",
        "private property <bool> show-running-dot: root.icon == \">\" || root.tone == \"running\";",
        "if root.show-running-dot: Rectangle",
        "width: MaterialStyleMetrics.size_6;",
        "height: parent.height;",
        "Rectangle { vertical-stretch: 1; min-height: 0px; }",
        "height: MaterialStyleMetrics.size_6;",
        "background: root.accent-color;",
        "clip: true;",
    ] {
        assert!(
            shared.contains(snippet) || status_pill.contains(snippet),
            "StatusPill must preserve the Hub header-status API while matching the reference square-rounded status pill; missing {snippet}"
        );
    }

    for forbidden in [
        "CenteredIcon",
        "ActionChip {",
        "font-size: MaterialTypography.label_large.font_size;",
        "font-weight: MaterialTypography.label_large.font_weight;",
        "colorize: root.accent-color;",
    ] {
        assert!(
            !status_pill.contains(forbidden),
            "StatusPill should not return to a custom painted icon/text pill: {forbidden}"
        );
    }
}

#[test]
fn editor_source_engine_row_uses_row_surface_slots() {
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let source_engine_row = editor_components
        .split("export component SourceEngineRow")
        .nth(1)
        .and_then(|source| source.split("export component EditorSideListPanel").next())
        .expect(
            "editor_page_components.slint must export SourceEngineRow before EditorSideListPanel",
        );
    assert!(
        !editor.contains("component SourceEngineRow"),
        "editor.slint should import SourceEngineRow instead of defining it inline"
    );

    for snippet in [
        "inherits HubInteractiveRowSurface",
        "callback engine-selected(string);",
        "selected: root.engine.active;",
        "interaction-foreground: root.content-foreground;",
        "clicked =>",
        "root.engine-selected(root.engine.id);",
        "alignment: stretch;",
        "HubRowLeadingIconSlot {",
        "shell-background: HubVisualSpec.neutral-icon-background;",
        "HubRowMainSlot {",
        "title: root.engine.title;",
        "detail: root.engine.version + \" / \" + root.engine.source-path + \" / \" + root.engine.last-build;",
        "HubRowTrailingSlot {",
        "badge-text: root.engine.status;",
        "action-icon-image: @image-url(\"../assets/icons/ui/close.svg\");",
        "root.remove(root.engine.id);",
    ] {
        assert!(
            source_engine_row.contains(snippet),
            "SourceEngineRow must compose source-engine rows through shared row-surface slots; missing {snippet}"
        );
    }

    for forbidden in [
        "area := TouchArea",
        "ListTile {",
        "Badge {",
        "IconButton {",
        "avatar_icon:",
        "avatar_background:",
        "avatar_foreground:",
        "callback selected(string);",
        "border-color: area.has-hover",
        "background: area.has-hover",
        "row-state := StateLayerArea {",
        "border-color: root.engine.active ?",
        "background: root.engine.active ?",
        "root.height - MaterialStyleMetrics.spacing_16",
    ] {
        assert!(
            !source_engine_row.contains(forbidden),
            "SourceEngineRow should not return to a local ListTile, badge/action, or custom row-surface implementation: {forbidden}"
        );
    }
}

#[test]
fn project_setting_summary_rows_use_key_value_row_slots() {
    let components = read_ui_file("components.slint");
    let data_display = read_ui_file("data_display.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_detail_components = read_ui_file("project_detail_components.slint");
    let material_components =
        read_repo_file("docs/ui-and-layout/hub-web-reference/material-components.js");
    let material_styles =
        read_repo_file("docs/ui-and-layout/hub-web-reference/material-styles.css");
    let web_app = read_repo_file("docs/ui-and-layout/hub-web-reference/app.js");
    let responsive_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-responsive.mjs");

    let key_value_row = data_display
        .split("export component HubKeyValueRow")
        .nth(1)
        .and_then(|source| source.split("export component ActionRow").next())
        .expect("data_display.slint must declare HubKeyValueRow before ActionRow");
    for snippet in [
        "export component HubKeyValueRow inherits HubRowSurface",
        "HubRowMetaSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "badge-text: root.value;",
        "show-action: false;",
    ] {
        assert!(
            data_display.contains(snippet) || key_value_row.contains(snippet),
            "HubKeyValueRow must be the shared key/value summary row-slot primitive; missing {snippet}"
        );
    }
    assert!(
        components.contains("HubKeyValueRow,"),
        "components.slint must re-export HubKeyValueRow for page-specific summary wrappers"
    );

    let setting_summary_row = project_components
        .split("export component ProjectSettingSummaryRow")
        .nth(1)
        .and_then(|source| source.split("export component ProjectCreateSummary").next())
        .expect(
            "project_page_components.slint must declare ProjectSettingSummaryRow before ProjectCreateSummary",
        );
    for snippet in [
        "inherits HubKeyValueRow",
        "label-width: root.row-height * 9 / 2;",
        "badge-width: root.row-height * 5;",
        "row-spacing: max(MaterialStyleMetrics.spacing_8, root.row-height / 5);",
    ] {
        assert!(
            setting_summary_row.contains(snippet),
            "ProjectSettingSummaryRow should only specialize HubKeyValueRow geometry; missing {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "Badge {",
        "HorizontalLayout {",
        "border-width: 0px;",
        "background: transparent;",
    ] {
        assert!(
            !setting_summary_row.contains(forbidden),
            "ProjectSettingSummaryRow should not return to local text/badge layout after HubKeyValueRow extraction: {forbidden}"
        );
    }
    assert!(
        project_detail_components.contains("ProjectSettingSummaryRow {")
            && project_components.contains("ProjectSettingSummaryRow {"),
        "Project Detail and New Project summaries should consume the same ProjectSettingSummaryRow wrapper"
    );

    for snippet in [
        "setting-summary-row",
        "function settingSummaryRow",
        "row-meta-slot",
        "row-main-slot",
        "row-trailing-slot",
        "settingSummaryRow,",
        "data-component=\"setting-summary-row\"",
        ".setting-summary-row",
        "requiredComponents.push(\"setting-summary-row\", \"row-meta-slot\", \"row-main-slot\", \"row-trailing-slot\");",
    ] {
        assert!(
            material_components.contains(snippet)
                || material_styles.contains(snippet)
                || web_app.contains(snippet)
                || responsive_validator.contains(snippet),
            "Hub web reference must expose the setting-summary-row molecule before Slint migration relies on it: {snippet}"
        );
    }
    for snippet in [
        "settingSummaryRow(\"Source Engine\", \"Engine Alpha v2.8.1\")",
        "settingSummaryRow(\"Template\", \"Standard Service\")",
        "settingSummaryRow(\"Compatibility\", \"Ready\", true, \"success\")",
    ] {
        assert!(
            web_app.contains(snippet),
            "New Project Blueprint summary should render through settingSummaryRow instead of info rows: {snippet}"
        );
    }
}

#[test]
fn project_detail_status_strip_uses_badge_meta_strip_slots() {
    let components = read_ui_file("components.slint");
    let data_display = read_ui_file("data_display.slint");
    let project_detail_components = read_ui_file("project_detail_components.slint");
    let project_detail_page = read_ui_file("project_detail_page.slint");
    let material_components =
        read_repo_file("docs/ui-and-layout/hub-web-reference/material-components.js");
    let material_styles =
        read_repo_file("docs/ui-and-layout/hub-web-reference/material-styles.css");
    let web_app = read_repo_file("docs/ui-and-layout/hub-web-reference/app.js");
    let responsive_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-responsive.mjs");

    let badge_meta_strip = data_display
        .split("export component HubBadgeMetaStrip")
        .nth(1)
        .and_then(|source| source.split("export component ActionRow").next())
        .expect("data_display.slint must declare HubBadgeMetaStrip before ActionRow");
    for snippet in [
        "export component HubBadgeMetaStrip inherits HubRowSurface",
        "HubRowTrailingSlot {",
        "HubRowMetaSlot {",
        "badge-text: root.first-badge-text;",
        "badge-text: root.second-badge-text;",
        "text: root.meta-text;",
        "show-action: false;",
    ] {
        assert!(
            data_display.contains(snippet) || badge_meta_strip.contains(snippet),
            "HubBadgeMetaStrip must be the shared badge/meta status-strip primitive; missing {snippet}"
        );
    }
    assert!(
        components.contains("HubBadgeMetaStrip,"),
        "components.slint must re-export HubBadgeMetaStrip for Project Detail wrappers"
    );

    let status_strip = project_detail_components
        .split("export component ProjectDetailStatusStrip")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailInfoSection").next())
        .expect(
            "project_detail_components.slint must declare ProjectDetailStatusStrip before ProjectDetailInfoSection",
        );
    for snippet in [
        "inherits HubBadgeMetaStrip",
        "first-badge-text: root.detail.version;",
        "first-badge-tone: \"accent\";",
        "first-badge-width: root.version-badge-width;",
        "second-badge-text: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "second-badge-tone: root.detail.pinned ? \"accent\" : \"neutral\";",
        "second-badge-width: root.pin-badge-width;",
        "meta-text: root.copy.modified-prefix + root.detail.modified;",
    ] {
        assert!(
            status_strip.contains(snippet),
            "ProjectDetailStatusStrip should only map project status data into HubBadgeMetaStrip; missing {snippet}"
        );
    }
    for forbidden in [
        "Badge {",
        "MaterialText {",
        "HorizontalLayout {",
        "background: transparent;",
    ] {
        assert!(
            !status_strip.contains(forbidden),
            "ProjectDetailStatusStrip should not return to local badge/text strip layout after HubBadgeMetaStrip extraction: {forbidden}"
        );
    }
    let main_panel = project_detail_components
        .split("export component ProjectDetailMainPanel")
        .nth(1)
        .expect("project_detail_components.slint must declare ProjectDetailMainPanel");
    assert!(
        project_detail_page.contains("ProjectDetailMainPanel {")
            && main_panel.matches("ProjectDetailStatusStrip {").count() == 1,
        "ProjectDetailPage should route the main detail column through ProjectDetailMainPanel, which owns one status-strip wrapper call"
    );

    for snippet in [
        "project-status-strip",
        "function projectStatusStrip",
        "row-meta-slot",
        "row-trailing-slot",
        "projectStatusStrip,",
        "data-component=\"project-status-strip\"",
        ".project-status-strip",
        "requiredComponents.push(\"project-status-strip\", \"row-meta-slot\", \"row-trailing-slot\");",
    ] {
        assert!(
            material_components.contains(snippet)
                || material_styles.contains(snippet)
                || web_app.contains(snippet)
                || responsive_validator.contains(snippet),
            "Hub web reference must expose the Project Detail status strip molecule before Slint migration relies on it: {snippet}"
        );
    }
    assert!(
        web_app.contains("projectDetailMainPanel(selected)")
            && material_components.contains(
                "projectStatusStrip(project.version, \"Not pinned\", project.modified)"
            ),
        "Project Detail web reference should render status badges and modified metadata through projectDetailMainPanel/projectStatusStrip"
    );
}

#[test]
fn project_pages_use_material_scroll_view() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_card_flow_components = read_ui_file("project_card_flow_components.slint");
    let dashboard_surface =
        format!("{dashboard}\n{dashboard_components}\n{project_card_flow_components}");
    let project_components = read_ui_file("project_page_components.slint");
    let project_detail_components = read_ui_file("project_detail_components.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let project_new_page = read_ui_file("project_new_page.slint");
    let project_browser_page = read_ui_file("project_browser_page.slint");
    let project_detail_page = read_ui_file("project_detail_page.slint");
    let project_browser_components = read_ui_file("project_browser_components.slint");
    let layout = read_ui_file("layout.slint");
    let project_surface = format!(
        "{project_pages}\n{project_new_page}\n{project_browser_page}\n{project_detail_page}\n{project_components}\n{project_browser_components}\n{project_detail_components}"
    );

    for snippet in [
        "FlowScrollSurface",
        "HubTableView,",
        "HubListPanelSlot,",
        "export component DashboardRecentProjectsPanel inherits HubTableView",
        "export component DashboardQuickActionsPanel inherits HubListPanelSlot",
    ] {
        assert!(
            dashboard_surface.contains(snippet),
            "ProjectDashboardPage must route page/list scrolling through shared Material-backed scroll wrappers; missing {snippet}"
        );
    }
    for snippet in [
        "ScrollView,",
        "export component FlowScrollSurface inherits ScrollView",
        "vertical_scrollbar_policy: ScrollBarPolicy.as-needed;",
        "horizontal_scrollbar_policy: ScrollBarPolicy.always-off;",
    ] {
        assert!(
            layout.contains(snippet),
            "layout.slint must own the Material ScrollView API used by Dashboard flow scrolling; missing {snippet}"
        );
    }
    for snippet in [
        "export component ProjectCreateSettingsPanel inherits HubContentPanelSlot",
        "export component ProjectCreateCompactSummaryPanel inherits HubContentPanelSlot",
        "export component ProjectTemplateRailPanel inherits HubListPanelSlot",
        "HubListPanelSlot,",
        "ProjectCreateSettingsPanel {",
        "ProjectCreateCompactSummaryPanel {",
        "ProjectTemplateRailPanel {",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.template-count;",
        "ProjectBrowserResultsPanel {",
        "browser-list := HubTableBody {",
        "export component ProjectEngineChoiceList inherits PanelListViewport",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.engine-count;",
        "ProjectEngineChoiceList {",
        "export component ProjectCreateActionRow inherits HubFormActionRow",
        "ProjectCreateActionRow {",
    ] {
        assert!(
            project_surface.contains(snippet),
            "Project secondary pages must route list scrolling through the shared Material ScrollView wrapper; missing {snippet}"
        );
    }
    for (page, source) in [
        ("ProjectDashboardPage", &dashboard),
        ("Project New page", &project_new_page),
        ("Project Browser page", &project_browser_page),
        ("Project Detail page", &project_detail_page),
    ] {
        for forbidden in [
            "std-widgets.slint",
            "viewport-y <=>",
            "mouse-drag-pan-enabled",
        ] {
            assert!(
                !source.contains(forbidden),
                "{page} should not return to std-widgets ScrollView properties: {forbidden}"
            );
        }
        for forbidden_property in ["visible-width", "visible-height"] {
            assert!(
                !source.lines().any(|line| {
                    let trimmed = line.trim_start();
                    trimmed.starts_with(&format!("{forbidden_property}:"))
                        || trimmed.starts_with(&format!("{forbidden_property} <=>"))
                }),
                "{page} should not return to std-widgets ScrollView property {forbidden_property}"
            );
        }
    }

    for snippet in [
        "PageScrollSurface,",
        "export component ProjectDashboardPage inherits PageScrollSurface",
        "page-padding: root.page-pad-x;",
        "page-padding-x: root.page-pad-x;",
        "page-padding-y: root.page-pad-y;",
        "bottom-padding: root.page-pad-y;",
        "gap: root.page-gap;",
        "root.content-height / 18",
        "root.viewport-height * HubTokens.project-dashboard-lower-compact-ratio",
        "root.viewport-height * HubTokens.project-dashboard-lower-regular-ratio",
    ] {
        assert!(
            dashboard.contains(snippet),
            "ProjectDashboardPage must inherit the shared Material PageScrollSurface directly; missing {snippet}"
        );
    }
    for forbidden in [
        "page-surface := PageScrollSurface {",
        "scroll-y <=> root.scroll-y;",
        "content-width: page-surface.content-width;",
        "page-surface.content-height",
        "page-surface.viewport-height",
        "content-width: max(1px, root.width",
        "root.width - root.page-pad",
        "root.height /",
        "page-scroll := ScrollView",
        "dashboard-scroll := ScrollView",
        "width: root.content-width;",
    ] {
        assert!(
            !dashboard.contains(forbidden),
            "ProjectDashboardPage should inherit PageScrollSurface content sizing instead of nesting a page surface or hand-written page formulas: {forbidden}"
        );
    }

    for snippet in [
        "PageScrollSurface,",
        "export component ProjectNewPage inherits PageScrollSurface",
        "export component ProjectBrowserPage inherits PageScrollSurface",
        "export component ProjectDetailPage inherits PageScrollSurface",
        "page-padding: root.page-pad;",
        "bottom-padding: root.page-pad;",
        "gap: root.page-gap;",
        "root.content-height / 18",
        "root.content-height / 13",
        "root.detail-visible-height",
        "private property <length> browser-scroll-y: 0px;",
    ] {
        assert!(
            project_surface.contains(snippet),
            "Project secondary pages must inherit the shared Material PageScrollSurface directly; missing {snippet}"
        );
    }
    for forbidden in [
        "page-surface := PageScrollSurface {",
        "scroll-y <=> root.scroll-y;",
        "scroll-y <=> root.page-scroll-y;",
        "content-width: page-surface.content-width;",
        "page-surface.content-height",
        "page-surface.viewport-height",
        "private property <length> content-height:",
        "content-width: max(1px, root.width",
        "root.width - root.page-pad",
        "root.height /",
        "page-scroll := ScrollView",
        "dashboard-scroll := ScrollView",
        "width: root.content-width;",
    ] {
        assert!(
            !project_pages.contains(forbidden)
                && !project_new_page.contains(forbidden)
                && !project_browser_page.contains(forbidden)
                && !project_detail_page.contains(forbidden),
            "Project secondary pages should derive page content sizing from PageScrollSurface instead of hand-written page formulas: {forbidden}"
        );
    }

    for snippet in [
        "card-scroll := FlowScrollSurface {",
        "scroll-y <=> root.card-scroll-y;",
        "content-height: root.expanded ? root.flow-content-height : root.flow-visible-height;",
        "export component DashboardRecentProjectsPanel inherits HubTableView",
        "show-header: true;",
        "show-divider: false;",
        "export component DashboardQuickActionsPanel inherits HubListPanelSlot",
        "HubListPanelSlot,",
        "DashboardQuickActionsPanel {",
        "quick-actions-scroll-y <=> root.quick-actions-scroll-y;",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "row-count: root.quick-action-count;",
    ] {
        assert!(
            dashboard_surface.contains(snippet),
            "ProjectDashboardPage must keep project cards on the shared Material-backed flow viewport and quick actions on the shared list viewport; missing {snippet}"
        );
    }
    assert!(
        !project_card_flow_components.contains("card-scroll := ScrollView")
            && !project_card_flow_components.contains("ScrollView,"),
        "ProjectFlow should consume FlowScrollSurface instead of importing Material ScrollView directly"
    );

    for snippet in [
        "browser-list := HubTableBody {",
        "scroll-y <=> root.list-scroll-y;",
        "row-width: root.table-row-width;",
    ] {
        assert!(
            project_browser_components.contains(snippet),
            "Project Browser must keep list scrolling on the shared Material ScrollView wrapper while New/Detail use PageScrollSurface; missing {snippet}"
        );
    }
    for snippet in [
        "ProjectBrowserResultsPanel {",
        "list-scroll-y <=> root.browser-scroll-y;",
        "list-height: root.browser-list-height;",
    ] {
        assert!(
            project_browser_page.contains(snippet),
            "ProjectBrowserPage should forward Browser list sizing and scroll state into ProjectBrowserResultsPanel; missing {snippet}"
        );
    }
    assert!(
        !project_browser_page.contains("browser-list := PanelListViewport")
            && !project_browser_page.contains("row-width: browser-list.visible_width;"),
        "ProjectBrowserPage should not own the Browser list viewport internals"
    );
    assert!(
        project_pages
            .contains("export { ProjectBrowserPage } from \"project_browser_page.slint\";")
            && project_pages.contains("export { ProjectNewPage } from \"project_new_page.slint\";")
            && project_pages.contains("export { ProjectDetailPage } from \"project_detail_page.slint\";")
            && !project_pages.contains("export component ProjectBrowserPage inherits"),
        "project_pages.slint should expose New, Browser, and Detail pages through dedicated page modules"
    );
}

#[test]
fn projects_page_routes_to_dashboard_module() {
    let projects = read_ui_file("projects.slint");
    let line_count = projects.lines().count();
    assert!(
        line_count <= 220,
        "projects.slint should stay a subpage router; found {line_count} lines"
    );
    assert!(
        projects.contains("ProjectDashboardPage"),
        "projects.slint must route the dashboard through ProjectDashboardPage"
    );
    assert!(
        !projects.contains("component ProjectCard")
            && !projects.contains("component ProjectFlow")
            && !projects.contains("dashboard-scroll :="),
        "dashboard implementation details belong in focused dashboard component modules"
    );
    for forbidden in [
        "project-entry-mode",
        "project-list-rows",
        "project-list-row-count",
        "recent-project-rows",
        "recent-project-row-count",
    ] {
        assert!(
            !projects.contains(forbidden),
            "projects.slint should not keep unused dashboard/list-era routing inputs: {forbidden}"
        );
    }
    let app = read_ui_file("app.slint");
    for forbidden in [
        "project-entry-mode",
        "project-list-rows",
        "project-list-row-count",
        "recent-project-rows",
        "recent-project-row-count",
    ] {
        assert!(
            !app.contains(forbidden),
            "app.slint should not pass unused Projects list-era routing inputs: {forbidden}"
        );
    }
    let binding = read_crate_file("src/app/binding.rs");
    for forbidden in [
        "project_list_rows",
        "set_project_list_row_count",
        "set_project_list_rows",
        "recent_project_rows",
        "set_recent_project_row_count",
        "set_recent_project_rows",
    ] {
        assert!(
            !binding.contains(forbidden),
            "binding.rs should only project dashboard rows and browser rows for Projects: {forbidden}"
        );
    }
    for snippet in [
        "in-out property <string> project-detail-return-subpage: \"dashboard\";",
        "root.project-view-mode = \"list\";",
        "root.project-subpage = \"project-browser\";",
        "root.project-detail-return-subpage = \"dashboard\";",
        "root.project-detail-return-subpage = \"project-browser\";",
        "root.project-subpage = \"project-detail\";",
        "root.project-subpage = root.project-detail-return-subpage;",
        "root.show-project-subpage(root.project-detail-return-subpage);",
        "root.project-view-mode = \"grid\";",
    ] {
        assert!(
            projects.contains(snippet),
            "projects.slint must update local Projects subpage/view state before relying on runtime callbacks; missing {snippet}"
        );
    }

    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_card_flow_components = read_ui_file("project_card_flow_components.slint");
    let dashboard_surface =
        format!("{dashboard}\n{dashboard_components}\n{project_card_flow_components}");
    for primitive in [
        "ProjectFlow",
        "Flow",
        "PanelGrid",
        "PanelSlot",
        "ResponsiveSlot",
    ] {
        assert!(
            dashboard_surface.contains(primitive),
            "project_dashboard.slint must compose dashboard layout with {primitive}"
        );
    }
}

#[test]
fn dashboard_project_selectors_use_material_state_layers() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_card_flow_components = read_ui_file("project_card_flow_components.slint");
    let dashboard_surface =
        format!("{dashboard}\n{dashboard_components}\n{project_card_flow_components}");
    let project_card = project_card_flow_components
        .split("export component ProjectCard")
        .nth(1)
        .and_then(|source| source.split("export component ProjectFlow").next())
        .expect("project_card_flow_components.slint must export ProjectCard before ProjectFlow");
    for snippet in [
        "HubInteractiveCardSurface",
        "export component ProjectCard inherits HubInteractiveCardSurface",
        "selected: root.project.selected;",
        "interaction-foreground: MaterialPalette.on_surface;",
        "clicked =>",
        "root.select(root.project.open-path);",
    ] {
        assert!(
            dashboard_surface.contains(snippet) || project_card.contains(snippet),
            "ProjectCard must delegate whole-card select feedback to the shared interactive card surface; missing {snippet}"
        );
    }
    for forbidden in [
        "area := TouchArea",
        "area.has-hover",
        "card-state := StateLayerArea {",
        "border_radius: HubVisualSpec.panel-radius;",
    ] {
        assert!(
            !project_card.contains(forbidden),
            "ProjectCard should not return to custom full-card TouchArea or local StateLayerArea handling: {forbidden}"
        );
    }

    let table_view = read_ui_file("table_view_components.slint");
    let table_row = table_view
        .split("export component ProjectTableRow")
        .nth(1)
        .and_then(|source| source.split("export component DataTable").next())
        .expect("table_view_components.slint must declare ProjectTableRow before DataTable");
    for snippet in [
        "import { HubInteractiveRowSurface } from \"data_display.slint\";",
        "import { PanelListViewport } from \"list_container_components.slint\";",
        "export component ProjectTableRow inherits HubInteractiveRowSurface",
        "row-radius: HubVisualSpec.compact-radius;",
        "interaction-foreground: root.content-foreground;",
        "root.select(root.project.open-path);",
    ] {
        assert!(
            table_view.contains(snippet) || table_row.contains(snippet),
            "ProjectTableRow must delegate whole-row select feedback to the shared interactive row surface; missing {snippet}"
        );
    }
    for forbidden in [
        "area := TouchArea",
        "area.has-hover",
        "row-state := StateLayerArea {",
    ] {
        assert!(
            !table_row.contains(forbidden),
            "ProjectTableRow should not return to custom full-row TouchArea or local StateLayerArea handling: {forbidden}"
        );
    }
}

#[test]
fn project_choice_rows_use_shared_material_row_slots_and_checkbox_selection() {
    let components = read_ui_file("project_page_components.slint");
    let data_display = read_ui_file("data_display.slint");
    let shared = read_ui_file("shared.slint");
    assert!(
        data_display.contains("export component InfoRow inherits HubInteractiveRowSurface")
            && data_display.contains("HubRowLeadingIconSlot {")
            && data_display.contains("HubRowMainSlot {")
            && data_display.contains("HubRowTrailingSlot {"),
        "InfoRow must remain available as a shared interactive row-surface and row-slot-backed body for page rows"
    );
    let engine_choice_row = components
        .split("export component EngineChoiceRow")
        .nth(1)
        .and_then(|source| source.split("export component TemplateChoiceRow").next())
        .expect(
            "project_page_components.slint must declare EngineChoiceRow before TemplateChoiceRow",
        );
    for snippet in [
        "inherits HubInteractiveRowSurface",
        "in property <bool> collapse-label: false;",
        "callback engine-selected(string);",
        "effective-row-height: max(root.row-height, HubTokens.list-row-md);",
        "supporting-text: root.engine.version == \"\" ? root.engine.source-path : (root.engine.source-path == \"\" ? root.engine.version : root.engine.version + \" / \" + root.engine.source-path);",
        "trailing-label: root.engine.active ? root.selected-label : root.registered-label;",
        "height: root.effective-row-height;",
        "row-height: root.effective-row-height;",
        "selected: root.engine.active;",
        "interaction-foreground: root.content-foreground;",
        "clicked =>",
        "root.engine-selected(root.engine.id);",
        "HubRowLeadingIconSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "detail: root.supporting-text;",
        "slot-width: root.collapse-label ? 0px : root.trailing-badge-width;",
        "badge-text: root.trailing-label;",
        "badge-tone: root.engine.active ? \"accent\" : \"neutral\";",
        "collapse-badge: root.collapse-label;",
    ] {
        assert!(
            engine_choice_row.contains(snippet),
            "EngineChoiceRow must compose the shared row-surface and row-slot primitives while preserving engine text, selection, and compact badge behavior; missing {snippet}"
        );
    }
    let template_choice_row = components
        .split("export component TemplateChoiceRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("project_page_components.slint must declare TemplateChoiceRow");
    for snippet in [
        "inherits HubInteractiveRowSurface",
        "private property <CheckState> selection-state: root.template.selected ? CheckState.checked : CheckState.unchecked;",
        "interaction-enabled: root.template.enabled;",
        "interaction-foreground: root.template.selected ? HubVisualSpec.accent-stroke : MaterialPalette.on_surface;",
        "clicked =>",
        "HubRowSelectionSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "check-state: root.selection-state;",
        "badge-text: root.trailing-label;",
        "row-height: root.effective-row-height;",
        "selected: root.template.selected;",
        "root.template-selected(root.template.id);",
    ] {
        assert!(
            template_choice_row.contains(snippet),
            "TemplateChoiceRow must render template selection through shared row-slot components backed by Material primitives; missing {snippet}"
        );
    }
    for forbidden in [
        "CenteredIcon",
        "area := TouchArea",
        "InfoRow {",
        "HubCheckBox {",
        "StatusBadge {",
        "StateLayerArea {",
    ] {
        assert!(
            !engine_choice_row.contains(forbidden) && !template_choice_row.contains(forbidden),
            "Project choice rows should not return to page-local icon, click, checkbox, or badge rows: {forbidden}"
        );
    }

    let project_pages = read_ui_file("project_pages.slint");
    let project_new_page = read_ui_file("project_new_page.slint");
    let project_detail_page = read_ui_file("project_detail_page.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_detail_components = read_ui_file("project_detail_components.slint");
    let project_surface =
        format!("{project_pages}\n{project_new_page}\n{project_detail_page}\n{project_components}\n{project_detail_components}");
    let actions_section = project_detail_components
        .split("export component ProjectDetailActionsSection")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailStatusStrip").next())
        .expect("project_detail_components.slint must declare ProjectDetailActionsSection before ProjectDetailStatusStrip");
    let action_stack = project_detail_components
        .split("export component ProjectDetailActionStack")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailDeleteActionStack").next())
        .expect("project_detail_components.slint must declare ProjectDetailActionStack before ProjectDetailDeleteActionStack");
    let delete_action_stack = project_detail_components
        .split("export component ProjectDetailDeleteActionStack")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailEngineSection").next())
        .expect("project_detail_components.slint must declare ProjectDetailDeleteActionStack before ProjectDetailEngineSection");
    for snippet in [
        "choice-row-height: max(HubTokens.list-row-md, min(HubTokens.list-row-lg, root.content-height / 10));",
        "template-row-height: max(HubTokens.list-row-md, min(HubTokens.list-row-lg, root.content-height / 9));",
        "compact-choice-labels: root.narrow-flow;",
        "collapse-label: root.compact-choice-labels;",
    ] {
        assert!(
            project_new_page.contains(snippet),
            "ProjectNewPage choice rows must use responsive Material row tokens instead of undersized local formulas; missing {snippet}"
        );
    }
    assert!(
        project_detail_page.contains("detail-choice-row-height: max(HubTokens.list-row-md"),
        "ProjectDetailPage engine choices must respect the shared Material row minimum height"
    );
    assert!(
        project_detail_page.contains("collapse-engine-label: root.narrow-flow;")
            && project_detail_components.contains("collapse-label: root.collapse-engine-label;"),
        "ProjectDetailPage engine choices should collapse trailing labels through the typed detail action section in the compact flow"
    );
    for snippet in [
        "change-source-engine: string,",
        "remove-from-hub-detail: string,",
        "StatusBanner,",
        "text: root.copy.remove-from-hub-detail;",
        "if root.project.pending-delete: ProjectDetailDeleteActionStack",
        "export component ProjectDetailActionStack inherits HubActionStack",
        "export component ProjectDetailDeleteActionStack inherits HubActionStack",
        "StatusBanner {",
        "title: root.copy.confirm-delete;",
        "detail: root.copy.recycle-bin-delete-detail;",
        "tone: \"error\";",
        "export component ProjectDetailActionsSection inherits HubContentPanelSlot",
        "body-padding: root.panel-padding;",
        "body-spacing: root.panel-spacing;",
        "content-spacing: root.panel-spacing;",
        "title: root.copy.project-actions-title;",
        "project: root.project;",
        "copy: root.ui-text;",
        "engine-scroll-y <=> root.detail-engine-scroll-y;",
        "open-project(path) => { root.open-project(path); }",
        "select-engine(id) => { root.select-engine(id); }",
        "toggle-pin => { root.toggle-pin(); }",
        "remove-from-hub => { root.remove-from-hub(); }",
        "request-delete => { root.request-delete(); }",
        "cancel-delete => { root.cancel-delete(); }",
        "confirm-delete => { root.confirm-delete(); }",
        "export component ProjectDetailStatusStrip inherits HubBadgeMetaStrip",
        "first-badge-text: root.detail.version;",
        "second-badge-text: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "second-badge-tone: root.detail.pinned ? \"accent\" : \"neutral\";",
        "meta-text: root.copy.modified-prefix + root.detail.modified;",
        "export component ProjectDetailMainPanel inherits HubMediaContentPanelSlot",
        "media-height: root.cover-height;",
        "media-source: root.project.cover-image;",
        "has-media-source: root.project.has-cover;",
        "content-spacing: root.content-stack-spacing;",
        "ProjectDetailStatusStrip {",
        "ProjectDetailInfoSection {",
        "export component ProjectDetailPinToggleRow inherits HubToggleRow",
        "checked: root.detail.pinned;",
        "label: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "supporting-text: root.detail.pinned ? root.copy.unpin-project : root.copy.pin-project;",
        "export component ProjectDetailEngineSection inherits HubSection",
        "title: root.copy.change-source-engine;",
        "subtitle: root.copy.bound-source-engine + \": \" + root.detail.engine-label;",
        "ProjectDetailEngineSection {",
        "HubActionCommandButton {",
        "source-image: @image-url(\"../assets/icons/nav/editor.svg\");",
        "source-image: @image-url(\"../assets/icons/ui/close.svg\");",
        "source-image: @image-url(\"../assets/icons/ui/alert.svg\");",
        "has-source-image: true;",
        "clicked => { root.confirm-delete(); }",
        "clicked => { root.cancel-delete(); }",
        "clicked => { root.open-project(root.project.open-path); }",
        "if !root.project.pending-delete: ProjectDetailActionStack",
        "ProjectDetailPinToggleRow {",
        "toggled(checked) => { root.toggle-pin(); }",
        "clicked => { root.remove-from-hub(); }",
        "clicked => { root.request-delete(); }",
        "if !root.project.pending-delete: ProjectDetailEngineSection",
    ] {
        assert!(
            shared.contains(snippet) || project_surface.contains(snippet),
            "ProjectDetailPage must expose changing the bound Source Engine and the non-destructive Remove from Hub action as explicit secondary-page operations; missing {snippet}"
        );
    }
    assert!(
        !project_detail_components.contains("export component ProjectDetailActionButton")
            && !project_detail_components.contains("ProjectDetailActionButton {"),
        "ProjectDetailActionsSection should consume the shared HubActionCommandButton directly instead of a pass-through ProjectDetailActionButton wrapper"
    );
    assert_eq!(
        action_stack.matches("HubActionCommandButton {").count()
            + delete_action_stack
                .matches("HubActionCommandButton {")
                .count(),
        5,
        "ProjectDetail action stacks should render confirm, cancel, open, remove, and delete through HubActionCommandButton while pinning uses a toggle row"
    );
    assert_eq!(
        action_stack.matches("ProjectDetailPinToggleRow {").count(),
        1,
        "ProjectDetailActionStack should render pin/unpin through one Material toggle row"
    );
    assert_eq!(
        project_detail_page
            .matches("ProjectDetailActionsSection {")
            .count(),
        1,
        "ProjectDetailPage should render the whole actions column through one ProjectDetailActionsSection"
    );
    assert_eq!(
        project_detail_components
            .matches("ProjectDetailStatusStrip {")
            .count(),
        1,
        "ProjectDetailMainPanel should render version, pin state, and modified time through one status-strip wrapper"
    );
    assert_eq!(
        actions_section
            .matches("ProjectDetailEngineSection {")
            .count(),
        1,
        "ProjectDetailActionsSection should render the Change Source Engine block through one section wrapper"
    );
    for forbidden in [
        "text: root.project.pending-delete ? root.ui-text.confirm-delete : root.ui-text.delete-project;",
        "if (root.project.pending-delete) {",
        "if root.project.pending-delete: PillButton",
        "if !root.project.pending-delete: PillButton",
    ] {
        assert!(
            !project_pages.contains(forbidden) && !project_detail_page.contains(forbidden),
            "ProjectDetailPage pending delete should use a confirmation-first action cluster instead of a bottom-of-panel ternary delete button: {forbidden}"
        );
    }
}
