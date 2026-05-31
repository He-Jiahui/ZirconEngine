//! Static contracts for Projects subpage navigation and action routing.

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
fn open_project_detail_runtime_selects_project_before_detail_subpage() {
    let project_workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    let open_detail = project_workspace
        .split("pub(super) fn open_project_detail")
        .nth(1)
        .expect("project_workspace.rs should define open_project_detail");

    for snippet in [
        "self.select_project_path(project_path)?;",
        "self.project_subpage = ProjectSubpage::ProjectDetail;",
        "self.project_view_mode = ProjectViewMode::List;",
        "self.pending_delete_project_path = None;",
    ] {
        assert!(
            open_detail.contains(snippet),
            "open_project_detail must select the project and enter the detail subpage; missing {snippet}"
        );
    }
}

#[test]
fn import_project_header_imports_existing_project_without_launching_editor() {
    let app = read_ui_file("app.slint");
    let runtime = read_crate_file("src/app/runtime.rs");
    let folder_picker = read_crate_file("src/app/runtime/folder_picker.rs");
    let project_workspace = read_crate_file("src/app/runtime/project_workspace.rs");

    assert!(
        app.contains("callback import-project();"),
        "Hub page header Import Project must expose the dedicated runtime import callback"
    );
    assert!(
        app.contains("import-project => {") && app.contains("root.import-project();"),
        "Hub page header Import Project must call the dedicated runtime import callback instead of only routing to Project Browser"
    );
    for snippet in [
        "ui.on_import_project",
        "runtime.import_project(ui)",
        "pub(super) fn import_project(&mut self, ui: &HubWindow) -> Result<(), HubError>",
        "folder_picker_title(\"import-project\")",
        "self.import_project_path(selection)",
        "\"import-project\" => first_existing_dir",
        "\"import-project\" => \"Select existing project\"",
    ] {
        assert!(
            runtime.contains(snippet) || folder_picker.contains(snippet),
            "Import Project must be a real picker-backed runtime action; missing {snippet}"
        );
    }

    let import_project_path = project_workspace
        .split("pub(super) fn import_project_path")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn install_recent_project_to_device").next())
        .expect("project_workspace.rs must declare import_project_path before install_recent_project_to_device");
    for snippet in [
        "validate_project_root(&project_path)",
        "self.remember_project(RecentProject::with_now(",
        "self.project_filter = ProjectFilterMode::All;",
        "self.project_view_mode = ProjectViewMode::List;",
        "self.project_subpage = ProjectSubpage::ProjectDetail;",
        "self.pending_delete_project_path = None;",
        "self.search_query.clear();",
        "TaskStatus::success(\"Project imported\"",
    ] {
        assert!(
            import_project_path.contains(snippet),
            "Import Project must validate, remember, select, and open the imported project detail; missing {snippet}"
        );
    }
    for forbidden in [
        "ensure_editor_available",
        "launch_editor",
        "EditorLaunchCommand",
        "ProjectSubpage::ProjectBrowser",
    ] {
        assert!(
            !import_project_path.contains(forbidden),
            "Import Project should not launch Editor or degrade into a plain browser route: {forbidden}"
        );
    }
}

#[test]
fn project_browser_row_selects_and_detail_button_opens_detail() {
    let components = read_ui_file("project_browser_components.slint");
    let browser_row = components
        .split("export component ProjectBrowserRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("project_browser_components.slint must own ProjectBrowserRow");
    assert!(
        browser_row
            .matches("root.select(root.project.open-path);")
            .count()
            >= 1,
        "ProjectBrowserRow row state must still select the non-detail row body"
    );
    assert_eq!(
        browser_row
            .matches("root.open-detail(root.project.open-path);")
            .count(),
        1,
        "ProjectBrowserRow detail navigation should be limited to the trailing detail hit branch"
    );
    assert!(
        browser_row.contains("StateLayerArea {")
            && browser_row.contains("row-state :=")
            && browser_row.contains("border_radius: HubVisualSpec.compact-radius;"),
        "ProjectBrowserRow must use Material StateLayerArea for whole-row hover/press behavior"
    );
    let state_index = browser_row
        .find("row-state := StateLayerArea {")
        .expect("ProjectBrowserRow must declare a row StateLayerArea");
    let detail_button_index = browser_row
        .find("detail-button-shell := Rectangle {")
        .expect("ProjectBrowserRow must expose a trailing detail button shell");
    assert!(
        state_index < detail_button_index,
        "ProjectBrowserRow row StateLayerArea must own the full row before laying out the trailing detail shell"
    );
    let state_body = &browser_row[state_index..detail_button_index];
    assert!(
        state_body.contains("root.select(root.project.open-path);"),
        "ProjectBrowserRow StateLayerArea should select clicks outside the detail zone"
    );
    assert!(
        state_body.contains("horizontal-stretch: 1;") && state_body.contains("min-width: 1px;"),
        "ProjectBrowserRow StateLayerArea should let layout reserve the trailing detail zone instead of subtracting row width by hand"
    );
    assert!(
        !state_body.contains("row-state.mouse-x")
            && !state_body.contains("root.open-detail(root.project.open-path);"),
        "ProjectBrowserRow row StateLayerArea should not infer detail clicks from mouse coordinates"
    );
    assert!(
        browser_row
            .contains("detail-button-size: max(MaterialStyleMetrics.size_40, root.row-height * 2 / 5);")
            && browser_row
                .contains("in property <length> detail-column-width: HubTokens.control-md;"),
        "ProjectBrowserRow detail control should be derived from Material icon-button and row-height tokens instead of fixed pixel coordinates"
    );
    let detail_body = &browser_row[detail_button_index..];
    let detail_state_index = detail_body
        .find("detail-state := StateLayerArea {")
        .expect("ProjectBrowserRow detail shell must contain a Material StateLayerArea");
    let detail_state_body = &detail_body[detail_state_index..];
    let detail_state_declaration = detail_state_body
        .split("color:")
        .next()
        .expect("ProjectBrowserRow detail StateLayerArea must declare its size before color");
    assert!(
        detail_body.contains("width: root.detail-column-width;")
            && detail_body.contains("height: parent.height;")
            && detail_body.contains("alignment: center;")
            && detail_state_body.contains("width: root.detail-button-size;")
            && detail_state_body.contains("height: root.detail-button-size;")
            && detail_state_body.contains("border_radius: root.detail-button-size / 2;")
            && detail_state_body.contains("chevron-right.svg")
            && detail_state_body.contains("Icon {")
            && detail_state_body.contains("clicked => { root.open-detail(root.project.open-path); }"),
        "ProjectBrowserRow trailing detail shell should center a Material StateLayerArea icon button inside the tokenized trailing slot"
    );
    assert!(
        !detail_state_declaration.contains("width: parent.width;")
            && !detail_state_declaration.contains("height: parent.height;"),
        "ProjectBrowserRow detail StateLayerArea should not paint the full trailing slot"
    );
    for forbidden in [
        "thumb-area := TouchArea",
        "body-area := TouchArea",
        "area := TouchArea",
    ] {
        assert!(
            !browser_row.contains(forbidden),
            "ProjectBrowserRow must not return to custom cell/root TouchArea navigation: {forbidden}"
        );
    }
}

#[test]
fn project_browser_entry_points_are_separate_from_dashboard_show_more() {
    let projects_page = read_ui_file("projects.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let dashboard_surface = format!("{dashboard}\n{dashboard_components}");
    let project_projection = read_crate_file("src/app/view_model/projects.rs");

    let show_more_block = dashboard_components
        .split("text: root.expanded ? root.collapse-label : root.show-more-label;")
        .nth(1)
        .and_then(|source| source.split("export component DashboardQuickActionRow").next())
        .expect("DashboardProjectCardsSection must keep the Show More button inside the dashboard component module");
    assert!(
        show_more_block.contains("clicked => { root.expanded = !root.expanded; }"),
        "Dashboard Show More must only expand/collapse project cards"
    );
    for forbidden in [
        "project-browser",
        "root.view-all-projects();",
        "root.show-project-subpage",
        "root.set-project-view-mode(\"list\")",
    ] {
        assert!(
            !show_more_block.contains(forbidden),
            "Dashboard Show More must not share View All/List navigation behavior: {forbidden}"
        );
    }

    for snippet in [
        "DashboardProjectCardsSection {",
        "expanded <=> root.project-cards-expanded;",
        "action-text: root.ui-text.view-all-projects;",
        "action-clicked => { root.view-all(); }",
        "view-all => { root.view-all-projects(); }",
        "root.project-view-mode = \"list\";",
        "root.project-subpage = \"project-browser\";",
        "root.view-all-projects();",
        "icon-image: @image-url(\"../assets/icons/ui/list.svg\");",
        "active: root.project-view-mode == \"list\";",
        "root.set-project-view-mode(\"list\");",
        "root.show-project-subpage(\"project-browser\");",
    ] {
        assert!(
            projects_page.contains(snippet) || dashboard_surface.contains(snippet),
            "View All Projects and the dashboard list toggle must enter the Project Browser secondary page; missing {snippet}"
        );
    }

    assert!(
        projects_page.contains("list-title: root.dashboard-project-title;"),
        "ProjectBrowserPage must reuse the shared Recent projection title instead of a hard-coded Project List heading"
    );
    for snippet in [
        "localization::text(language, \"Recent Projects\", \"最近项目\")",
        "pub(in crate::app) fn dashboard_project_rows(snapshot: &HubSnapshot) -> Vec<RecentProjectRowData>",
        ".filtered_recent_projects()",
        "fn project_browser_projects(snapshot: &HubSnapshot) -> Vec<RecentProject>",
        ".filter(|project| project_is_pinned(project, snapshot))",
        "if pinned.is_empty()",
    ] {
        assert!(
            project_projection.contains(snippet),
            "Dashboard must keep the complete recent-project list while Project Browser can prefer pinned projects; missing {snippet}"
        );
    }
}

#[test]
fn new_project_location_state_is_separate_from_settings_default_location() {
    let app = read_ui_file("app.slint");
    let projects = read_ui_file("projects.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let project_new_page = read_ui_file("project_new_page.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let binding = read_crate_file("src/app/binding.rs");
    let runtime = read_crate_file("src/app/runtime.rs");
    let folder_picker = read_crate_file("src/app/runtime/folder_picker.rs");
    let project_workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    let hub_config = read_crate_file("src/settings/hub_config.rs");
    let snapshot = read_crate_file("src/state/hub_snapshot.rs");

    for snippet in [
        "in-out property <string> new-project-location;",
        "new-project-location <=> root.new-project-location;",
        "project-location <=> root.project-location;",
    ] {
        assert!(
            app.contains(snippet),
            "app.slint must keep New Project location separate from Settings default location; missing {snippet}"
        );
    }

    for snippet in [
        "in-out property <string> new-project-location;",
        "project-location <=> root.new-project-location;",
    ] {
        assert!(
            projects.contains(snippet),
            "projects.slint must bind ProjectNewPage to the dedicated New Project location; missing {snippet}"
        );
    }

    assert!(
        project_pages.contains("export { ProjectNewPage } from \"project_new_page.slint\";")
            && project_new_page.contains("browse-folder(kind) => { root.browse-folder(kind); }")
            && project_components.contains("root.browse-folder(\"new-project-location\");"),
        "ProjectNewPage browse must not mutate Settings' default project location"
    );
    assert!(
        binding.contains("ui.set_new_project_location(")
            && binding.contains("new_project_location"),
        "binding.rs must drive the New Project location from HubSnapshot, not HubSettings"
    );
    let runtime_persistence = read_crate_file("src/app/runtime/persistence.rs");
    assert!(
        runtime.contains("new_project_location: PathBuf")
            && hub_config.contains("pub new_project_location: PathBuf,")
            && hub_config.contains("new_project_location: default_project_dir(),")
            && runtime_persistence
                .contains("new_project_location: runtime_state.new_project_location,")
            && runtime_persistence
                .contains("new_project_location: config.settings.default_project_dir.clone(),")
            && runtime_persistence.contains("new_project_location: self.new_project_location.clone(),")
            && runtime.contains(
                "self.new_project_location = PathBuf::from(ui.get_new_project_location().to_string());"
            ),
        "runtime and HubConfig must keep editable New Project location in Hub runtime state while retaining the default project directory fallback"
    );
    assert!(
        snapshot.contains("pub new_project_location: PathBuf,"),
        "HubSnapshot must expose the editable New Project location"
    );
    assert!(
        folder_picker.contains("\"new-project-location\" => {")
            && folder_picker.contains("ui.set_new_project_location(selected.clone().into());"),
        "folder picker must update New Project location without changing Settings"
    );
    assert!(
        project_workspace.contains("self.new_project_location.clone(),"),
        "create_project must use the dedicated New Project location"
    );
}
