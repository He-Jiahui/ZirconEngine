//! Static contracts for Projects-page Taffy layout usage.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
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

fn assert_semantic_taffy_properties_have_slint_flex_pairs(page: &str, source: &str) {
    let lines = source.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some((semantic, flex_property)) = trimmed
            .strip_prefix("basis: ")
            .map(|value| (value, "flex-basis"))
            .or_else(|| {
                trimmed
                    .strip_prefix("grow: ")
                    .map(|value| (value, "flex-grow"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("shrink: ")
                    .map(|value| (value, "flex-shrink"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("order: ")
                    .map(|value| (value, "flex-order"))
            })
        else {
            continue;
        };
        let expected = format!("{flex_property}: {semantic}");
        let next = lines
            .get(index + 1)
            .map(|line| line.trim())
            .unwrap_or_default();
        assert_eq!(
            next, expected,
            "{page} must keep Slint-required {flex_property} directly paired with semantic ResponsiveSlot sizing"
        );
    }

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some((flex_property, semantic_property)) = trimmed
            .strip_prefix("flex-basis: ")
            .map(|_| ("flex-basis", "basis"))
            .or_else(|| {
                trimmed
                    .strip_prefix("flex-grow: ")
                    .map(|_| ("flex-grow", "grow"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("flex-shrink: ")
                    .map(|_| ("flex-shrink", "shrink"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("flex-order: ")
                    .map(|_| ("flex-order", "order"))
            })
        else {
            continue;
        };
        let previous = index
            .checked_sub(1)
            .and_then(|previous| lines.get(previous))
            .map(|line| line.trim())
            .unwrap_or_default();
        assert!(
            previous.starts_with(&format!("{semantic_property}: ")),
            "{page} must not hand-code {flex_property} without the matching semantic ResponsiveSlot {semantic_property}"
        );
    }
}

#[test]
fn project_pages_use_responsive_taffy_sizing() {
    let app = read_ui_file("app.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_card_flow_components = read_ui_file("project_card_flow_components.slint");
    let dashboard_surface =
        format!("{dashboard}\n{dashboard_components}\n{project_card_flow_components}");
    let projects = read_ui_file("projects.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_browser_components = read_ui_file("project_browser_components.slint");
    let project_detail_components = read_ui_file("project_detail_components.slint");
    let project_new_page = read_ui_file("project_new_page.slint");
    let project_browser_page = read_ui_file("project_browser_page.slint");
    let project_detail_page = read_ui_file("project_detail_page.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let project_surface = format!(
        "{project_pages}\n{project_new_page}\n{project_browser_page}\n{project_detail_page}\n{project_components}\n{project_browser_components}\n{project_detail_components}"
    );
    let browser_page = &project_browser_page;
    for (page, source) in [
        ("ProjectDashboardPage", &dashboard),
        ("Project New page", &project_new_page),
        ("Project Browser page", &project_browser_page),
        ("Project Detail page", &project_detail_page),
    ] {
        assert_semantic_taffy_properties_have_slint_flex_pairs(page, source);
    }
    for (page, source, snippets) in [
        (
            "ProjectDashboardPage",
            &dashboard_surface,
            &[
                "search-basis: root.dashboard-toolbar-search-basis;",
                "select-basis: root.dashboard-toolbar-select-basis;",
                "grow: root.toolbar-wrap ? 1 : 0;",
                "flex-grow: root.toolbar-wrap ? 1 : 0;",
                "basis: root.search-basis;",
                "flex-basis: root.search-basis;",
                "basis: root.select-basis;",
                "flex-basis: root.select-basis;",
                "basis: root.compact ? root.content-width : root.dashboard-main-basis;",
                "flex-basis: root.compact ? root.content-width : root.dashboard-main-basis;",
            ][..],
        ),
        (
            "Project secondary surface",
            &project_surface,
            &[
                "basis: root.toolbar-search-basis;",
                "flex-basis: root.toolbar-search-basis;",
                "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
                "flex-basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
                "grow: 2;",
                "flex-grow: 2;",
            ][..],
        ),
    ] {
        for snippet in snippets {
            assert!(
                source.contains(snippet),
                "{page} should keep ResponsiveSlot/PanelSlot semantic sizing and mirror it to Slint flex properties at the direct Flexbox child use site: {snippet}"
            );
        }
    }
    for snippet in [
        "for card in root.project-cards: ProjectCard",
        "min-width: max(HubTokens.panel-min-sm * 2 / 3, min(HubTokens.panel-min-sm, root.card-basis));",
        "card-width-basis: root.card-basis;",
        "card-gap-x: HubTokens.space-6 - MaterialStyleMetrics.size_2;",
        "raw-card-columns: Math.floor((root.flow-width + root.card-gap-x) / (root.card-basis + root.card-gap-x));",
        "card-column-count: root.raw-card-columns < 1 ? 1 : root.raw-card-columns;",
        "card-row-count: (root.project-card-count + root.card-column-count - 1) / root.card-column-count;",
        "expanded-visible-rows: root.card-row-count < 3 ? root.card-row-count : 3;",
        "flow-visible-height: root.visible-row-count * root.card-height + (root.visible-row-count - 1) * root.card-gap-y;",
        "flow-content-height: root.card-row-count * root.card-height + (root.card-row-count - 1) * root.card-gap-y;",
        "card-scroll := FlowScrollSurface {",
        "scroll-y <=> root.card-scroll-y;",
        "content-height: root.expanded ? root.flow-content-height : root.flow-visible-height;",
        "project: card;",
        "cover-height: HubVisualSpec.card-cover-height;",
        "flow-width: parent.width;",
        "card-basis: max(HubTokens.panel-min-sm * 2 / 3, min(HubTokens.panel-min-sm, root.flow-width * HubTokens.project-dashboard-card-ratio));",
        "dashboard-toolbar-search-basis: root.dashboard-toolbar-wrap ? root.content-width",
        "dashboard-toolbar-select-basis: root.dashboard-toolbar-wrap ? root.toolbar-control-min-width",
        "export component DashboardToolbar inherits Flow",
        "DashboardToolbar {",
        "basis: root.dashboard-toolbar-search-basis;",
        "basis: root.dashboard-toolbar-select-basis;",
        "min-width: root.select-basis;",
        "search-basis: root.dashboard-toolbar-search-basis;",
        "select-basis: root.dashboard-toolbar-select-basis;",
        "toolbar-wrap: root.dashboard-toolbar-wrap;",
        "search-query <=> root.search-query;",
        "search-projects(query) => { root.search-projects(query); }",
        "set-project-filter(id) => { root.set-project-filter(id); }",
        "set-project-sort(id) => { root.set-project-sort(id); }",
        "set-project-view-mode(mode) => { root.set-project-view-mode(mode); }",
        "show-project-subpage(page) => { root.show-project-subpage(page); }",
        "dashboard-table-width: max(root.toolbar-height * 8, root.content-width * HubTokens.project-dashboard-table-ratio);",
        "dashboard-main-basis: HubTokens.panel-min-lg + HubTokens.control-lg;",
        "dashboard-side-basis: HubTokens.panel-min-md + HubTokens.control-lg;",
        "quick-action-count: root.quick-actions.length;",
        "quick-action-row-gap: MaterialStyleMetrics.spacing_6;",
        "export component DashboardQuickActionRow inherits ActionRow",
        "component DashboardButtonStatesTitle",
        "component DashboardButtonStatesSectionLabel",
        "DashboardButtonStatesTitle {",
        "DashboardButtonStatesSectionLabel {",
        "export component DashboardRecentProjectsPanel inherits HubTableView",
        "export component DashboardQuickActionsPanel inherits HubListPanelSlot",
        "HubTableView,",
        "HubListPanelSlot,",
        "ActionRow,",
        "import { HubViewToggleGroup } from \"icon_button_components.slint\";",
        "FlowScrollSurface",
        "HubButtonStateTextSample,",
        "HubButtonStateIconSample,",
        "prominent: true;",
        "HubFlowNextButton,",
        "HubFlowNextButton {",
        "HubMoreMenuButton,",
        "HubMoreMenuButton {",
        "HubViewToggleGroup {",
        "selected-mode: root.project-view-mode;",
        "selected(mode) =>",
        "HubButtonStateTextSample {",
        "variant: \"primary\";",
        "variant: \"secondary\";",
        "variant: \"tertiary\";",
        "HubButtonStateIconSample {",
        "Build your project for development or release",
        "Deploy your project to a connected device",
        "Create a distributable package",
        "Launch the editor with a project",
        "action: root.quick-action-data;",
        "detail-override: root.visual-detail;",
        "compact-shell: true;",
        "plain-avatar: true;",
        "plain-trailing: true;",
        "leading-shell-size-override: MaterialStyleMetrics.size_24;",
        "leading-icon-size-override: MaterialStyleMetrics.size_24;",
        "activate(id) =>",
        "root.triggered(id);",
        "DashboardRecentProjectsPanel {",
        "body-padding: HubTokens.space-3;",
        "body-spacing: HubTokens.space-1;",
        "show-header: true;",
        "header-height: HubTokens.control-md * 2 / 3;",
        "title: root.panel-title;",
        "show-divider: false;",
        "minimum-row-height: root.table-row-height;",
        "HubPanelNavigationCommand {",
        "text: root.action-text;",
        "source-image: @image-url(\"../assets/icons/nav/projects.svg\");",
        "clicked => { root.view-all(); }",
        "DashboardQuickActionsPanel {",
        "body-spacing: HubTokens.space-2;",
        "project-rows: root.dashboard-project-rows;",
        "project-row-count: root.dashboard-project-row-count;",
        "quick-actions: root.quick-actions;",
        "quick-action-count: root.quick-action-count;",
        "quick-actions-scroll-y <=> root.quick-actions-scroll-y;",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "row-count: root.quick-action-count;",
        "row-height: root.quick-action-row-height;",
        "row-spacing: root.quick-action-row-gap;",
        "empty-height: HubTokens.list-row-lg;",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "quick-action-data: action;",
        "triggered(id) => { root.triggered(id); }",
        "if root.quick-action-count == 0: EmptyStateBlock {",
        "empty-title: root.ui-text.no-quick-actions;",
        "empty-detail: root.ui-text.quick-actions-empty-detail;",
        "title: root.empty-title;",
        "detail: root.empty-detail;",
        "center-content: true;",
        "basis: root.compact ? root.content-width : root.dashboard-main-basis;",
        "basis: root.compact ? root.content-width : root.dashboard-side-basis;",
        "grow: 2;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-md;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-sm;",
    ] {
        assert!(
            dashboard_surface.contains(snippet),
            "ProjectDashboardPage is missing dashboard Taffy sizing snippet: {snippet}"
        );
    }
    for component_name in [
        "ProjectCover",
        "ProjectCardIdentityStack",
        "ProjectCard",
        "ProjectFlow",
        "DashboardProjectCardsSection",
    ] {
        let declaration = if component_name == "ProjectCardIdentityStack" {
            format!("component {component_name}")
        } else {
            format!("export component {component_name}")
        };
        assert!(
            project_card_flow_components.contains(&declaration),
            "project_card_flow_components.slint should own the focused project-card-flow component {component_name}"
        );
        assert!(
            !dashboard_components.contains(&format!("export component {component_name}")),
            "project_dashboard_components.slint should not regain project card-flow ownership: {component_name}"
        );
        assert!(
            !dashboard.contains(&format!("component {component_name} inherits")),
            "project_dashboard.slint should import {component_name} instead of declaring it locally"
        );
    }
    for component_name in [
        "DashboardToolbar",
        "DashboardQuickActionRow",
        "DashboardRecentProjectsPanel",
        "DashboardQuickActionsPanel",
    ] {
        assert!(
            dashboard_components.contains(&format!("export component {component_name}")),
            "project_dashboard_components.slint should own the exported dashboard component {component_name}"
        );
        assert!(
            !dashboard.contains(&format!("component {component_name} inherits")),
            "project_dashboard.slint should import {component_name} instead of declaring it locally"
        );
    }
    let project_cover = project_card_flow_components
        .split("export component ProjectCover")
        .nth(1)
        .and_then(|source| source.split("export component ProjectCard").next())
        .expect("project_card_flow_components.slint must export ProjectCover before ProjectCard");
    assert!(
        !project_cover.contains("MaterialPalette.shadow_15"),
        "ProjectCover should render the reference project cover PNGs without an extra darkening overlay"
    );
    for snippet in [
        "HubDisclosureButton,",
        "} from \"button_components.slint\";",
        "HubFlowNextButton,",
        "HubMoreMenuButton,",
        "} from \"icon_button_components.slint\";",
        "HubMoreMenuButton {",
        "button-width: root.menu-width;",
        "button-height: root.menu-height;",
        "root.menu-clicked();",
    ] {
        assert!(
            project_card_flow_components.contains(snippet),
            "ProjectCover should route its cover menu through the shared more-menu button primitive: {snippet}"
        );
    }
    assert!(
        !project_cover.contains("StateLayerArea {")
            && !project_cover.contains("source: @image-url(\"../assets/icons/ui/more-vertical.svg\");")
            && !project_cover.contains("icon-image: @image-url(\"../assets/icons/ui/more-vertical.svg\");")
            && !project_cover.contains("has-icon-image: true;"),
        "ProjectCover should not reintroduce local menu icon binding after HubMoreMenuButton extraction"
    );
    for snippet in [
        "if root.project-card-count > root.card-column-count && !root.expanded: HubFlowNextButton {",
        "x: parent.width - self.width - MaterialStyleMetrics.size_2;",
        "y: root.card-height * 2 / 5;",
        "root.expanded = true;",
    ] {
        assert!(
            project_card_flow_components.contains(snippet),
            "ProjectFlow should route its collapsed next affordance through the shared HubFlowNextButton primitive: {snippet}"
        );
    }
    assert!(
        !project_card_flow_components.contains("component ProjectFlowNextButton")
            && !project_card_flow_components.lines().any(|line| line.trim() == "HubIconButton {"),
        "ProjectFlow should not reintroduce a page-local flow next button or raw HubIconButton after adopting HubFlowNextButton"
    );
    assert!(
        !project_card_flow_components.contains("card-scroll := ScrollView")
            && !project_card_flow_components.contains("ScrollView,"),
        "ProjectFlow should consume FlowScrollSurface instead of importing Material ScrollView directly"
    );
    for forbidden in [
        "DataTable {",
        "action-list := PanelListViewport {",
        "SearchBox {",
        "ProjectFilterSelect {",
        "ProjectSortSelect {",
        "ProjectFlow {",
        "PillButton {",
        "DashboardViewToggleButton {",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "if root.quick-action-count == 0: EmptyStateBlock {",
    ] {
        assert!(
            !dashboard.contains(forbidden),
            "ProjectDashboardPage should leave dashboard component internals inside focused component modules: {forbidden}"
        );
    }
    let dashboard_quick_action_row = dashboard_components
        .split("export component DashboardQuickActionRow")
        .nth(1)
        .and_then(|source| source.split("export component DashboardToolbar").next())
        .expect("project_dashboard_components.slint must declare DashboardQuickActionRow before DashboardToolbar");
    for forbidden in [
        "StateLayerArea {",
        "MaterialText {",
        "Image {",
        "HorizontalLayout {",
        "VerticalLayout {",
        "border-radius: HubVisualSpec.compact-radius;",
        "root.triggered(root.quick-action-data.id);",
    ] {
        assert!(
            !dashboard_quick_action_row.contains(forbidden),
            "DashboardQuickActionRow should specialize the shared ActionRow instead of repainting a local row: {forbidden}"
        );
    }
    assert!(
        !dashboard_components.contains("component DashboardViewToggleButton inherits")
            && !dashboard_components.contains("DashboardViewToggleButton {"),
        "Dashboard toolbar should consume HubViewToggleGroup instead of retaining a page-local DashboardViewToggleButton"
    );
    for forbidden in [
        "component DashboardStateButton inherits",
        "component DashboardTertiaryState inherits",
        "component DashboardStateIcon inherits",
    ] {
        assert!(
            !dashboard_components.contains(forbidden),
            "Dashboard Button States strip should consume button-family state samples instead of retaining page-local button primitives: {forbidden}"
        );
    }
    let dashboard_button_states_title = dashboard_components
        .split("component DashboardButtonStatesTitle")
        .nth(1)
        .and_then(|source| source.split("component DashboardButtonStatesSectionLabel").next())
        .expect(
            "project_dashboard_components.slint must declare DashboardButtonStatesTitle before DashboardButtonStatesSectionLabel",
        );
    let dashboard_button_states_section_label = dashboard_components
        .split("component DashboardButtonStatesSectionLabel")
        .nth(1)
        .and_then(|source| source.split("export component DashboardButtonStatesStrip").next())
        .expect(
            "project_dashboard_components.slint must declare DashboardButtonStatesSectionLabel before DashboardButtonStatesStrip",
        );
    let dashboard_button_states_strip = dashboard_components
        .split("export component DashboardButtonStatesStrip")
        .nth(1)
        .expect("project_dashboard_components.slint must export DashboardButtonStatesStrip");
    for snippet in [
        "inherits MaterialText",
        "text: root.title;",
        "style: MaterialTypography.title_medium;",
        "color: MaterialPalette.on_surface;",
    ] {
        assert!(
            dashboard_button_states_title.contains(snippet),
            "DashboardButtonStatesTitle must own the Button States title typography: {snippet}"
        );
    }
    for snippet in ["inherits MutedText", "text: root.label;"] {
        assert!(
            dashboard_button_states_section_label.contains(snippet),
            "DashboardButtonStatesSectionLabel must own the Button States section label typography: {snippet}"
        );
    }
    for snippet in [
        "DashboardButtonStatesTitle { title: \"Button States\"; }",
        "DashboardButtonStatesSectionLabel { label: \"Primary\"; }",
        "DashboardButtonStatesSectionLabel { label: \"Secondary\"; }",
        "DashboardButtonStatesSectionLabel { label: \"Tertiary\"; }",
        "DashboardButtonStatesSectionLabel { label: \"Icon\"; }",
        "HubButtonStateTextSample {",
        "HubButtonStateIconSample {",
    ] {
        assert!(
            dashboard_button_states_strip.contains(snippet),
            "DashboardButtonStatesStrip must compose shared label and button-state helpers: {snippet}"
        );
    }
    for forbidden in ["MaterialText {", "MutedText {"] {
        assert!(
            !dashboard_button_states_strip.contains(forbidden),
            "DashboardButtonStatesStrip should not recreate label typography after adopting label helpers: {forbidden}"
        );
    }
    for snippet in [
        "DashboardProjectCardsSection {",
        "expanded <=> root.project-cards-expanded;",
        "project-cards: root.project-cards;",
        "project-card-count: root.project-card-count;",
        "collapse-label: root.ui-text.collapse-projects;",
        "show-more-label: root.ui-text.show-more-projects;",
        "show-more-height: root.dashboard-show-more-height;",
        "select(path) => { root.select-project(path); }",
        "open(path) => { root.open-project-detail(path); }",
        "export component DashboardProjectCardsSection inherits VerticalLayout",
        "ProjectFlow {",
        "HubDisclosureButton {",
        "button-height: root.show-more-height;",
        "expanded: root.expanded;",
        "expanded-label: root.collapse-label;",
        "collapsed-label: root.show-more-label;",
        "toggled(expanded) => { root.expanded = expanded; }",
    ] {
        assert!(
            dashboard_surface.contains(snippet),
            "ProjectDashboardPage should route cards and Show More through DashboardProjectCardsSection: {snippet}"
        );
    }
    assert!(
        !project_card_flow_components.contains("PillButton {"),
        "DashboardProjectCardsSection should consume HubDisclosureButton instead of instantiating low-level PillButton directly"
    );
    assert!(
        !dashboard_surface.contains("dashboard-column-width"),
        "ProjectDashboardPage lower panels should not return to page-local remaining width formulas"
    );
    for forbidden in [
        "dashboard-toolbar-select-width",
        "root.content-width - root.toolbar-height",
        "root.content-width - root.page-gap * 3",
        "root.content-width / 4",
        "root.content-width / 9",
        "root.card-basis * 4 / 5",
        "root.content-width * 23 / 100",
        "root.available-content * 23 / 100",
        "available-content:",
        "card-height-token:",
        "dashboard-card-basis",
        "dashboard-card-height",
        "dashboard-flow-height",
        "root.content-width * 58 / 100",
        "page-surface.viewport-height * 3 / 5",
        "page-surface.viewport-height * 7 / 20",
    ] {
        assert!(
            !dashboard_surface.contains(forbidden),
            "ProjectDashboardPage should not return to toolbar/card remaining-width formulas: {forbidden}"
        );
    }
    assert!(
        app.contains("viewport-compact: responsive-state.compact;"),
        "HubWindow must pass viewport compact state from shared ResponsiveState into ProjectsPage"
    );
    assert!(
        projects.contains("in property <bool> viewport-compact: false;"),
        "ProjectsPage must expose viewport compact state for secondary project pages"
    );
    assert!(
        projects
            .matches("viewport-compact: root.viewport-compact;")
            .count()
            >= 3,
        "ProjectsPage must forward viewport compact state into ProjectNewPage, ProjectBrowserPage, and ProjectDetailPage"
    );
    for snippet in [
        "in property <bool> viewport-compact: false;",
        "compact-page: root.viewport-compact;",
        "page-pad: root.compact-page ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
    ] {
        assert!(
            project_new_page.contains(snippet)
                && browser_page.contains(snippet)
                && project_detail_page.contains(snippet),
            "ProjectNewPage, ProjectBrowserPage, and ProjectDetailPage must use viewport compact state for secondary-page spacing; missing {snippet}"
        );
    }
    for snippet in [
        "narrow-flow: root.content-width < HubTokens.panel-min-lg + HubTokens.panel-min-md + root.page-gap;",
        "flex-wrap: root.narrow-flow ? FlexboxLayoutWrap.wrap : FlexboxLayoutWrap.no-wrap;",
        "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
        "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-md;",
        "compact-page: root.viewport-compact;",
        "page-pad: root.compact-page ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
        "toolbar-search-basis: root.toolbar-wrap ? root.content-width : root.content-width * HubTokens.project-browser-toolbar-search-ratio;",
        "basis: root.toolbar-search-basis;",
        "toolbar-select-basis: root.toolbar-wrap ? root.toolbar-control-min-width",
        "basis: root.toolbar-select-basis;",
        "min-width: root.toolbar-select-basis;",
        "compact-page: root.viewport-compact;",
        "page-pad: root.compact-page ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
        "form-panel-height: HubTokens.space-4 * 2 + HubTokens.list-row-sm + root.field-height * 2 + root.engine-section-height + root.create-action-row-height + root.page-gap * 4;",
        "ProjectCreateSettingsPanel {",
        "export component ProjectCreateSettingsPanel inherits HubContentPanelSlot",
        "ProjectCreateCompactSummaryPanel {",
        "export component ProjectCreateCompactSummaryPanel inherits HubContentPanelSlot",
        "project-name <=> root.project-name;",
        "project-location <=> root.project-location;",
        "engine-scroll-y <=> root.new-engine-scroll-y;",
        "show-summary: !root.narrow-flow;",
        "browse-folder(kind) => { root.browse-folder(kind); }",
        "create-project => { root.create-project(); }",
        "select-engine(id) => { root.select-engine(id); }",
        "component ProjectCreateField inherits Rectangle",
        "ProjectCreateField {",
        "HubPathFieldRow {",
        "field-label: root.ui-text.project-name;",
        "field-text <=> root.project-name;",
        "field-label: root.ui-text.location;",
        "field-text <=> root.project-location;",
        "show-browse: true;",
        "root.browse-folder(\"new-project-location\");",
        "export component ProjectCreateActionRow inherits HubFormActionRow",
        "action-icon: @image-url(\"../assets/icons/ui/plus.svg\");",
        "ProjectCreateActionRow {",
        "row-height: root.create-action-row-height;",
        "row-spacing: root.page-gap;",
        "action-label: root.ui-text.create;",
        "action-enabled: root.form-ready;",
        "root.create-project();",
        "panel-padding: root.summary-panel-padding;",
        "summary-height: root.narrow-flow ? root.summary-section-height : 0px;",
        "body-spacing: 0px;",
        "summary-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "summary-section-height: root.summary-header-height + root.summary-row-height * 2 + root.page-gap;",
        "summary-panel-padding: root.narrow-flow ? HubTokens.space-3 : HubTokens.space-4;",
        "engine-panel-rows: root.engine-count < 1 ? 1 : (root.engine-count > 3 ? 3 : root.engine-count);",
        "engine-row-gap: MaterialStyleMetrics.spacing_8;",
        "engine-list-height: root.engine-count == 0 ? root.choice-row-height : root.engine-panel-rows * root.choice-row-height + (root.engine-panel-rows - 1) * root.engine-row-gap;",
        "new-engine-scroll-y: 0px;",
        "component ProjectEngineChoiceList inherits PanelListViewport",
        "in-out property <length> list-scroll-y: 0px;",
        "height: root.list-height;",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.engine-count;",
        "vertical-padding: 0px;",
        "for engine in root.engines: EngineChoiceRow {",
        "engine-selected(id) => { root.selected(id); }",
        "ProjectEngineChoiceList {",
        "list-height: root.engine-list-height;",
        "list-scroll-y <=> root.engine-scroll-y;",
        "row-count: root.engine-count;",
        "row-height: root.choice-row-height;",
        "empty-height: root.choice-row-height;",
        "empty-title: root.ui-text.register-source-engine-before-create;",
        "summary-panel-height: root.summary-panel-padding * 2 + root.summary-section-height;",
        "project-settings-panel-height: root.narrow-flow ? root.form-panel-height : root.form-panel-height + root.summary-section-height + root.page-gap;",
        "template-panel-rows: root.template-count < 1 ? 1 : (root.template-count > 4 ? 4 : root.template-count);",
        "template-list-height: root.template-count == 0 ? HubTokens.list-row-lg : root.template-panel-rows * root.template-row-height + (root.template-panel-rows - 1) * root.page-gap;",
        "template-panel-height: HubTokens.space-4 * 2 + HubTokens.control-md + root.template-list-height + root.page-gap;",
        "template-scroll-y: 0px;",
        "ProjectTemplateRailPanel {",
        "panel-title: root.ui-text.templates-title;",
        "templates: root.templates;",
        "template-count: root.template-count;",
        "list-height: root.template-list-height;",
        "list-scroll-y <=> root.template-scroll-y;",
        "row-count: root.template-count;",
        "row-height: root.row-height;",
        "row-height: root.template-row-height;",
        "row-spacing: root.page-gap;",
        "soon-label: root.ui-text.soon-label;",
        "export component ProjectTemplateRailPanel inherits HubListPanelSlot",
        "HubListPanelSlot,",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.template-count;",
        "empty-height: HubTokens.list-row-lg;",
        "for template in root.templates: TemplateChoiceRow",
        "template-selected(id) => { root.selected(id); }",
        "private property <CheckState> selection-state: root.template.selected ? CheckState.checked : CheckState.unchecked;",
        "export component TemplateChoiceRow inherits HubInteractiveRowSurface",
        "HubRowSelectionSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "check-state: root.selection-state;",
        "interaction-enabled: root.template.enabled;",
        "interaction-foreground: root.template.selected ? HubVisualSpec.accent-stroke : MaterialPalette.on_surface;",
        "clicked =>",
        "badge-text: root.trailing-label;",
        "flow-visible-height: max(root.content-height - root.header-height - root.page-gap, root.project-settings-panel-height);",
        "flow-height: root.narrow-flow ? root.project-settings-panel-height + root.page-gap + root.summary-panel-height + root.page-gap + root.template-panel-height : max(root.flow-visible-height, root.template-panel-height);",
        "height: root.choice-row-height;",
        "title: root.ui-text.register-source-engine-before-create;",
        "height: root.narrow-flow ? root.project-settings-panel-height : root.flow-height;",
        "height: root.narrow-flow ? root.summary-panel-height : 0px;",
        "height: root.narrow-flow ? root.template-panel-height : root.flow-height;",
        "browser-row-slot-height: root.browser-row-height + root.browser-row-gap;",
        "browser-table-header-height: HubTokens.control-md;",
        "browser-panel-chrome-height: HubTokens.control-md + root.browser-table-header-height + root.page-gap * 4;",
        "browser-fit-row-count: Math.floor(root.browser-available-list-height / root.browser-row-slot-height);",
        "browser-list-height: root.row-count == 0 ? HubTokens.list-row-lg : root.browser-panel-rows * root.browser-row-height + (root.browser-panel-rows - 1) * root.browser-row-gap;",
        "browser-panel-height: root.browser-panel-chrome-height + root.browser-list-height;",
        "ProjectBrowserResultsPanel {",
        "panel-height: root.browser-panel-height;",
        "list-height: root.browser-list-height;",
        "table-header-height: root.browser-table-header-height;",
        "panel-spacing: root.page-gap;",
        "list-scroll-y <=> root.browser-scroll-y;",
        "export component ProjectBrowserTableHeader inherits Rectangle",
        "export component ProjectBrowserResultsPanel inherits HubTableView",
        "ProjectBrowserTableHeader {",
        "browser-list := HubTableBody",
        "height: root.browser-list-height;",
        "height: root.list-height;",
        "empty-height: HubTokens.list-row-lg;",
        "if root.row-count == 0: EmptyStateBlock",
        "height: HubTokens.list-row-lg;",
        "title: root.ui-text.no-projects-match;",
        "body-padding: HubTokens.space-4;",
        "center-content: true;",
        "detail-panel-padding: root.narrow-flow ? HubTokens.space-3 : HubTokens.space-4;",
        "cover-height: root.narrow-flow ? HubTokens.list-row-lg",
        "detail-title-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "detail-info-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "detail-info-section-height: root.detail-info-header-height + root.info-row-height * 5 + root.detail-info-row-gap * 5;",
        "component ProjectDetailMainPanel inherits HubMediaContentPanelSlot",
        "media-height: root.cover-height;",
        "media-radius: HubVisualSpec.panel-radius;",
        "media-background: root.project.accent == 0 ? MaterialPalette.primary_container",
        "media-source: root.project.cover-image;",
        "has-media-source: root.project.has-cover;",
        "content-spacing: root.content-stack-spacing;",
        "ProjectDetailMainPanel {",
        "cover-height: root.cover-height;",
        "content-stack-spacing: root.page-gap;",
        "header-subtitle: root.narrow-flow ? \"\" : root.project.project-path;",
        "info-header-subtitle: root.narrow-flow ? \"\" : root.ui-text.project-info-subtitle;",
        "component ProjectDetailStatusStrip inherits HubBadgeMetaStrip",
        "in property <ProjectDetailData> detail;",
        "in property <UiTextData> copy;",
        "first-badge-text: root.detail.version;",
        "second-badge-text: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "meta-text: root.copy.modified-prefix + root.detail.modified;",
        "ProjectDetailStatusStrip {",
        "row-height: root.status-row-height;",
        "row-spacing: root.status-gap;",
        "version-badge-width: root.version-badge-width;",
        "pin-badge-width: root.pin-badge-width;",
        "detail: root.project;",
        "copy: root.ui-text;",
        "pin-toggle-row-height: max(HubTokens.list-row-sm, root.action-row-height);",
        "component ProjectDetailInfoSection inherits HubSection",
        "in property <ProjectDetailData> project;",
        "in property <UiTextData> ui-text;",
        "section-spacing: root.row-spacing;",
        "title: root.ui-text.project-info-title;",
        "ProjectDetailInfoSection {",
        "section-height: root.detail-info-section-height;",
        "row-height: root.info-row-height;",
        "row-spacing: root.detail-info-row-gap;",
        "header-subtitle: root.narrow-flow ? \"\" : root.ui-text.project-info-subtitle;",
        "project: root.project;",
        "ui-text: root.ui-text;",
        "detail-engine-list-height: root.engine-count == 0 ? HubTokens.list-row-lg : root.detail-engine-panel-rows * root.detail-choice-row-height + (root.detail-engine-panel-rows - 1) * root.detail-engine-row-gap;",
        "component ProjectDetailActionsSection inherits HubContentPanelSlot",
        "body-padding: root.panel-padding;",
        "body-spacing: root.panel-spacing;",
        "content-spacing: root.panel-spacing;",
        "ProjectDetailActionsSection {",
        "panel-padding: root.detail-panel-padding;",
        "panel-spacing: root.page-gap;",
        "project: root.project;",
        "copy: root.ui-text;",
        "engine-section-height: root.detail-engine-section-height;",
        "engine-list-height: root.detail-engine-list-height;",
        "engine-scroll-y <=> root.detail-engine-scroll-y;",
        "engine-row-height: root.detail-choice-row-height;",
        "engine-row-spacing: root.detail-engine-row-gap;",
        "collapse-engine-label: root.narrow-flow;",
        "HubActionCommandButton {",
        "source-image: @image-url(\"../assets/icons/nav/editor.svg\");",
        "source-image: @image-url(\"../assets/icons/ui/close.svg\");",
        "source-image: @image-url(\"../assets/icons/ui/alert.svg\");",
        "has-source-image: true;",
        "action-height: root.action-row-height;",
        "text: root.copy.confirm-delete;",
        "text: root.copy.cancel-delete;",
        "text: root.copy.open;",
        "text: root.copy.remove-from-hub;",
        "text: root.copy.delete-project;",
        "component ProjectDetailPinToggleRow inherits HubToggleRow",
        "checked: root.detail.pinned;",
        "label: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "supporting-text: root.detail.pinned ? root.copy.unpin-project : root.copy.pin-project;",
        "component ProjectDetailActionNote inherits MutedText",
        "text: root.note-text;",
        "height: root.note-height;",
        "ProjectDetailActionNote {",
        "note-height: root.note-height;",
        "note-text: root.copy.remove-from-hub-detail;",
        "ProjectDetailPinToggleRow {",
        "row-height: root.pin-toggle-row-height;",
        "toggled(checked) => { root.toggle-pin(); }",
        "list-height: root.engine-list-height;",
        "list-scroll-y <=> root.engine-scroll-y;",
        "row-height: root.engine-row-height;",
        "empty-height: HubTokens.list-row-lg;",
        "component ProjectDetailEngineSection inherits HubSection",
        "in property <ProjectDetailData> detail;",
        "in property <UiTextData> copy;",
        "title: root.copy.change-source-engine;",
        "subtitle: root.copy.bound-source-engine + \": \" + root.detail.engine-label;",
        "selected-label: root.copy.selected-label;",
        "registered-label: root.copy.registered;",
        "empty-title: root.copy.no-source-engine-available;",
        "ProjectDetailEngineSection {",
        "section-height: root.engine-section-height;",
        "section-spacing: root.panel-spacing;",
        "detail: root.project;",
        "copy: root.copy;",
        "list-scroll-y <=> root.engine-scroll-y;",
        "selected(id) => { root.select-engine(id); }",
        "detail-main-panel-height: root.detail-panel-padding * 2 + root.cover-height + root.detail-title-header-height + root.status-row-height + root.detail-info-section-height + root.page-gap * 3;",
        "detail-action-panel-height: root.detail-panel-padding * 2 + (root.project.pending-delete ? root.detail-action-delete-height : root.detail-action-standard-height);",
        "detail-visible-height: max(root.content-height - root.detail-header-height - root.page-gap, root.detail-main-panel-height);",
        "detail-flow-height: root.narrow-flow ? root.detail-main-height + root.page-gap + root.detail-action-height : max(root.detail-main-height, root.detail-action-height);",
        "scroll-y <=> root.browser-scroll-y;",
    ] {
        assert!(
            project_surface.contains(snippet),
            "Project secondary pages are missing ResponsiveSlot/scroll sizing snippet: {snippet}"
        );
    }
    let page_header = project_components
        .split("export component PageHeader")
        .nth(1)
        .and_then(|source| source.split("export component EngineChoiceRow").next())
        .expect("project_page_components.slint must declare PageHeader before EngineChoiceRow");
    let page_header_title_stack = project_components
        .split("component PageHeaderTitleStack")
        .nth(1)
        .and_then(|source| source.split("export component PageHeader").next())
        .expect(
            "project_page_components.slint must declare PageHeaderTitleStack before PageHeader",
        );
    for snippet in [
        "import { HubFormActionRow } from \"button_components.slint\";",
        "import { HubBackButton } from \"icon_button_components.slint\";",
        "HubBackButton {",
        "button-size: root.back-size;",
        "clicked => { root.back(); }",
        "PageHeaderTitleStack {",
        "stack-height: parent.height;",
        "title: root.title;",
        "subtitle: root.subtitle;",
        "stack-spacing: MaterialStyleMetrics.spacing_2;",
    ] {
        assert!(
            project_components.contains(snippet) || page_header.contains(snippet),
            "PageHeader must render its secondary-page back affordance and text lane through shared/focused helpers: {snippet}"
        );
    }
    for snippet in [
        "MaterialText {",
        "in property <length> stack-height: MaterialStyleMetrics.size_48;",
        "height: root.stack-height;",
        "text: root.title;",
        "style: MaterialTypography.title_large;",
        "MutedText {",
        "text: root.subtitle;",
    ] {
        assert!(
            page_header_title_stack.contains(snippet),
            "PageHeaderTitleStack must own Projects secondary-page header text styling: {snippet}"
        );
    }
    assert!(
        project_components.contains("component PageHeaderTitleStack inherits Rectangle"),
        "project_page_components.slint must keep PageHeaderTitleStack as a private focused helper"
    );
    assert!(
        !page_header.lines().any(|line| line.trim() == "IconButton {")
            && !page_header.lines().any(|line| line.trim() == "HubIconButton {"),
        "PageHeader should not return to generic IconButton or raw HubIconButton after adopting HubBackButton"
    );
    for forbidden in ["MaterialText {", "MutedText {"] {
        assert!(
            !page_header.contains(forbidden),
            "PageHeader should not recreate title/subtitle text after adopting PageHeaderTitleStack: {forbidden}"
        );
    }
    for component_name in [
        "ProjectCreateField",
        "ProjectCreateActionRow",
        "ProjectEngineChoiceList",
        "ProjectCreateSettingsPanel",
        "ProjectCreateCompactSummaryPanel",
        "ProjectTemplateRailPanel",
    ] {
        assert!(
            project_components.contains(&format!("export component {component_name}")),
            "project_page_components.slint should own the exported shared Projects workflow component {component_name}"
        );
        assert!(
            !project_pages.contains(&format!("component {component_name} inherits"))
                && !project_detail_page.contains(&format!("component {component_name} inherits")),
            "project_pages.slint and project_detail_page.slint should import {component_name} instead of declaring it locally"
        );
    }
    let project_create_field = project_components
        .split("export component ProjectCreateField")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("project_page_components.slint must declare ProjectCreateField");
    for snippet in [
        "HubPathFieldRow {",
        "label: root.field-label;",
        "placeholder: root.field-placeholder;",
        "text <=> root.field-text;",
        "field-height: root.field-height;",
        "show-action: root.show-browse;",
        "action-label: root.browse-label;",
        "action-clicked =>",
    ] {
        assert!(
            project_create_field.contains(snippet),
            "ProjectCreateField must consume the shared HubPathFieldRow primitive; missing {snippet}"
        );
    }
    for forbidden in ["HubTextField {", "PillButton {", "HorizontalLayout {"] {
        assert!(
            !project_create_field.contains(forbidden),
            "ProjectCreateField must not keep page-local field/action layout after HubPathFieldRow extraction: {forbidden}"
        );
    }
    for component_name in [
        "ProjectDetailPinToggleRow",
        "ProjectDetailActionsSection",
        "ProjectDetailStatusStrip",
        "ProjectDetailInfoSection",
        "ProjectDetailEngineSection",
        "ProjectDetailMainPanel",
    ] {
        assert!(
            project_detail_components.contains(&format!("export component {component_name}")),
            "project_detail_components.slint should own the exported Project Detail component {component_name}"
        );
        assert!(
            !project_pages.contains(&format!("component {component_name} inherits"))
                && !project_components.contains(&format!("export component {component_name}"))
                && !project_detail_page.contains(&format!("component {component_name} inherits")),
            "Project Detail components should live in project_detail_components.slint and be imported by the page"
        );
    }
    assert!(
        !project_detail_components.contains("export component ProjectDetailActionButton")
            && !project_detail_components.contains("ProjectDetailActionButton {"),
        "Project Detail command actions should consume HubActionCommandButton directly instead of keeping a pass-through wrapper"
    );
    for component_name in [
        "ProjectFilterSelect",
        "ProjectSortSelect",
        "ProjectBrowserTableHeader",
        "ProjectBrowserRow",
        "ProjectBrowserResultsPanel",
    ] {
        assert!(
            project_browser_components.contains(&format!("export component {component_name}")),
            "project_browser_components.slint should own the exported Project Browser component {component_name}"
        );
        assert!(
            project_components.contains(&format!("{component_name},"))
                && project_components.contains("} from \"project_browser_components.slint\";"),
            "project_page_components.slint should re-export {component_name} from project_browser_components.slint"
        );
        assert!(
            !project_components.contains(&format!("component {component_name} inherits"))
                && !project_pages.contains(&format!("component {component_name} inherits"))
                && !project_detail_components
                    .contains(&format!("component {component_name} inherits"))
                && !project_detail_page.contains(&format!("component {component_name} inherits"))
                && !project_browser_page.contains(&format!("component {component_name} inherits")),
            "Project Browser components should not be declared in project_page_components.slint, project_pages.slint, project_detail_components.slint, project_detail_page.slint, or project_browser_page.slint"
        );
    }
    assert!(
        project_pages
            .contains("export { ProjectBrowserPage } from \"project_browser_page.slint\";"),
        "project_pages.slint should re-export ProjectBrowserPage from its dedicated page module"
    );
    assert!(
        project_pages.contains("export { ProjectNewPage } from \"project_new_page.slint\";"),
        "project_pages.slint should re-export ProjectNewPage from its dedicated page module"
    );
    assert!(
        project_pages.contains("export { ProjectDetailPage } from \"project_detail_page.slint\";"),
        "project_pages.slint should re-export ProjectDetailPage from its dedicated page module"
    );
    assert!(
        !project_pages.contains("export component ProjectBrowserPage inherits"),
        "project_pages.slint should not keep the ProjectBrowserPage implementation inline"
    );
    assert!(
        !project_pages.contains("export component ProjectNewPage inherits"),
        "project_pages.slint should not keep the ProjectNewPage implementation inline"
    );
    assert!(
        !project_pages.contains("export component ProjectDetailPage inherits"),
        "project_pages.slint should not keep the ProjectDetailPage implementation inline"
    );
    for forbidden in [
        "column-width",
        "toolbar-select-width",
        "root.content-width - root.page-gap",
        "page-surface.viewport-height - root.page-pad - root.page-pad",
        "page-surface.viewport-height - root.page-pad * 2",
        "page-surface.viewport-height - root.page-pad - root.page-pad - root.action-row-height",
        "page-surface.content-height",
        "page-surface.viewport-height",
        "viewport_y <=> root.scroll-y;",
        "scroll-y <=> root.page-scroll-y;",
        "root.content-width / 8",
        "root.content-width * 2 / 5",
        "root.flow-height * 3 / 5",
        "root.flow-height * 2 / 5",
        "root.content-height / 4",
        "root.content-width / 10",
        "root.content-width / 14",
        "root.content-width / 2",
        "root.content-width / 4",
        "root.content-height * 3 / 5",
        "root.content-height * 2 / 5",
        "if root.engine-count > 0: PanelListViewport",
        "template-list := PanelListViewport",
        "for template in root.templates: TemplateChoiceRow",
        "detail-engine-list := PanelListViewport",
        "HubTextField {\n                                height: root.field-height;\n                                label: root.ui-text.project-name;",
        "if root.project.pending-delete: PillButton",
        "if !root.project.pending-delete: PillButton",
        "Badge { text: root.project.version; tone: \"accent\"; badge-width: root.version-badge-width; }",
        "root.project.pinned ? root.ui-text.pinned-label : root.ui-text.not-pinned-label",
        "text: root.ui-text.modified-prefix + root.project.modified;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.project-status;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.project-root-path;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.source-engine;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.engine-version-column;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.last-modified-column;",
        "export component ProjectCreateActionRow inherits HubFormActionRow",
        "if !root.project.pending-delete: VerticalLayout {\n                        horizontal-stretch: 1;\n                        height: root.detail-engine-section-height;",
        "title: root.ui-text.change-source-engine;\n                            subtitle: root.ui-text.bound-source-engine + \": \" + root.project.engine-label;",
        "browser-list := HubTableBody",
        "if root.row-count == 0: EmptyStateBlock",
        "PanelHeader {\n                                title: root.ui-text.project-settings-title;",
        "ProjectCreateField {",
        "ProjectCreateActionRow {",
        "ProjectEngineChoiceList {",
    ] {
        assert!(
            !project_pages.contains(forbidden)
                && !project_new_page.contains(forbidden)
                && !project_browser_page.contains(forbidden)
                && !project_detail_page.contains(forbidden),
            "Project secondary pages should not return to page-local remaining width/height formulas: {forbidden}"
        );
    }
    assert_eq!(
        project_new_page
            .matches("ProjectCreateSettingsPanel {")
            .count(),
        1,
        "ProjectNewPage should route settings form and Source Engine choices through ProjectCreateSettingsPanel directly"
    );
    assert_eq!(
        project_new_page
            .matches("ProjectCreateCompactSummaryPanel {")
            .count(),
        1,
        "ProjectNewPage should route its compact summary through ProjectCreateCompactSummaryPanel directly"
    );
    assert_eq!(
        project_new_page.matches("ProjectEngineChoiceList {").count(),
        0,
        "ProjectNewPage should leave Source Engine choice internals inside ProjectCreateSettingsPanel"
    );
    assert_eq!(
        project_new_page
            .matches("ProjectTemplateRailPanel {")
            .count(),
        1,
        "ProjectNewPage should route template choices through ProjectTemplateRailPanel directly"
    );
    assert_eq!(
        project_browser_page.matches("ProjectBrowserResultsPanel {").count(),
        1,
        "ProjectBrowserPage should route the results panel through ProjectBrowserResultsPanel directly"
    );
    assert!(
        !project_browser_page.contains("PanelHeader {")
            && !project_browser_page.contains("PanelListViewport {")
            && !project_browser_page.contains("ProjectBrowserRow {")
            && !project_browser_page.contains("EmptyStateBlock {"),
        "ProjectBrowserPage should keep result-list internals in project_browser_components.slint"
    );
    assert_eq!(
        project_detail_components
            .matches("ProjectEngineChoiceList {")
            .count(),
        1,
        "ProjectDetailEngineSection should reuse ProjectEngineChoiceList for detail engine choices"
    );
    assert_eq!(
        project_detail_components
            .matches("HubActionCommandButton {")
            .count(),
        5,
        "ProjectDetailActionsSection should route command actions directly through HubActionCommandButton and reserve pin state for ProjectDetailPinToggleRow"
    );
    assert_eq!(
        project_detail_components
            .matches("ProjectDetailPinToggleRow {")
            .count(),
        1,
        "ProjectDetailActionsSection should route pin/unpin through one ProjectDetailPinToggleRow"
    );
    assert_eq!(
        project_detail_page
            .matches("ProjectDetailActionsSection {")
            .count(),
        1,
        "ProjectDetailPage should route the actions column through one ProjectDetailActionsSection"
    );
    assert_eq!(
        project_detail_page
            .matches("ProjectDetailMainPanel {")
            .count(),
        1,
        "ProjectDetailPage should route the main media/status/info column through one ProjectDetailMainPanel"
    );
    assert_eq!(
        project_detail_components
            .matches("ProjectDetailInfoSection {")
            .count(),
        1,
        "ProjectDetailMainPanel should route the five project-info rows through one ProjectDetailInfoSection"
    );
    assert_eq!(
        project_detail_components
            .matches("ProjectDetailStatusStrip {")
            .count(),
        1,
        "ProjectDetailMainPanel should route version, pinned state, and modified time through one ProjectDetailStatusStrip"
    );
    assert_eq!(
        project_new_page.matches("ProjectCreateActionRow {").count(),
        0,
        "ProjectNewPage should leave its create button row inside ProjectCreateSettingsPanel"
    );
    assert_eq!(
        project_detail_components
            .matches("ProjectDetailEngineSection {")
            .count(),
        1,
        "ProjectDetailActionsSection should route Change Source Engine controls through one ProjectDetailEngineSection"
    );
    let project_detail_action_stack = project_detail_components
        .split("export component ProjectDetailActionStack")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailDeleteActionStack").next())
        .expect(
            "project_detail_components.slint must declare ProjectDetailActionStack before ProjectDetailDeleteActionStack",
        );
    for forbidden in [
        "MutedText {",
        "        text: root.copy.remove-from-hub-detail;",
    ] {
        assert!(
            !project_detail_action_stack.contains(forbidden),
            "ProjectDetailActionStack should route remove-from-Hub supporting copy through ProjectDetailActionNote: {forbidden}"
        );
    }
}
