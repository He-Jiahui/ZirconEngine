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

    for snippet in [
        "ScrollView,",
        "table-scroll := ScrollView {",
        "catalog-scroll := ScrollView {",
        "export component PanelListViewport inherits ScrollView",
        "export component CatalogPage inherits PageScrollSurface",
        "min-height: root.row-height * 4;",
        "row-slot-height: root.row-height + HubTokens.space-2;",
        "panel-chrome-height: HubTokens.space-4 * 2 + HubTokens.control-md + HubTokens.toolbar-gap;",
        "fit-row-count: Math.floor(root.fit-list-height / root.row-slot-height);",
        "panel-height: root.row-count > root.visible-row-count ? root.fitted-panel-height : root.content-height;",
        "root.row-count == 0 ? root.row-height + HubTokens.space-4",
        "table-content-height: root.row-count == 0 ? root.row-height + root.row-gap * 4 : root.row-count * root.row-height + (root.row-count - 1) * root.row-gap + root.row-gap * 4;",
        "catalog-content-height: root.row-count == 0 ? root.row-height + HubTokens.space-4 : root.row-count * root.row-height + (root.row-count - 1) * HubTokens.space-2 + HubTokens.space-2;",
        "list-content-height: root.row-count == 0 ? root.empty-height : root.row-count * root.row-height + (root.row-count - 1) * root.row-spacing + root.vertical-padding * 2;",
        "height: max(root.row-height + HubTokens.space-2, catalog-scroll.visible_height);",
        "Rectangle { vertical-stretch: 1; min-height: 0px; }",
        "viewport_y <=> root.scroll-y;",
        "viewport_width: table-scroll.visible_width;",
        "viewport_width: catalog-scroll.visible_width;",
        "viewport_width: root.visible_width;",
        "viewport_height: max(table-scroll.visible_height, root.table-content-height);",
        "viewport_height: max(catalog-scroll.visible_height, root.catalog-content-height);",
        "viewport_height: max(root.visible_height, root.list-content-height);",
        "vertical_scrollbar_policy: ScrollBarPolicy.as-needed;",
        "horizontal_scrollbar_policy: ScrollBarPolicy.always-off;",
    ] {
        assert!(
            data_display.contains(snippet),
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
        "root.row-count * (root.row-height + root.row-gap) + root.row-gap * 4",
        "root.row-count * (root.row-height + HubTokens.space-2) + HubTokens.space-2",
        "root.row-count * (root.row-height + root.row-spacing) + root.vertical-padding * 2",
        "root.height - HubTokens.page-padding * 2",
        "page-surface := PageScrollSurface",
        "width: root.width;",
        "height: root.height;",
    ] {
        assert!(
            !data_display.contains(forbidden),
            "Data-display list/table scrolling should not return to the std-widgets ScrollView surface: {forbidden}"
        );
    }
}

#[test]
fn data_display_table_text_uses_material_text() {
    let data_display = read_ui_file("data_display.slint");
    let table_header = data_display
        .split("export component TableColumnHeader")
        .nth(1)
        .and_then(|source| source.split("export component ProjectTableRow").next())
        .expect("data_display.slint must declare TableColumnHeader before ProjectTableRow");
    let table_row = data_display
        .split("export component ProjectTableRow")
        .nth(1)
        .and_then(|source| source.split("export component DataTable").next())
        .expect("data_display.slint must declare ProjectTableRow before DataTable");
    let data_table = data_display
        .split("export component DataTable")
        .nth(1)
        .and_then(|source| source.split("export component CatalogListPanel").next())
        .expect("data_display.slint must declare DataTable before CatalogListPanel");

    for snippet in [
        "MaterialText,",
        "style: MaterialTypography.label_medium;",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            data_display.contains(snippet),
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
    ] {
        assert!(
            !info_row.contains(forbidden),
            "InfoRow should not return to a custom painted information row: {forbidden}"
        );
    }
}

#[test]
fn catalog_rows_opt_into_compact_trailing_labels() {
    for page in ["assets.slint", "plugins.slint", "learn.slint"] {
        let source = read_ui_file(page);
        for snippet in [
            "in property <bool> collapse-label: false;",
            "collapse-trailing-label: root.collapse-label;",
            "collapse-label: root.content-width < HubTokens.breakpoint-medium;",
        ] {
            assert!(
                source.contains(snippet),
                "{page} catalog rows should drive compact trailing-label behavior from the page width instead of squeezing body copy at narrow widths or deriving layout from row width; missing {snippet}"
            );
        }
    }

    let learn = read_ui_file("learn.slint");
    assert!(
        learn.contains("show-arrow: true;"),
        "Learn rows should keep their compact arrow affordance after the category badge collapses"
    );
}

#[test]
fn action_row_uses_material_list_tile() {
    let data_display = read_ui_file("data_display.slint");
    let action_row = data_display
        .split("export component ActionRow")
        .nth(1)
        .and_then(|source| source.split("export component TableColumnHeader").next())
        .expect("data_display.slint must declare ActionRow before TableColumnHeader");

    for snippet in [
        "row-height: HubTokens.list-row-md;",
        "ListTile {",
        "text: root.action.title;",
        "supporting_text: root.action.detail;",
        "avatar_icon:",
        "avatar_background:",
        "avatar_foreground:",
        "clicked =>",
        "IconButton {",
        "chevron-right.svg",
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
    let builds = read_ui_file("builds.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export component BuildHistoryRow inherits InfoRow",
        "in property <SourceBuildHistoryRowData> record;",
        "in property <bool> collapse-label: false;",
        "title: root.record.detail;",
        "detail: root.record.output-path;",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            data_display.contains(snippet),
            "BuildHistoryRow must be a shared Material ListTile-backed data-display row; missing {snippet}"
        );
    }

    assert!(
        editor.contains("BuildHistoryRow,") && !editor.contains("component BuildHistoryRow"),
        "EditorPage should reuse the shared BuildHistoryRow instead of owning a page-local row"
    );
    for snippet in [
        "in property <[SourceBuildHistoryRowData]> source-build-history;",
        "in property <int> source-build-history-count;",
        "PanelListViewport {",
        "for record in root.source-build-history: BuildHistoryRow",
        "collapse-label: root.compact-labels;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-build-history;",
    ] {
        assert!(
            builds.contains(snippet),
            "BuildsPage must surface selected-project build-context history; missing {snippet}"
        );
    }
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
