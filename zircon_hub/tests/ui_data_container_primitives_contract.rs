//! Static contracts for Hub Material list, table, and tree container primitives.

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

#[test]
fn components_reexport_data_container_primitives() {
    let components = read_ui_file("components.slint");
    for snippet in [
        "TreeItemData,",
        "HubListPanelSlot,",
        "HubListView,",
        "HubTreeRow,",
        "HubTreeView,",
        "} from \"data_display.slint\";",
        "HubTableView,",
        "HubTableBody,",
        "} from \"table_view_components.slint\";",
    ] {
        assert!(
            components.contains(snippet),
            "components.slint must re-export data container primitive {snippet}"
        );
    }
}

#[test]
fn list_table_and_tree_views_compose_material_scroll_and_list_tiles() {
    let data_display = read_ui_file("data_display.slint");
    let table_view = read_ui_file("table_view_components.slint");
    let data_surface = format!("{data_display}\n{table_view}");
    for snippet in [
        "export struct TreeItemData",
        "depth: int,",
        "has-children: bool,",
        "trailing-tone: string,",
        "export component HubListView inherits HubPanel",
        "PanelListViewport {",
        "scroll-y <=> root.scroll-y;",
        "row-count: root.row-count;",
        "export component HubListPanelSlot inherits PanelSlot",
        "PanelHeader {",
        "height: root.header-height;",
        "export component HubTableView inherits HubPanel",
        "in property <length> basis: HubTokens.panel-min-md;",
        "in property <float> grow: 1;",
        "in property <float> shrink: 1;",
        "in property <int> order: 0;",
        "in property <length> header-height: HubTokens.control-md;",
        "height: root.header-height;",
        "minimum-row-height: HubVisualSpec.visual-table-row-height;",
        "export component HubTableBody inherits PanelListViewport",
        "row-height: HubVisualSpec.visual-table-row-height;",
        "export component HubTreeRow inherits Rectangle",
        "private property <length> depth-indent: root.item.depth * HubTokens.space-4;",
        "StateLayerArea {",
        "ListTile {",
        "source: root.item.expanded ? @image-url(\"../assets/icons/ui/chevron-down.svg\") : @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "StatusBadge {",
        "export component HubTreeView inherits HubPanel",
        "for item in root.items: HubTreeRow",
        "toggle-expanded(id) =>",
    ] {
        assert!(
            data_surface.contains(snippet),
            "data-display and table-view modules must keep list/table/tree containers on shared Material-backed primitives; missing {snippet}"
        );
    }

    for wrapper_name in [
        "HubListView",
        "HubListPanelSlot",
        "HubTableView",
        "HubTableBody",
        "HubTreeRow",
        "HubTreeView",
    ] {
        let wrapper_source = if wrapper_name.contains("Table") {
            &table_view
        } else {
            &data_display
        };
        let wrapper = wrapper_source
            .split(&format!("export component {wrapper_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("focused data container module must declare {wrapper_name}"));
        for forbidden in ["TouchArea", "area.has-hover", "LineEdit"] {
            assert!(
                !wrapper.contains(forbidden),
                "{wrapper_name} must not reintroduce hand-rolled data container behavior: {forbidden}"
            );
        }
    }
}

#[test]
fn cloud_and_team_lower_lists_consume_hub_list_panel_slot() {
    let cloud_components = read_ui_file("cloud_page_components.slint");
    let team_components = read_ui_file("team_page_components.slint");

    for (name, source, component, scroll_binding, title_binding) in [
        (
            "CloudServicesPanel",
            &cloud_components,
            "export component CloudServicesPanel inherits HubListPanelSlot",
            "scroll-y <=> root.service-scroll-y;",
            "title: root.ui-text.cloud-services;",
        ),
        (
            "TeamMembersPanel",
            &team_components,
            "export component TeamMembersPanel inherits HubListPanelSlot",
            "scroll-y <=> root.member-scroll-y;",
            "title: root.ui-text.team-members-found;",
        ),
    ] {
        for snippet in [
            "HubListPanelSlot,",
            component,
            "body-padding: MaterialStyleMetrics.padding_16;",
            "body-spacing: HubTokens.toolbar-gap;",
            title_binding,
            "show-badge: true;",
            scroll_binding,
            "empty-height: HubTokens.list-row-lg + HubTokens.space-4;",
            "EmptyStateBlock {",
        ] {
            assert!(
                source.contains(snippet),
                "{name} must consume HubListPanelSlot while preserving title, badge, scroll, and empty-state bindings; missing {snippet}"
            );
        }

        let panel = source
            .split(component)
            .nth(1)
            .and_then(|body| body.split("\n}").next())
            .unwrap_or_else(|| panic!("{name} must be declared in its page component module"));
        for forbidden in ["PanelHeader {", "PanelListViewport {", "inherits PanelSlot"] {
            assert!(
                !panel.contains(forbidden),
                "{name} should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
            );
        }
    }
}

#[test]
fn settings_lower_lists_consume_hub_list_panel_slot() {
    let settings_components = read_ui_file("settings_page_components.slint");

    for (name, component, scroll_binding, row_count, row_height, row_marker) in [
        (
            "SettingsDefaultPathsPanel",
            "export component SettingsDefaultPathsPanel inherits HubListPanelSlot",
            "scroll-y <=> root.paths-scroll-y;",
            "row-count: 4;",
            "row-height: HubTokens.input-field;",
            "PathSettingRow {",
        ),
        (
            "SettingsConfigurationHealthPanel",
            "export component SettingsConfigurationHealthPanel inherits HubListPanelSlot",
            "scroll-y <=> root.health-scroll-y;",
            "row-count: root.settings-status-count;",
            "row-height: HubTokens.list-row-md;",
            "SettingStatusRow {",
        ),
    ] {
        for snippet in [
            "HubListPanelSlot,",
            component,
            "title: root.panel-title;",
            scroll_binding,
            row_count,
            row_height,
            "row-spacing: HubTokens.space-2;",
            row_marker,
        ] {
            assert!(
                settings_components.contains(snippet),
                "{name} must consume HubListPanelSlot while preserving title, scroll, row, and content bindings; missing {snippet}"
            );
        }

        let panel = settings_components
            .split(component)
            .nth(1)
            .and_then(|body| body.split("\n}").next())
            .unwrap_or_else(|| panic!("{name} must be declared in settings_page_components.slint"));
        for forbidden in ["PanelHeader {", "PanelListViewport {", "inherits PanelSlot"] {
            assert!(
                !panel.contains(forbidden),
                "{name} should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
            );
        }
    }

    for snippet in [
        "empty-height: root.health-empty-height;",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-configuration-checks;",
        "detail: root.ui-text.configuration-health-empty-detail;",
        "center-content: true;",
    ] {
        assert!(
            settings_components.contains(snippet),
            "SettingsConfigurationHealthPanel must preserve the shared empty-state copy and sizing: {snippet}"
        );
    }
}

#[test]
fn project_template_rail_consumes_hub_list_panel_slot() {
    let project_components = read_ui_file("project_page_components.slint");

    for snippet in [
        "HubListPanelSlot,",
        "export component ProjectTemplateRailPanel inherits HubListPanelSlot",
        "title: root.panel-title;",
        "show-badge: true;",
        "badge-text: root.template-count + \"\";",
        "body-spacing: root.row-spacing;",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.template-count;",
        "row-height: root.row-height;",
        "empty-height: HubTokens.list-row-lg;",
        "for template in root.templates: TemplateChoiceRow {",
        "selected(id) => { root.selected(id); }",
    ] {
        assert!(
            project_components.contains(snippet),
            "ProjectTemplateRailPanel must consume HubListPanelSlot while preserving template row bindings; missing {snippet}"
        );
    }

    let panel = project_components
        .split("export component ProjectTemplateRailPanel inherits HubListPanelSlot")
        .nth(1)
        .expect("ProjectTemplateRailPanel must be declared in project_page_components.slint");
    for forbidden in [
        "template-list := PanelListViewport",
        "PanelHeader {",
        "inherits PanelSlot",
    ] {
        assert!(
            !panel.contains(forbidden),
            "ProjectTemplateRailPanel should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
        );
    }
}

#[test]
fn dashboard_quick_actions_consumes_hub_list_panel_slot() {
    let dashboard_components = read_ui_file("project_dashboard_components.slint");

    for snippet in [
        "HubListPanelSlot,",
        "export component DashboardQuickActionsPanel inherits HubListPanelSlot",
        "header-height: HubTokens.control-md * 2 / 3;",
        "title: root.panel-title;",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "row-count: root.quick-action-count;",
        "row-height: root.quick-action-row-height;",
        "row-spacing: root.quick-action-row-gap;",
        "empty-height: HubTokens.list-row-lg;",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "quick-action-data: action;",
        "if root.quick-action-count == 0: EmptyStateBlock",
    ] {
        assert!(
            dashboard_components.contains(snippet),
            "DashboardQuickActionsPanel must consume HubListPanelSlot while preserving quick action row bindings; missing {snippet}"
        );
    }

    let panel = dashboard_components
        .split("export component DashboardQuickActionsPanel inherits HubListPanelSlot")
        .nth(1)
        .expect(
            "DashboardQuickActionsPanel must be declared in project_dashboard_components.slint",
        );
    for forbidden in [
        "action-list := PanelListViewport",
        "PanelHeader {",
        "inherits PanelSlot",
    ] {
        assert!(
            !panel.contains(forbidden),
            "DashboardQuickActionsPanel should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
        );
    }
}

#[test]
fn catalog_page_consumes_shared_hub_list_view() {
    let data_display = read_ui_file("data_display.slint");
    let assets = read_ui_file("assets.slint");
    let plugins = read_ui_file("plugins.slint");
    let learn = read_ui_file("learn.slint");

    for snippet in [
        "export component CatalogListPanel inherits HubListView",
        "show-header: true;",
        "vertical-padding: HubTokens.space-1;",
        "empty-height: root.row-height + HubTokens.space-4;",
        "if root.row-count == 0: EmptyStateBlock",
    ] {
        assert!(
            data_display.contains(snippet),
            "CatalogListPanel must consume HubListView while preserving catalog list chrome: {snippet}"
        );
    }

    for (page, source, row) in [
        ("assets.slint", assets, "for asset in root.assets: AssetRow"),
        (
            "plugins.slint",
            plugins,
            "for plugin in root.plugins: PluginRow",
        ),
        (
            "learn.slint",
            learn,
            "for resource in root.resources: LearnRow",
        ),
    ] {
        assert!(
            source.contains("inherits CatalogPage") && source.contains(row),
            "{page} must continue routing catalog rows through CatalogPage after the list-view refactor"
        );
    }
}

#[test]
fn dashboard_recent_table_consumes_shared_hub_table_body() {
    let data_display = read_ui_file("data_display.slint");
    let table_view = read_ui_file("table_view_components.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let data_table = table_view
        .split("export component DataTable")
        .nth(1)
        .and_then(|source| source.split("export component HubTableView").next())
        .expect("table_view_components.slint must declare DataTable before HubTableView");

    for snippet in [
        "table-scroll := HubTableBody {",
        "scroll-y <=> root.scroll-y;",
        "row-count: root.row-count;",
        "row-height: root.row-height;",
        "row-spacing: root.row-gap;",
        "vertical-padding: root.row-gap * 2;",
        "empty-height: root.row-height + root.row-gap * 4;",
        "for row in root.rows: ProjectTableRow",
        "if root.row-count == 0: EmptyStateBlock",
    ] {
        assert!(
            data_table.contains(snippet),
            "DataTable must consume HubTableBody for the Dashboard Recent Projects table body; missing {snippet}"
        );
    }

    for forbidden in [
        "table-scroll := ScrollView",
        "table-content-height:",
        "viewport_width: table-scroll.visible_width",
        "viewport_height: max(table-scroll.visible_height",
    ] {
        assert!(
            !data_table.contains(forbidden),
            "DataTable should not keep a page-local table scroll implementation after moving to HubTableBody: {forbidden}"
        );
    }
    assert!(
        !data_display.contains("export component DataTable")
            && !data_display.contains("export component HubTableView")
            && !data_display.contains("export component HubTableBody"),
        "data_display.slint should not regain table-view ownership after the table module split"
    );

    for snippet in [
        "HubTableView,",
        "export component DashboardRecentProjectsPanel inherits HubTableView",
        "body-padding: HubTokens.space-3;",
        "body-spacing: HubTokens.space-1;",
        "show-header: true;",
        "header-height: HubTokens.control-md * 2 / 3;",
        "title: root.panel-title;",
        "show-divider: false;",
        "minimum-row-height: root.table-row-height;",
        "DataTable {",
    ] {
        assert!(
            dashboard_components.contains(snippet),
            "DashboardRecentProjectsPanel must consume HubTableView while DataTable owns the shared HubTableBody bridge; missing {snippet}"
        );
    }

    let recent_panel = dashboard_components
        .split("export component DashboardRecentProjectsPanel inherits HubTableView")
        .nth(1)
        .and_then(|source| {
            source
                .split("export component DashboardQuickActionsPanel")
                .next()
        })
        .expect("DashboardRecentProjectsPanel must be declared before DashboardQuickActionsPanel");
    for forbidden in ["PanelHeader {", "inherits PanelSlot", "HubTableBody {"] {
        assert!(
            !recent_panel.contains(forbidden),
            "DashboardRecentProjectsPanel should not reintroduce a local panel/table shell after moving to HubTableView: {forbidden}"
        );
    }
}

#[test]
fn editor_side_lists_consume_shared_hub_list_view() {
    let editor_components = read_ui_file("editor_page_components.slint");

    for snippet in [
        "HubListView,",
        "export component EditorSideListPanel inherits HubListView",
        "title: root.panel-title;",
        "badge-tone: \"info\";",
        "show-header: true;",
        "body-spacing: HubTokens.toolbar-gap;",
        "scroll-y <=> root.list-scroll-y;",
        "export component EditorSourceEngineListPanel inherits EditorSideListPanel",
        "export component EditorBuildHistoryPanel inherits EditorSideListPanel",
    ] {
        assert!(
            editor_components.contains(snippet),
            "Editor source/build-history side lists must consume HubListView instead of hand-building a second list shell; missing {snippet}"
        );
    }

    let side_panel = editor_components
        .split("export component EditorSideListPanel")
        .nth(1)
        .and_then(|source| {
            source
                .split("export component EditorSourceEngineListPanel")
                .next()
        })
        .expect(
            "editor_page_components.slint must declare EditorSideListPanel before typed side lists",
        );
    for forbidden in ["HubPanel", "PanelListViewport", "VerticalLayout {"] {
        assert!(
            !side_panel.contains(forbidden),
            "EditorSideListPanel should not return to a local panel/list shell after moving to HubListView: {forbidden}"
        );
    }
}

#[test]
fn builds_action_and_pipeline_lists_consume_hub_list_panel_slot() {
    let builds_components = read_ui_file("builds_page_components.slint");

    for (name, component, row_count, row_marker) in [
        (
            "BuildControlsPanel",
            "export component BuildControlsPanel inherits HubListPanelSlot",
            "row-count: 5;",
            "BuildControlAction {",
        ),
        (
            "BuildPipelinePanel",
            "export component BuildPipelinePanel inherits HubListPanelSlot",
            "row-count: 4;",
            "BuildPipelineStep {",
        ),
    ] {
        for snippet in [
            "HubListPanelSlot,",
            component,
            "row-height: root.row-height;",
            "row-spacing: HubTokens.toolbar-gap;",
            row_count,
            row_marker,
        ] {
            assert!(
                builds_components.contains(snippet),
                "{name} must consume HubListPanelSlot while preserving row sizing and delegates; missing {snippet}"
            );
        }

        let panel = builds_components
            .split(component)
            .nth(1)
            .and_then(|body| body.split("\nexport component ").next())
            .unwrap_or_else(|| panic!("{name} must be declared in builds_page_components.slint"));
        for forbidden in ["PanelHeader {", "PanelListViewport {", "inherits PanelSlot"] {
            assert!(
                !panel.contains(forbidden),
                "{name} should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
            );
        }
    }
}

#[test]
fn build_task_history_uses_hub_section_for_internal_sections() {
    let builds_components = read_ui_file("builds_page_components.slint");
    let task_panel = builds_components
        .split("export component BuildTaskHistoryPanel")
        .nth(1)
        .expect("builds_page_components.slint must declare BuildTaskHistoryPanel");

    for snippet in [
        "HubSection,",
        "HubSection {",
        "private property <length> current-task-section-height:",
        "private property <length> history-list-empty-height:",
        "private property <length> history-list-height:",
        "private property <length> history-section-height:",
        "section-height: root.current-task-section-height;",
        "section-height: root.history-section-height;",
        "section-spacing: HubTokens.panel-gap;",
        "title: root.current-task-title;",
        "title: root.build-history-title;",
        "PanelListViewport {",
        "scroll-y <=> root.history-scroll-y;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
    ] {
        assert!(
            builds_components.contains(snippet),
            "BuildTaskHistoryPanel must use HubSection for current-task/history sections while preserving the history list: missing {snippet}"
        );
    }

    assert_eq!(
        task_panel.matches("HubSection {").count(),
        2,
        "BuildTaskHistoryPanel should expose exactly two internal HubSection blocks"
    );
    assert!(
        !task_panel.contains("PanelHeader {"),
        "BuildTaskHistoryPanel should not hand-build current-task/history section headers after moving to HubSection"
    );
}

#[test]
fn editor_and_cloud_action_panels_consume_hub_list_panel_slot() {
    let editor_components = read_ui_file("editor_page_components.slint");
    let cloud_components = read_ui_file("cloud_page_components.slint");

    for (name, source, component, row_count, row_marker) in [
        (
            "EditorActionsPanel",
            &editor_components,
            "export component EditorActionsPanel inherits HubListPanelSlot",
            "row-count: 4;",
            "EditorActionRow {",
        ),
        (
            "CloudPackageActionsPanel",
            &cloud_components,
            "export component CloudPackageActionsPanel inherits HubListPanelSlot",
            "row-count: 2;",
            "CloudPackageActionRow {",
        ),
    ] {
        for snippet in [
            "HubListPanelSlot,",
            component,
            row_count,
            "row-spacing: HubTokens.toolbar-gap;",
            row_marker,
        ] {
            assert!(
                source.contains(snippet),
                "{name} must consume HubListPanelSlot while preserving action rows; missing {snippet}"
            );
        }

        let panel = source
            .split(component)
            .nth(1)
            .and_then(|body| body.split("\nexport component ").next())
            .unwrap_or_else(|| panic!("{name} must be declared in its page component module"));
        for forbidden in ["PanelHeader {", "PanelListViewport {", "inherits PanelSlot"] {
            assert!(
                !panel.contains(forbidden),
                "{name} should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
            );
        }
    }
}

#[test]
fn project_engine_choices_consume_panel_list_viewport_directly() {
    let project_components = read_ui_file("project_page_components.slint");
    let engine_list = project_components
        .split("export component ProjectEngineChoiceList")
        .nth(1)
        .and_then(|source| source.split("export component ProjectCreateSettingsPanel").next())
        .expect("project_page_components.slint must declare ProjectEngineChoiceList before ProjectCreateSettingsPanel");

    for snippet in [
        "export component ProjectEngineChoiceList inherits PanelListViewport",
        "height: root.list-height;",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.engine-count;",
        "vertical-padding: 0px;",
        "if root.engine-count == 0: EmptyStateBlock",
        "for engine in root.engines: EngineChoiceRow",
        "EngineChoiceRow {\n        row-height: root.row-height;",
    ] {
        assert!(
            project_components.contains(snippet),
            "ProjectEngineChoiceList must expose the shared list viewport directly; missing {snippet}"
        );
    }

    for forbidden in ["engine-list := PanelListViewport", "inherits Rectangle"] {
        assert!(
            !engine_list.contains(forbidden),
            "ProjectEngineChoiceList should not keep a local Rectangle/list wrapper after direct viewport migration: {forbidden}"
        );
    }
}
