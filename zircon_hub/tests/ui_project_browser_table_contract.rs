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

    for snippet in [
        "export component ProjectBrowserTableHeader inherits Rectangle",
        "export component ProjectBrowserRow inherits Rectangle",
        "export component ProjectBrowserResultsPanel inherits HubTableView",
        "in property <length> table-padding-x",
        "in property <length> table-gap",
        "in property <length> thumbnail-column-width",
        "in property <length> engine-column-width",
        "in property <length> modified-column-width",
        "in property <length> status-column-width",
        "in property <length> detail-column-width",
        "in property <bool> compact-table",
        "if !root.compact-table: MaterialText",
        "if !root.compact-table: StatusBadge",
    ] {
        assert!(
            components.contains(snippet),
            "Project Browser table components must expose a shared responsive column model; missing {snippet}"
        );
    }

    let header = components
        .split("export component ProjectBrowserTableHeader")
        .nth(1)
        .and_then(|source| source.split("export component ProjectBrowserRow").next())
        .expect("project_browser_components.slint must declare ProjectBrowserTableHeader before ProjectBrowserRow");
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
        header.matches("horizontal_alignment: left;").count() >= 4,
        "ProjectBrowserTableHeader must left-align every visible column label so labels share row column starts"
    );
    assert!(
        row.matches("horizontal_alignment: left;").count() >= 3,
        "ProjectBrowserRow must left-align name, engine, and modified text against the table header columns"
    );

    let panel = components
        .split("export component ProjectBrowserResultsPanel")
        .nth(1)
        .expect("project_browser_components.slint must declare ProjectBrowserResultsPanel");
    for snippet in [
        "private property <bool> compact-table: root.width < HubTokens.breakpoint-compact;",
        "private property <length> table-row-width: max(HubTokens.control-md, root.width - root.panel-spacing * 2);",
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
            "ProjectBrowserResultsPanel must be the single owner of browser table geometry and pass it to header and rows; missing {snippet}"
        );
    }

    for snippet in [
        "browser-table-header-height: HubTokens.control-md;",
        "browser-panel-chrome-height: HubTokens.control-md + root.browser-table-header-height + root.page-gap * 4;",
        "table-header-height: root.browser-table-header-height;",
    ] {
        assert!(
            page.contains(snippet),
            "ProjectBrowserPage must reserve vertical space for the native table header; missing {snippet}"
        );
    }
}
