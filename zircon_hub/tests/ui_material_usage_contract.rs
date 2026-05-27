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
fn material_template_is_directly_imported_through_bridge() {
    let build_rs = read_crate_file("build.rs");
    for snippet in [
        "config.enable_experimental = true;",
        "config.library_paths.insert(",
        "\"material\".to_string(),",
        ".join(\"dev/material-rust-template/material-1.0/material.slint\"),",
    ] {
        assert!(
            build_rs.contains(snippet),
            "Hub must keep @material mapped directly to the checked-in Slint Material template; missing {snippet}"
        );
    }

    let bridge = read_ui_file("material_bridge.slint");
    for snippet in [
        "} from \"@material\";",
        "FilledButton,",
        "OutlineButton,",
        "TonalButton,",
        "IconButton,",
        "ListTile,",
        "MaterialWindow,",
        "MaterialWindowAdapter,",
        "NavigationRail,",
        "SearchBar,",
        "ScrollView,",
        "SegmentedButton,",
        "TextField,",
        "MaterialPalette,",
        "MaterialStyleMetrics,",
        "MaterialTypography,",
    ] {
        assert!(
            bridge.contains(snippet),
            "material_bridge.slint must re-export the imported Material template surface instead of local clones; missing {snippet}"
        );
    }

    let material_entry = read_repo_file("dev/material-rust-template/material-1.0/material.slint");
    for snippet in [
        "export { FilledButton }",
        "export { OutlineButton }",
        "export { IconButton }",
        "export { NavigationRail }",
        "export { SearchBar }",
        "export { ScrollView }",
        "export { SegmentedButton }",
        "export { TextField }",
        "export { MaterialStyleMetrics }",
        "export { MaterialPalette }",
        "export { MaterialTypography }",
    ] {
        assert!(
            material_entry.contains(snippet),
            "the vendored Slint Material template must remain the source for Hub Material components; missing {snippet}"
        );
    }

    let inputs = read_ui_file("inputs.slint");
    for snippet in [
        "SearchBar,",
        "TextField,",
        "OutlineButton,",
        "PopupMenu,",
        "SegmentedButton,",
        "} from \"material_bridge.slint\";",
    ] {
        assert!(
            inputs.contains(snippet),
            "input wrappers must compose the imported template through material_bridge.slint; missing {snippet}"
        );
    }
}

#[test]
fn status_pill_uses_material_action_chip() {
    let shared = read_ui_file("shared.slint");
    let status_pill = shared
        .split("export component StatusPill")
        .nth(1)
        .and_then(|source| source.split("export component WindowButton").next())
        .expect("shared.slint must declare StatusPill before WindowButton");

    for snippet in [
        "ActionChip,",
        "ActionChip {",
        "icon: root.has-icon-image ? root.icon-image : @image-url(\"../assets/icons/status/running.svg\");",
        "text: root.text;",
        "tooltip: root.text;",
        "clip: true;",
    ] {
        assert!(
            shared.contains(snippet) || status_pill.contains(snippet),
            "StatusPill must preserve the Hub header-status API while delegating its chip body to Material ActionChip; missing {snippet}"
        );
    }

    for forbidden in [
        "CenteredIcon",
        "HorizontalLayout",
        "font-size: MaterialTypography.label_large.font_size;",
        "font-weight: MaterialTypography.label_large.font_weight;",
    ] {
        assert!(
            !status_pill.contains(forbidden),
            "StatusPill should not return to a custom painted icon/text pill: {forbidden}"
        );
    }
}

#[test]
fn editor_source_engine_row_uses_material_list_tile() {
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let source_engine_row = editor_components
        .split("export component SourceEngineRow")
        .nth(1)
        .and_then(|source| source.split("export component EditorSideListPanel").next())
        .expect("editor_page_components.slint must export SourceEngineRow before EditorSideListPanel");
    assert!(
        !editor.contains("component SourceEngineRow"),
        "editor.slint should import SourceEngineRow instead of defining it inline"
    );

    for snippet in [
        "height: HubTokens.list-row-md;",
        "ListTile {",
        "text: root.engine.title;",
        "supporting_text: root.engine.version + \" / \" + root.engine.source-path + \" / \" + root.engine.last-build;",
        "avatar_icon:",
        "avatar_background:",
        "avatar_foreground:",
        "clicked =>",
        "Badge {",
        "IconButton {",
        "remove-size: min(HubTokens.icon-lg, HubTokens.list-row-md - MaterialStyleMetrics.spacing_16);",
    ] {
        assert!(
            source_engine_row.contains(snippet),
            "SourceEngineRow must delegate source-engine rows to Material ListTile; missing {snippet}"
        );
    }

    for forbidden in [
        "area := TouchArea",
        "border-color: area.has-hover",
        "background: area.has-hover",
        "root.height - MaterialStyleMetrics.spacing_16",
    ] {
        assert!(
            !source_engine_row.contains(forbidden),
            "SourceEngineRow should not return to a custom full-row TouchArea or self-height subtraction implementation: {forbidden}"
        );
    }
}

#[test]
fn project_pages_use_material_scroll_view() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let project_surface = format!("{project_pages}\n{project_components}");

    for snippet in [
        "ScrollView,",
        "PanelListViewport,",
        "vertical_scrollbar_policy: ScrollBarPolicy.as-needed;",
        "horizontal_scrollbar_policy: ScrollBarPolicy.always-off;",
    ] {
        assert!(
            dashboard.contains(snippet),
            "ProjectDashboardPage must route page/list scrolling through the Material ScrollView API; missing {snippet}"
        );
    }
    for snippet in [
        "PanelListViewport,",
        "browser-list := PanelListViewport {",
        "export component ProjectEngineChoiceList inherits Rectangle",
        "engine-list := PanelListViewport {",
        "ProjectEngineChoiceList {",
        "export component ProjectCreateActionRow inherits Rectangle",
        "ProjectCreateActionRow {",
    ] {
        assert!(
            project_surface.contains(snippet),
            "Project secondary pages must route list scrolling through the shared Material ScrollView wrapper; missing {snippet}"
        );
    }
    for (page, source) in [
        ("ProjectDashboardPage", &dashboard),
        ("Project secondary pages", &project_pages),
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
        "page-padding: root.page-pad;",
        "bottom-padding: root.page-pad;",
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
            project_pages.contains(snippet),
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
            !project_pages.contains(forbidden),
            "Project secondary pages should derive page content sizing from PageScrollSurface instead of hand-written page formulas: {forbidden}"
        );
    }

    for snippet in [
        "card-scroll := ScrollView {",
        "viewport_y <=> root.card-scroll-y;",
        "action-list := PanelListViewport {",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "row-count: root.quick-action-count;",
    ] {
        assert!(
            dashboard.contains(snippet),
            "ProjectDashboardPage must keep project cards on Material ScrollView and quick actions on the shared list viewport; missing {snippet}"
        );
    }

    for snippet in [
        "browser-list := PanelListViewport {",
        "scroll-y <=> root.browser-scroll-y;",
        "row-width: browser-list.visible_width;",
    ] {
        assert!(
            project_pages.contains(snippet),
            "Project Browser must keep list scrolling on the shared Material ScrollView wrapper while New/Detail use PageScrollSurface; missing {snippet}"
        );
    }
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
        "dashboard implementation details belong in project_dashboard.slint"
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
    for primitive in ["Flow", "PanelGrid", "PanelSlot", "ResponsiveSlot"] {
        assert!(
            dashboard.contains(primitive),
            "project_dashboard.slint must compose dashboard layout with {primitive}"
        );
    }
}

#[test]
fn dashboard_project_selectors_use_material_state_layers() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_card = dashboard
        .split("component ProjectCard")
        .nth(1)
        .and_then(|source| source.split("component ProjectFlow").next())
        .expect("project_dashboard.slint must declare ProjectCard before ProjectFlow");
    for snippet in [
        "StateLayerArea,",
        "card-state := StateLayerArea {",
        "border_radius: MaterialStyleMetrics.border_radius_12;",
        "root.select(root.project.open-path);",
    ] {
        assert!(
            dashboard.contains(snippet) || project_card.contains(snippet),
            "ProjectCard must delegate whole-card select feedback to Material StateLayerArea; missing {snippet}"
        );
    }
    for forbidden in ["area := TouchArea", "area.has-hover"] {
        assert!(
            !project_card.contains(forbidden),
            "ProjectCard should not return to a custom full-card TouchArea: {forbidden}"
        );
    }

    let data_display = read_ui_file("data_display.slint");
    let table_row = data_display
        .split("export component ProjectTableRow")
        .nth(1)
        .and_then(|source| source.split("export component DataTable").next())
        .expect("data_display.slint must declare ProjectTableRow before DataTable");
    for snippet in [
        "StateLayerArea,",
        "row-state := StateLayerArea {",
        "border_radius: MaterialStyleMetrics.border_radius_8;",
        "root.select(root.project.open-path);",
    ] {
        assert!(
            data_display.contains(snippet) || table_row.contains(snippet),
            "ProjectTableRow must delegate whole-row select feedback to Material StateLayerArea; missing {snippet}"
        );
    }
    for forbidden in ["area := TouchArea", "area.has-hover"] {
        assert!(
            !table_row.contains(forbidden),
            "ProjectTableRow should not return to custom full-row TouchArea hover/select handling: {forbidden}"
        );
    }
}

#[test]
fn project_choice_rows_use_material_list_tiles() {
    let components = read_ui_file("project_page_components.slint");
    let data_display = read_ui_file("data_display.slint");
    let shared = read_ui_file("shared.slint");
    assert!(
        data_display.contains("export component InfoRow") && data_display.contains("ListTile {"),
        "InfoRow must remain the shared Material ListTile-backed choice-row body"
    );
    for row_name in ["EngineChoiceRow", "TemplateChoiceRow"] {
        let row = components
            .split(&format!("export component {row_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("project_page_components.slint must declare {row_name}"));
        assert!(
            row.contains("InfoRow {"),
            "{row_name} must delegate row layout and interaction to the shared Material ListTile-backed InfoRow"
        );
        for snippet in [
            "in property <bool> collapse-label: false;",
            "effective-row-height: max(root.row-height, HubTokens.list-row-md);",
            "height: root.effective-row-height;",
            "row-height: root.effective-row-height;",
            "collapse-trailing-label: root.collapse-label;",
        ] {
            assert!(
                row.contains(snippet),
                "{row_name} must keep text and trailing badges aligned by respecting Material ListTile's minimum row height; missing {snippet}"
            );
        }
        for forbidden in ["CenteredIcon", "area := TouchArea"] {
            assert!(
                !row.contains(forbidden),
                "{row_name} should not return to a custom icon/click row: {forbidden}"
            );
        }
    }
    let engine_choice_row = components
        .split("export component EngineChoiceRow")
        .nth(1)
        .and_then(|source| source.split("export component TemplateChoiceRow").next())
        .expect(
            "project_page_components.slint must declare EngineChoiceRow before TemplateChoiceRow",
        );
    for snippet in [
        "detail: root.engine.version;",
        "meta: root.engine.source-path;",
    ] {
        assert!(
            engine_choice_row.contains(snippet),
            "EngineChoiceRow must surface both engine version and source path in the Material ListTile supporting text; missing {snippet}"
        );
    }

    let project_pages = read_ui_file("project_pages.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_surface = format!("{project_pages}\n{project_components}");
    for snippet in [
        "choice-row-height: max(HubTokens.list-row-md, min(HubTokens.list-row-lg, root.content-height / 10));",
        "template-row-height: max(HubTokens.list-row-md, min(HubTokens.list-row-lg, root.content-height / 9));",
        "compact-choice-labels: root.narrow-flow;",
        "collapse-label: root.compact-choice-labels;",
    ] {
        assert!(
            project_pages.contains(snippet),
            "ProjectNewPage choice rows must use responsive Material row tokens instead of undersized local formulas; missing {snippet}"
        );
    }
    assert!(
        project_pages.contains("detail-choice-row-height: max(HubTokens.list-row-md"),
        "ProjectDetailPage engine choices must respect Material ListTile's minimum row height"
    );
    assert!(
        project_pages.contains("collapse-label: root.narrow-flow;"),
        "ProjectDetailPage engine choices should collapse trailing labels in the compact flow"
    );
    for snippet in [
        "change-source-engine: string,",
        "remove-from-hub-detail: string,",
        "StatusBanner,",
        "text: root.ui-text.remove-from-hub-detail;",
        "if root.project.pending-delete: StatusBanner",
        "title: root.ui-text.confirm-delete;",
        "detail: root.ui-text.recycle-bin-delete-detail;",
        "tone: \"error\";",
        "export component ProjectDetailStatusStrip inherits Rectangle",
        "Badge {",
        "text: root.detail.version;",
        "text: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "text: root.copy.modified-prefix + root.detail.modified;",
        "ProjectDetailStatusStrip {",
        "export component ProjectDetailEngineSection inherits Rectangle",
        "title: root.copy.change-source-engine;",
        "subtitle: root.copy.bound-source-engine + \": \" + root.detail.engine-label;",
        "ProjectDetailEngineSection {",
        "export component ProjectDetailActionButton inherits PillButton",
        "in property <image> action-icon;",
        "icon-image: root.action-icon;",
        "has-icon-image: true;",
        "ProjectDetailActionButton {",
        "if root.project.pending-delete: ProjectDetailActionButton",
        "clicked => { root.confirm-delete(); }",
        "clicked => { root.cancel-delete(); }",
        "if !root.project.pending-delete: ProjectDetailActionButton",
        "clicked => { root.open-project(root.project.open-path); }",
        "clicked => { root.toggle-pin(); }",
        "clicked => { root.remove-from-hub(); }",
        "clicked => { root.request-delete(); }",
        "if !root.project.pending-delete: ProjectDetailEngineSection",
    ] {
        assert!(
            shared.contains(snippet) || project_surface.contains(snippet),
            "ProjectDetailPage must expose changing the bound Source Engine and the non-destructive Remove from Hub action as explicit secondary-page operations; missing {snippet}"
        );
    }
    assert_eq!(
        project_pages.matches("ProjectDetailActionButton {").count(),
        6,
        "ProjectDetailPage should render confirm, cancel, open, pin, remove, and delete through one action-button wrapper"
    );
    assert_eq!(
        project_pages.matches("ProjectDetailStatusStrip {").count(),
        1,
        "ProjectDetailPage should render version, pin state, and modified time through one status-strip wrapper"
    );
    assert_eq!(
        project_pages.matches("ProjectDetailEngineSection {").count(),
        1,
        "ProjectDetailPage should render the Change Source Engine block through one section wrapper"
    );
    for forbidden in [
        "text: root.project.pending-delete ? root.ui-text.confirm-delete : root.ui-text.delete-project;",
        "if (root.project.pending-delete) {",
        "if root.project.pending-delete: PillButton",
        "if !root.project.pending-delete: PillButton",
    ] {
        assert!(
            !project_pages.contains(forbidden),
            "ProjectDetailPage pending delete should use a confirmation-first action cluster instead of a bottom-of-panel ternary delete button: {forbidden}"
        );
    }
}
