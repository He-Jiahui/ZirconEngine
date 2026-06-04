//! Static contracts for Hub table-view primitives.

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
fn table_header_and_body_text_use_shared_material_primitives() {
    let table_view = read_ui_file("table_view_components.slint");
    let table_header = table_view
        .split("export component TableColumnHeader")
        .nth(1)
        .and_then(|source| source.split("export component TableCellText").next())
        .expect("table_view_components.slint must declare TableColumnHeader before TableCellText");
    let table_cell = table_view
        .split("export component TableCellText")
        .nth(1)
        .and_then(|source| source.split("component ProjectTableNameCell").next())
        .expect(
            "table_view_components.slint must declare TableCellText before ProjectTableNameCell",
        );
    let table_name_cell = table_view
        .split("component ProjectTableNameCell")
        .nth(1)
        .and_then(|source| source.split("export component ProjectTableRow").next())
        .expect(
            "table_view_components.slint must declare ProjectTableNameCell before ProjectTableRow",
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
        "horizontal_alignment: left;",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            table_view.contains(snippet),
            "Table view typography should delegate metrics to MaterialText-backed primitives; missing {snippet}"
        );
    }

    for (name, source) in [
        ("TableColumnHeader", table_header),
        ("TableCellText", table_cell),
        ("ProjectTableNameCell", table_name_cell),
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

    assert!(
        table_view.contains("export component ProjectTableRow inherits HubInteractiveRowSurface"),
        "ProjectTableRow should inherit the shared interactive row surface"
    );
    assert!(
        table_view.contains("component ProjectTableNameCell inherits Rectangle"),
        "ProjectTableNameCell should stay a private focused helper for the table row name lane"
    );
    for snippet in [
        "in property <RecentProjectRowData> project;",
        "in property <color> title-foreground: MaterialPalette.on_surface;",
        "private property <color> thumbnail-outline: root.show-thumbnail-accent ? HubVisualSpec.accent-stroke : MaterialPalette.on_surface.with_alpha(0.08);",
        "border-color: root.thumbnail-outline;",
        "source: root.project.cover-image;",
        "source: @image-url(\"../assets/brand/zircon-mark.svg\");",
        "MaterialText {",
        "text: root.project.title;",
        "color: root.title-foreground;",
        "style: MaterialTypography.label_large;",
        "vertical_alignment: center;",
    ] {
        assert!(
            table_name_cell.contains(snippet),
            "ProjectTableNameCell must own the table title/thumbnail lane: {snippet}"
        );
    }
    for snippet in [
        "ProjectTableNameCell {",
        "project: root.project;",
        "row-height: root.row-height;",
        "content-gap: root.content-gap;",
        "thumbnail-size: root.thumbnail-size;",
        "thumbnail-radius: root.thumbnail-radius;",
        "name-min-width: root.name-min-width;",
        "show-thumbnail-accent: root.show-thumbnail-accent;",
        "title-foreground: root.selection-visible ? MaterialPalette.on_primary_container : MaterialPalette.on_surface;",
    ] {
        assert!(
            table_row.contains(snippet),
            "ProjectTableRow must delegate custom title/thumbnail content to ProjectTableNameCell: {snippet}"
        );
    }
    for snippet in [
        "export component TableCellText inherits Rectangle",
        "in property <string> text;",
        "in property <color> foreground: MaterialPalette.on_surface_variant;",
        "style: MaterialTypography.body_small;",
        "horizontal_alignment: left;",
        "vertical_alignment: center;",
    ] {
        assert!(
            table_cell.contains(snippet) || table_view.contains(snippet),
            "TableCellText must own shared body table-cell typography: {snippet}"
        );
    }
    for snippet in [
        "TableCellText {",
        "text: root.project.version;",
        "text: root.visible-modified;",
        "text: root.project.project-path;",
        "row-height: root.row-height;",
    ] {
        assert!(
            table_row.contains(snippet),
            "ProjectTableRow must delegate ordinary body columns to TableCellText: {snippet}"
        );
    }
    assert_eq!(
        table_row.matches("TableCellText {").count(),
        3,
        "ProjectTableRow should use TableCellText for version, modified, and location columns"
    );

    for snippet in [
        "row-radius: HubVisualSpec.compact-radius;",
        "interaction-foreground: root.content-foreground;",
        "clicked =>",
        "root.select(root.project.open-path);",
        "visible-modified: root.project.modified == \"1d ago\" ? \"Yesterday\" : root.project.modified;",
        "text: root.visible-modified;",
    ] {
        assert!(
            table_row.contains(snippet),
            "ProjectTableRow should normalize reference fixture labels at the presentation edge: {snippet}"
        );
    }
    for forbidden in [
        "text: root.project.version;\n            overflow: elide;",
        "text: root.visible-modified;\n            overflow: elide;",
        "text: root.project.project-path;\n            overflow: elide;",
        "style: MaterialTypography.body_small;",
        "MaterialText {",
        "text: root.project.title;",
        "source: root.project.cover-image;",
        "thumbnail-outline:",
    ] {
        assert!(
            !table_row.contains(forbidden),
            "ProjectTableRow should not own table body-cell or name-lane rendering after adopting focused cells: {forbidden}"
        );
    }
}

#[test]
fn data_table_rows_use_shared_trailing_action_slot() {
    let table_view = read_ui_file("table_view_components.slint");
    let table_row = table_view
        .split("export component ProjectTableRow")
        .nth(1)
        .and_then(|source| source.split("export component DataTable").next())
        .expect("table_view_components.slint must declare ProjectTableRow before DataTable");

    for snippet in [
        "import { HubRowTrailingSlot } from \"row_slot_components.slint\";",
        "HubRowTrailingSlot {",
        "slot-width: root.action-size;",
        "show-badge: false;",
        "show-action: true;",
        "action-framed: false;",
        "action-size: root.action-size;",
        "action-icon-image: @image-url(\"../assets/icons/ui/more-vertical.svg\");",
        "clicked => {\n                root.open(root.project.open-path);",
    ] {
        assert!(
            table_view.contains(snippet) || table_row.contains(snippet),
            "ProjectTableRow must express its trailing action with the shared row slot: {snippet}"
        );
    }

    for forbidden in [
        "row-state := StateLayerArea {",
        "source: @image-url(\"../assets/icons/ui/more-vertical.svg\");",
        "StateLayerArea {\n                width: parent.width;",
    ] {
        assert!(
            !table_row.contains(forbidden),
            "ProjectTableRow should not recreate a local trailing action implementation after adopting HubRowTrailingSlot: {forbidden}"
        );
    }
}
