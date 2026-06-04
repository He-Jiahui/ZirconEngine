//! Static contracts for the native Projects Browser table layout.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

#[test]
fn project_browser_rows_have_localized_status_data() {
    let shared = read_ui_file("shared.slint");
    let projects = read_crate_file("src/app/view_model/projects.rs");

    let row_struct = shared
        .split("export struct RecentProjectRowData")
        .nth(1)
        .and_then(|source| source.split("export struct ProjectTemplateData").next())
        .expect("shared.slint must declare RecentProjectRowData");
    assert!(
        row_struct.contains("status: string,"),
        "RecentProjectRowData must expose localized project status for table rows"
    );

    let row_projection = projects
        .split("fn recent_project_row")
        .nth(1)
        .and_then(|source| source.split("fn project_is_selected").next())
        .expect("projects.rs must declare recent_project_row before project_is_selected");
    for snippet in [
        "let missing = project_path_missing(project);",
        "let can_open = !missing &&",
        "status: project_detail_status_label(missing, can_open, language),",
    ] {
        assert!(
            row_projection.contains(snippet),
            "recent_project_row must project localized Ready/Missing/Invalid status into the UI row model; missing {snippet}"
        );
    }
}

#[test]
fn project_browser_header_and_rows_share_one_column_model() {
    let components = read_ui_file("project_browser_components.slint");
    let page = read_ui_file("project_browser_page.slint");
    let table_view = read_ui_file("table_view_components.slint");

    for snippet in [
        "export component ProjectBrowserTableHeader inherits Rectangle",
        "component ProjectBrowserNameCell inherits Rectangle",
        "export component ProjectBrowserRow inherits HubInteractiveRowSurface",
        "export component ProjectBrowserResultsPanel inherits HubTableView",
        "in property <length> table-padding-x",
        "in property <length> table-gap",
        "in property <length> thumbnail-column-width",
        "in property <length> engine-column-width",
        "in property <length> modified-column-width",
        "in property <length> status-column-width",
        "in property <length> detail-column-width",
        "in property <bool> compact-table",
        "TableCellText,",
        "TableColumnHeader,",
        "if !root.compact-table: TableColumnHeader",
        "if !root.compact-table: StatusBadge",
        "selected: root.project.selected;",
        "idle-background: HubVisualSpec.panel-background;",
        "selected-background: MaterialPalette.secondary_container;",
        "selected-border-width: MaterialStyleMetrics.size_1;",
        "row-radius: HubVisualSpec.compact-radius;",
        "interaction-foreground: root.content-foreground;",
        "clicked =>",
        "root.select(root.project.open-path);",
        "detail-slot := HubRowTrailingSlot {",
        "slot-width: root.detail-column-width;",
        "show-action: true;",
        "action-size: root.detail-button-size;",
    ] {
        assert!(
            components.contains(snippet),
            "Project Browser table components must expose a shared responsive column model; missing {snippet}"
        );
    }

    let header = components
        .split("export component ProjectBrowserTableHeader")
        .nth(1)
        .and_then(|source| source.split("component ProjectBrowserNameCell").next())
        .expect("project_browser_components.slint must declare ProjectBrowserTableHeader before ProjectBrowserNameCell");
    let name_cell = components
        .split("component ProjectBrowserNameCell")
        .nth(1)
        .and_then(|source| source.split("export component ProjectBrowserRow").next())
        .expect(
            "project_browser_components.slint must declare ProjectBrowserNameCell before ProjectBrowserRow",
        );
    let row = components
        .split("export component ProjectBrowserRow")
        .nth(1)
        .and_then(|source| source.split("export component ProjectBrowserResultsPanel").next())
        .expect("project_browser_components.slint must declare ProjectBrowserRow before ProjectBrowserResultsPanel");
    for shared_column in [
        "table-padding-x",
        "table-gap",
        "thumbnail-column-width",
        "engine-column-width",
        "modified-column-width",
        "status-column-width",
        "detail-column-width",
        "compact-table",
    ] {
        assert!(
            header.contains(shared_column) && row.contains(shared_column),
            "ProjectBrowserTableHeader and ProjectBrowserRow must both consume shared column property {shared_column}"
        );
    }
    assert!(
        header.matches("TableColumnHeader {").count() >= 4,
        "ProjectBrowserTableHeader must delegate every visible column label to the shared TableColumnHeader primitive"
    );
    for snippet in [
        "text: root.name-label;",
        "row-height: root.header-height;",
        "horizontal-stretch: 1;",
        "if !root.compact-table: TableColumnHeader",
        "width: root.engine-column-width;",
        "text: root.engine-label;",
        "width: root.modified-column-width;",
        "text: root.modified-label;",
        "width: root.status-column-width;",
        "text: root.status-label;",
    ] {
        assert!(
            header.contains(snippet),
            "ProjectBrowserTableHeader must preserve Browser column labels while using TableColumnHeader; missing {snippet}"
        );
    }
    for forbidden in ["MaterialText {", "MaterialTypography.label_medium"] {
        assert!(
            !header.contains(forbidden),
            "ProjectBrowserTableHeader should not recreate table-header typography after adopting TableColumnHeader: {forbidden}"
        );
    }

    for snippet in [
        "in property <RecentProjectRowData> project;",
        "in property <length> cell-height: HubTokens.list-row-lg;",
        "in property <color> title-foreground: MaterialPalette.on_surface;",
        "MaterialText {",
        "text: root.project.title;",
        "color: root.title-foreground;",
        "style: MaterialTypography.label_large;",
        "horizontal_alignment: left;",
        "if root.project.pinned: Badge",
        "text: root.pinned-label;",
        "badge-width: root.pinned-badge-width;",
        "if root.project.missing: Badge",
        "text: root.missing-label;",
        "badge-width: root.missing-badge-width;",
        "MutedText {",
        "text: root.project.project-path;",
    ] {
        assert!(
            name_cell.contains(snippet),
            "ProjectBrowserNameCell must own Browser title/path/badge typography; missing {snippet}"
        );
    }
    for snippet in [
        "ProjectBrowserNameCell {",
        "cell-height: root.row-height;",
        "project: root.project;",
        "table-gap: root.table-gap;",
        "title-line-height: root.title-line-height;",
        "pinned-label: root.pinned-label;",
        "missing-label: root.missing-label;",
        "pinned-badge-width: root.pinned-badge-width;",
        "missing-badge-width: root.missing-badge-width;",
        "title-foreground: root.content-foreground;",
    ] {
        assert!(
            row.contains(snippet),
            "ProjectBrowserRow must delegate the custom name lane to ProjectBrowserNameCell; missing {snippet}"
        );
    }
    assert!(
        table_view
            .split("export component TableCellText")
            .nth(1)
            .and_then(|source| source.split("export component ProjectTableRow").next())
            .is_some_and(|source| source.contains("horizontal_alignment: left;")),
        "ProjectBrowserRow must inherit ordinary body-cell alignment from TableCellText"
    );
    assert_eq!(
        row.matches("if !root.compact-table: TableCellText").count(),
        2,
        "ProjectBrowserRow should delegate engine and modified ordinary body columns to TableCellText"
    );
    for snippet in [
        "if !root.compact-table: TableCellText",
        "width: root.engine-column-width;",
        "row-height: root.row-height;",
        "text: root.project.version;",
        "width: root.modified-column-width;",
        "text: root.project.modified;",
    ] {
        assert!(
            row.contains(snippet),
            "ProjectBrowserRow must preserve Browser body column geometry while using TableCellText; missing {snippet}"
        );
    }
    for forbidden in [
        "if !root.compact-table: MaterialText",
        "MaterialText {",
        "MutedText {",
        "style: MaterialTypography.label_large;",
        "text: root.project.title;",
        "text: root.project.project-path;",
        "style: MaterialTypography.body_small;",
        "color: MaterialPalette.on_surface_variant;",
    ] {
        assert!(
            !row.contains(forbidden),
            "ProjectBrowserRow should not recreate ordinary body-cell text styling after adopting TableCellText: {forbidden}"
        );
    }
    for forbidden in [
        "row-state := StateLayerArea {",
        "area := TouchArea",
        "row-state.mouse-x",
    ] {
        assert!(
            !row.contains(forbidden),
            "ProjectBrowserRow should keep Browser selection on HubInteractiveRowSurface rather than local row hit testing: {forbidden}"
        );
    }

    let panel = components
        .split("export component ProjectBrowserResultsPanel")
        .nth(1)
        .expect("project_browser_components.slint must declare ProjectBrowserResultsPanel");
    for snippet in [
        "in property <bool> compact-table: false;",
        "in property <length> table-row-width: HubTokens.control-md;",
        "ProjectBrowserTableHeader {",
        "row-width: root.table-row-width;",
        "table-padding-x: root.table-padding-x;",
        "status-label: root.ui-text.project-status;",
        "browser-list := HubTableBody {",
        "row-width: root.table-row-width;",
        "table-padding-x: root.table-padding-x;",
        "status-column-width: root.status-column-width;",
        "compact-table: root.compact-table;",
    ] {
        assert!(
            panel.contains(snippet),
            "ProjectBrowserResultsPanel must consume the page-owned browser table geometry and pass it to header and rows; missing {snippet}"
        );
    }

    for snippet in [
        "browser-table-header-height: HubTokens.control-md;",
        "browser-table-horizontal-inset: root.page-gap * 2;",
        "browser-table-row-width: max(HubTokens.control-md, root.content-width - root.browser-table-horizontal-inset);",
        "browser-panel-chrome-height: HubTokens.control-md + root.browser-table-header-height + root.page-gap * 4;",
        "table-header-height: root.browser-table-header-height;",
        "compact-table: root.viewport-compact;",
        "table-row-width: root.browser-table-row-width;",
    ] {
        assert!(
            page.contains(snippet),
            "ProjectBrowserPage must own viewport-scoped browser table geometry; missing {snippet}"
        );
    }
}
