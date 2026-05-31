//! Static contracts for Zircon Hub data-display primitives.

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
fn data_display_lists_use_material_scroll_view() {
    let data_display = read_ui_file("data_display.slint");
    let table_view = read_ui_file("table_view_components.slint");
    let data_surface = format!("{data_display}\n{table_view}");

    for snippet in [
        "ScrollView,",
        "table-scroll := HubTableBody {",
        "export component PanelListViewport inherits ScrollView",
        "export component HubTableBody inherits PanelListViewport",
        "export component CatalogPage inherits PageScrollSurface",
        "min-height: root.row-height * 4;",
        "row-slot-height: root.row-height + HubTokens.space-2;",
        "panel-chrome-height: HubTokens.space-4 * 2 + HubTokens.control-md + HubTokens.toolbar-gap;",
        "fit-row-count: Math.floor(root.fit-list-height / root.row-slot-height);",
        "panel-height: root.row-count > root.visible-row-count ? root.fitted-panel-height : root.content-height;",
        "empty-height: root.row-height + HubTokens.space-4;",
        "list-content-height: root.row-count == 0 ? root.empty-height : root.row-count * root.row-height + (root.row-count - 1) * root.row-spacing + root.vertical-padding * 2;",
        "row-spacing: root.row-gap;",
        "vertical-padding: root.row-gap * 2;",
        "empty-height: root.row-height + root.row-gap * 4;",
        "height: max(root.row-height + HubTokens.space-2, parent.height);",
        "Rectangle { vertical-stretch: 1; min-height: 0px; }",
        "viewport_y <=> root.scroll-y;",
        "scroll-y <=> root.scroll-y;",
        "row-count: root.row-count;",
        "viewport_width: root.visible_width;",
        "viewport_height: max(root.visible_height, root.list-content-height);",
        "vertical_scrollbar_policy: ScrollBarPolicy.as-needed;",
        "horizontal_scrollbar_policy: ScrollBarPolicy.always-off;",
    ] {
        assert!(
            data_surface.contains(snippet),
            "Data-display list and table surfaces must use the Material ScrollView API; missing {snippet}"
        );
    }

    for forbidden in [
        "std-widgets.slint",
        "mouse-drag-pan-enabled",
        "viewport-y <=>",
        "visible-width",
        "min-height: HubTokens.list-row-lg * 4",
        "panel-height: max(root.row-height * 4, root.content-height)",
        "panel-height: max(HubTokens.list-row-lg * 4, root.content-height)",
        "panel-height: max(HubTokens.list-row-lg * 4, root.height - HubTokens.page-padding * 2)",
        "table-scroll := ScrollView {",
        "catalog-scroll := ScrollView {",
        "table-content-height:",
        "catalog-content-height:",
        "viewport_width: table-scroll.visible_width;",
        "viewport_width: catalog-scroll.visible_width;",
        "viewport_height: max(table-scroll.visible_height, root.table-content-height);",
        "viewport_height: max(catalog-scroll.visible_height, root.catalog-content-height);",
        "root.row-count * (root.row-height + root.row-gap) + root.row-gap * 4",
        "root.row-count * (root.row-height + HubTokens.space-2) + HubTokens.space-2",
        "root.row-count * (root.row-height + root.row-spacing) + root.vertical-padding * 2",
        "root.height - HubTokens.page-padding * 2",
        "page-surface := PageScrollSurface",
        "width: root.width;",
        "height: root.height;",
    ] {
        assert!(
            !data_surface.contains(forbidden),
            "Data-display list/table scrolling should not return to the std-widgets ScrollView surface: {forbidden}"
        );
    }

    for table_component in [
        "TableColumnHeader",
        "ProjectTableRow",
        "HubTableBody",
        "DataTable",
        "HubTableView",
    ] {
        assert!(
            !data_display.contains(&format!("export component {table_component}")),
            "data_display.slint should not regain table-view ownership after the table module split: {table_component}"
        );
    }
}

#[test]
fn data_display_table_text_uses_material_text() {
    let table_view = read_ui_file("table_view_components.slint");
    let table_header = table_view
        .split("export component TableColumnHeader")
        .nth(1)
        .and_then(|source| source.split("export component ProjectTableRow").next())
        .expect(
            "table_view_components.slint must declare TableColumnHeader before ProjectTableRow",
        );
    let table_row = table_view
        .split("export component ProjectTableRow")
        .nth(1)
        .and_then(|source| source.split("export component DataTable").next())
        .expect("table_view_components.slint must declare ProjectTableRow before DataTable");
    let data_table = table_view
        .split("export component DataTable")
        .nth(1)
        .and_then(|source| source.split("export component HubTableView").next())
        .expect("table_view_components.slint must declare DataTable before HubTableView");

    for snippet in [
        "MaterialText,",
        "style: MaterialTypography.label_medium;",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            table_view.contains(snippet),
            "DataTable and ProjectTableRow typography should delegate metrics to MaterialText; missing {snippet}"
        );
    }

    for (name, source) in [
        ("TableColumnHeader", table_header),
        ("ProjectTableRow", table_row),
    ] {
        assert!(
            !source.lines().any(|line| line.trim() == "Text {"),
            "{name} should not return to raw Text nodes for table typography"
        );
        for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
            assert!(
                !source.contains(forbidden),
                "{name} should not return to raw Text font bindings: {forbidden}"
            );
        }
    }

    for snippet in [
        "if root.row-count == 0: EmptyStateBlock",
        "title: root.empty-text;",
        "center-content: true;",
    ] {
        assert!(
            data_table.contains(snippet),
            "DataTable empty state should reuse EmptyStateBlock instead of page-local muted text: {snippet}"
        );
    }

    for snippet in [
        "visible-modified: root.project.modified == \"1d ago\" ? \"Yesterday\" : root.project.modified;",
        "text: root.visible-modified;",
    ] {
        assert!(
            table_row.contains(snippet),
            "ProjectTableRow should normalize reference fixture labels at the presentation edge: {snippet}"
        );
    }
}

#[test]
fn data_display_catalog_empty_state_uses_material_text() {
    let data_display = read_ui_file("data_display.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let catalog_panel = data_display
        .split("export component CatalogListPanel")
        .nth(1)
        .and_then(|source| source.split("export component CatalogPage").next())
        .expect("data_display.slint must declare CatalogListPanel before CatalogPage");

    for snippet in [
        "if root.row-count == 0: EmptyStateBlock",
        "title: root.empty-title;",
        "detail: root.empty-detail;",
        "body-padding: MaterialStyleMetrics.padding_14;",
        "center-content: true;",
    ] {
        assert!(
            catalog_panel.contains(snippet),
            "CatalogListPanel empty state should route through EmptyStateBlock; missing {snippet}"
        );
    }

    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");
    for snippet in [
        "MaterialText {",
        "text: root.title;",
        "style: root.title-prominent ? MaterialTypography.title_medium : MaterialTypography.label_large;",
        "if root.detail != \"\": MutedText",
    ] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should own MaterialText title typography; missing {snippet}"
        );
    }

    assert!(
        !catalog_panel.lines().any(|line| line.trim() == "Text {")
            && !empty_block.lines().any(|line| line.trim() == "Text {"),
        "CatalogListPanel empty state should not return to a raw Text title"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !catalog_panel.contains(forbidden) && !empty_block.contains(forbidden),
            "CatalogListPanel empty state should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn info_row_uses_material_list_tile() {
    let data_display = read_ui_file("data_display.slint");
    let info_row = data_display
        .split("export component InfoRow")
        .nth(1)
        .and_then(|source| source.split("export component ActionRow").next())
        .expect("data_display.slint must declare InfoRow before ActionRow");

    for snippet in [
        "ListTile {",
        "text: root.title;",
        "supporting_text: root.supporting-text;",
        "avatar_icon:",
        "avatar_background:",
        "avatar_foreground:",
        "in property <bool> collapse-trailing-label: false;",
        "in property <bool> enabled: true;",
        "in property <bool> selected: false;",
        "in property <bool> focused: false;",
        "in property <bool> pressed: false;",
        "in property <bool> hovered: false;",
        "in property <length> idle-border-width: HubTokens.border-width;",
        "in property <length> selected-border-width: HubTokens.border-width;",
        "in property <color> idle-background: HubVisualSpec.panel-background;",
        "in property <color> selected-background: MaterialPalette.secondary_container;",
        "in property <color> enabled-avatar-background: root.avatar-background;",
        "in property <color> enabled-avatar-foreground: root.avatar-foreground;",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : (root.selected ? root.selected-border-width : root.idle-border-width);",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : (root.selected ? HubVisualSpec.accent-stroke : HubVisualSpec.outline-muted);",
        "background: !root.enabled ? HubVisualSpec.panel-background.with_alpha(HubVisualSpec.disabled-opacity) : (root.selected ? root.selected-background : root.idle-background);",
        "root.collapse-trailing-label ? (root.show-arrow ? HubTokens.control-md : 0px)",
        "enabled: root.enabled;",
        "clicked =>",
        "if root.show-trailing-badge && !root.collapse-trailing-label: StatusBadge",
        "IconButton {",
    ] {
        assert!(
            info_row.contains(snippet),
            "InfoRow must delegate its information-row body to Material ListTile and collapse optional trailing labels on compact rows; missing {snippet}"
        );
    }

    for forbidden in [
        "area := TouchArea",
        "border-color: area.has-hover",
        "background: area.has-hover",
        "preferred-width: HubTokens.panel-min-sm;",
        "avatar_background: root.enabled ? root.avatar-background",
        "avatar_foreground: root.enabled ? root.avatar-foreground",
    ] {
        assert!(
            !info_row.contains(forbidden),
            "InfoRow should not return to a custom painted information row: {forbidden}"
        );
    }

    let action_row = data_display
        .split("export component ActionRow")
        .nth(1)
        .and_then(|source| source.split("export component MetricCard").next())
        .expect("data_display.slint must declare ActionRow before MetricCard");
    for snippet in [
        "in property <bool> focused: false;",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : HubTokens.border-width;",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : HubVisualSpec.outline-muted;",
    ] {
        assert!(
            action_row.contains(snippet),
            "ActionRow must expose shared focus-ring primitive state; missing {snippet}"
        );
    }
}

#[test]
fn catalog_rows_opt_into_compact_trailing_labels() {
    let catalog_components = read_ui_file("catalog_page_components.slint");
    for (page, row_component) in [
        ("assets.slint", "AssetRow"),
        ("plugins.slint", "PluginRow"),
        ("learn.slint", "LearnRow"),
    ] {
        let source = read_ui_file(page);
        let surface = format!("{source}\n{catalog_components}");
        for snippet in [
            "in property <bool> collapse-label: false;",
            "collapse-trailing-label: root.collapse-label;",
            "label-collapse := ResponsiveCollapse {",
            "content-width: root.content-width;",
            "collapse-at: HubTokens.breakpoint-medium;",
            "collapse-label: label-collapse.collapsed;",
        ] {
            assert!(
                surface.contains(snippet),
                "{page} catalog rows should drive compact trailing-label behavior from the page width instead of squeezing body copy at narrow widths or deriving layout from row width; missing {snippet}"
            );
        }
        assert!(
            catalog_components.contains(&format!("export component {row_component}")),
            "catalog_page_components.slint should export {row_component}"
        );
        assert!(
            !source.contains(&format!("component {row_component}")),
            "{page} should import {row_component} instead of defining it inline"
        );
    }

    let learn = read_ui_file("learn.slint");
    let learn_surface = format!("{learn}\n{catalog_components}");
    assert!(
        learn_surface.contains("show-arrow: true;"),
        "Learn rows should keep their compact arrow affordance after the category badge collapses"
    );
}

#[test]
fn action_row_uses_material_list_tile() {
    let data_display = read_ui_file("data_display.slint");
    let action_row = data_display
        .split("export component ActionRow")
        .nth(1)
        .and_then(|source| source.split("export component MetricCard").next())
        .expect("data_display.slint must declare ActionRow before MetricCard");

    for snippet in [
        "row-height: HubTokens.list-row-md;",
        "in property <bool> plain-avatar: false;",
        "in property <bool> plain-trailing: false;",
        "in property <bool> show-trailing: true;",
        "in property <bool> compact-shell: false;",
        "in property <float> disabled-shell-opacity: 1.0;",
        "in property <color> enabled-avatar-background: MaterialPalette.primary_container;",
        "in property <color> enabled-avatar-foreground: MaterialPalette.primary;",
        "row-corner-radius: root.compact-shell ? HubVisualSpec.compact-radius : HubVisualSpec.panel-radius;",
        "action-avatar-background: root.plain-avatar ? transparent :",
        "action-avatar-foreground: root.plain-avatar ? MaterialPalette.on_surface :",
        "border-radius: root.row-corner-radius;",
        "background: root.action.enabled ? HubVisualSpec.panel-background : HubVisualSpec.panel-background.with_alpha(root.disabled-shell-opacity);",
        "ListTile {",
        "text: root.action.title;",
        "supporting_text: root.action.detail;",
        "avatar_icon:",
        "avatar_background: root.action-avatar-background;",
        "avatar_foreground: root.action-avatar-foreground;",
        "clicked =>",
        "width: root.show-trailing ? root.trailing-size : 0px;",
        "IconButton {",
        "chevron-right.svg",
        "if root.show-trailing && !root.plain-trailing: IconButton {",
        "if root.show-trailing && root.plain-trailing: Rectangle {",
    ] {
        assert!(
            action_row.contains(snippet),
            "ActionRow must delegate its operation-row body to Material ListTile; missing {snippet}"
        );
    }

    for forbidden in [
        "CenteredIcon",
        "area := TouchArea",
        "border-color: area.has-hover",
        "background: area.has-hover",
    ] {
        assert!(
            !action_row.contains(forbidden),
            "ActionRow should not return to a custom painted operation row: {forbidden}"
        );
    }
}

#[test]
fn build_history_rows_are_shared_between_editor_and_builds() {
    let data_display = read_ui_file("data_display.slint");
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let builds = read_ui_file("builds.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export component BuildHistoryRow inherits InfoRow",
        "in property <SourceBuildHistoryRowData> record;",
        "in property <bool> collapse-label: false;",
        "title: root.record.detail;",
        "detail: root.record.log != \"\" ? root.record.log : root.record.output-path;",
        "meta: root.record.process-id != \"\" ? root.record.process-id + \" / \" + root.record.profile + \" / \" + root.record.finished : root.record.profile + \" / \" + root.record.finished;",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            data_display.contains(snippet),
            "BuildHistoryRow must be a shared Material ListTile-backed data-display row; missing {snippet}"
        );
    }

    assert!(
        editor_components.contains("BuildHistoryRow,")
            && editor_components.contains("for record in root.source-build-history: BuildHistoryRow")
            && !editor.contains("BuildHistoryRow,")
            && !editor.contains("for record in root.source-build-history: BuildHistoryRow"),
        "EditorPage should reuse the shared BuildHistoryRow through EditorBuildHistoryPanel instead of owning a page-local row loop"
    );
    let builds_components = read_ui_file("builds_page_components.slint");
    let builds_surface = format!("{builds}\n{builds_components}");
    for snippet in [
        "in property <[SourceBuildHistoryRowData]> source-build-history;",
        "in property <int> source-build-history-count;",
        "PanelListViewport {",
        "for record in root.source-build-history: BuildHistoryRow",
        "collapse-label: root.compact-labels;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
        "no-build-history-title: root.ui-text.no-build-history;",
        "title: root.no-build-history-title;",
    ] {
        assert!(
            builds_surface.contains(snippet),
            "BuildsPage must surface selected-project build-context history; missing {snippet}"
        );
    }
    assert!(
        builds.contains("BuildTaskHistoryPanel {")
            && !builds.contains("for record in root.source-build-history: BuildHistoryRow"),
        "BuildsPage should pass build-history state into BuildTaskHistoryPanel instead of owning list rows inline"
    );
    for snippet in [
        "source-build-history: root.source-build-history;",
        "source-build-history-count: root.source-build-history-count;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must forward build history rows into BuildsPage; missing {snippet}"
        );
    }
}

#[test]
fn hub_window_exposes_operation_timeline_rows_for_runtime_binding() {
    let shared = read_ui_file("shared.slint");
    let operation_timeline = read_ui_file("operation_timeline_components.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export struct OperationTimelineRowData",
        "action: string,",
        "status: string,",
        "finished: string,",
        "target: string,",
        "detail: string,",
        "log: string,",
        "recovery: string,",
        "command: string,",
        "output-path: string,",
        "process-id: string,",
        "success: bool,",
    ] {
        assert!(
            shared.contains(snippet),
            "Operation timeline rows must keep a Slint struct matching the Rust view-model projection; missing {snippet}"
        );
    }

    for snippet in [
        "export component OperationTimelineRow inherits InfoRow",
        "in property <OperationTimelineRowData> record;",
        "title: root.record.action + \" / \" + root.record.target;",
        "detail: root.record.recovery != \"\" ? root.record.recovery : (root.record.log != \"\" ? root.record.log : (root.record.command != \"\" ? root.record.command : root.record.detail));",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            operation_timeline.contains(snippet),
            "Operation timeline rows must stay available as shared Material ListTile-backed rows; missing {snippet}"
        );
    }

    for snippet in [
        "OperationTimelineRowData,",
        "in property <[OperationTimelineRowData]> operation-timeline;",
        "in property <int> operation-timeline-count;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must expose operation timeline rows for Rust binding setters; missing {snippet}"
        );
    }
}

#[test]
fn operation_timeline_rows_are_shared_between_builds_and_settings() {
    let data_display = read_ui_file("data_display.slint");
    let operation_timeline = read_ui_file("operation_timeline_components.slint");
    let components = read_ui_file("components.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let builds = read_ui_file("builds.slint");
    let settings = read_ui_file("settings.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export component OperationTimelineRow inherits InfoRow",
        "in property <OperationTimelineRowData> record;",
        "title: root.record.action + \" / \" + root.record.target;",
        "detail: root.record.recovery != \"\" ? root.record.recovery : (root.record.log != \"\" ? root.record.log : (root.record.command != \"\" ? root.record.command : root.record.detail));",
        "meta: root.record.process-id != \"\" ? root.record.process-id + \" / \" + root.record.finished : root.record.finished;",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            operation_timeline.contains(snippet),
            "OperationTimelineRow must be a shared Material ListTile-backed operation-timeline row; missing {snippet}"
        );
    }

    for snippet in [
        "export component OperationTimelinePanel inherits PanelSlot",
        "in property <[OperationTimelineRowData]> operation-timeline;",
        "in property <int> operation-timeline-count;",
        "for record in root.operation-timeline: OperationTimelineRow",
        "if root.operation-timeline-count == 0: EmptyStateBlock",
        "detail: root.empty-detail;",
    ] {
        assert!(
            operation_timeline.contains(snippet),
            "OperationTimelinePanel should own the shared timeline viewport and empty state; missing {snippet}"
        );
    }
    assert!(
        components.contains("OperationTimelinePanel,"),
        "components.slint should re-export the shared OperationTimelinePanel from operation_timeline_components.slint"
    );
    assert!(
        !data_display.contains("OperationTimelinePanel")
            && !data_display.contains("OperationTimelineRow")
            && !data_display.contains("OperationTimelineRowData"),
        "data_display.slint should not regain operation-timeline ownership after the timeline module split"
    );
    assert!(
        !builds_components.contains("OperationTimelinePanel")
            && !builds_components.contains("OperationTimelineRow"),
        "builds_page_components.slint should not own the shared operation timeline panel or row"
    );
    assert!(
        !settings.contains("from \"builds_page_components.slint\""),
        "SettingsPage should import the shared OperationTimelinePanel through components.slint, not the Builds module"
    );

    for source in [&builds, &settings] {
        for snippet in [
            "OperationTimelinePanel {",
            "timeline-title: root.ui-text.operation-timeline;",
            "empty-title: root.ui-text.no-operation-timeline;",
            "empty-detail: root.ui-text.operation-timeline-empty-detail;",
            "operation-timeline: root.operation-timeline;",
            "operation-timeline-count: root.operation-timeline-count;",
        ] {
            assert!(
                source.contains(snippet),
                "Builds and Settings should both consume the shared operation timeline panel; missing {snippet}"
            );
        }
    }

    for snippet in [
        "in property <[OperationTimelineRowData]> operation-timeline;",
        "in property <int> operation-timeline-count;",
        "operation-timeline: root.operation-timeline;",
        "operation-timeline-count: root.operation-timeline-count;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow should forward operation timeline rows into Builds and Settings; missing {snippet}"
        );
    }
}
