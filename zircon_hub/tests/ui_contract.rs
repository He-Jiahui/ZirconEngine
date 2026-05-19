//! Static Hub UI layout and component contracts.

use std::{
    fs,
    path::{Path, PathBuf},
};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_ui_file(name: &str) -> String {
    fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
        panic!("failed to read Hub UI file {name}: {error}");
    })
}

fn read_crate_file(name: &str) -> String {
    fs::read_to_string(crate_dir().join(name)).unwrap_or_else(|error| {
        panic!("failed to read Hub crate file {name}: {error}");
    })
}

fn slint_files() -> Vec<PathBuf> {
    let mut files = fs::read_dir(ui_dir())
        .expect("failed to read Hub UI directory")
        .map(|entry| entry.expect("failed to read Hub UI entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "slint")
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn components_entrypoint_stays_thin_and_reexports_new_modules() {
    let components = read_ui_file("components.slint");
    let line_count = components.lines().count();
    assert!(
        line_count <= 140,
        "components.slint should remain a re-export entrypoint; found {line_count} lines"
    );

    for module in [
        "tokens.slint",
        "layout.slint",
        "surfaces.slint",
        "inputs.slint",
        "shell.slint",
        "navigation.slint",
        "data_display.slint",
        "overlays.slint",
        "material_bridge.slint",
    ] {
        assert!(
            components.contains(module),
            "components.slint must re-export {module}"
        );
    }
}

#[test]
fn hub_directly_registers_and_reexports_material_template() {
    let material_template = crate_dir()
        .parent()
        .expect("zircon_hub lives below the repository root")
        .join("dev/material-rust-template/material-1.0/material.slint");
    assert!(
        material_template.is_file(),
        "Hub must import the local Slint Material template directly from {}",
        material_template.display()
    );

    let build = read_crate_file("build.rs");
    for snippet in [
        "config.library_paths.insert(",
        "\"material\".to_string()",
        "dev/material-rust-template/material-1.0/material.slint",
    ] {
        assert!(
            build.contains(snippet),
            "build.rs must register the local Material template with Slint as @material; missing {snippet}"
        );
    }

    let material_bridge = read_ui_file("material_bridge.slint");
    assert!(
        material_bridge.contains("} from \"@material\";"),
        "material_bridge.slint must directly re-export the local @material template library"
    );
    assert!(
        material_bridge.lines().count() <= 120,
        "material_bridge.slint should stay a direct export bridge, not an implementation file"
    );

    let components = read_ui_file("components.slint");
    assert!(
        components.contains("material_bridge.slint"),
        "components.slint must expose Material template components through material_bridge.slint"
    );
    for component in [
        "AppBar",
        "CheckBox",
        "ActionChip",
        "Dialog",
        "Drawer",
        "DropDownMenu",
        "Vertical",
        "Horizontal",
        "Grid",
        "ScrollView",
        "ElevatedCard",
        "FilledCard",
        "OutlinedCard",
        "FilledButton",
        "IconButton",
        "ListView",
        "NavigationRail",
        "CircularProgressIndicator",
        "SearchBar",
        "SegmentedButton",
        "Switch",
        "TabBar",
        "TextButton",
        "TextField",
        "ToolTip",
        "MaterialStyleMetrics",
        "MaterialPalette",
        "MaterialTypography",
    ] {
        assert!(
            material_bridge.contains(component) && components.contains(component),
            "Hub must expose Material template component {component}"
        );
    }
    assert!(
        !material_bridge.contains("\n    Badge,"),
        "material_bridge.slint omits Material Badge so components.slint can keep Hub Badge as the public Badge name"
    );

    let layout = read_ui_file("layout.slint");
    assert!(
        layout.contains("import { ScrollView } from \"@material\";")
            && layout.contains("page-scroll := ScrollView"),
        "Page must use the ScrollView component from the direct @material template import"
    );
}

#[test]
fn hub_applies_zircon_material_theme_to_material_palette() {
    let app = read_ui_file("app.slint");
    for snippet in [
        "import { ZirconMaterialTheme } from \"theme.slint\";",
        "ZirconMaterialTheme { }",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must install the Zircon Material theme from Slint before rendering controls; missing {snippet}"
        );
    }

    let theme = read_ui_file("theme.slint");
    for snippet in [
        "import { MaterialPalette } from \"material_bridge.slint\";",
        "export component ZirconMaterialTheme",
        "MaterialPalette.schemes = {",
        "primary: rgb(52, 213, 208),",
        "primaryContainer: rgb(0, 81, 84),",
        "tertiary: rgb(255, 202, 83),",
        "background: rgb(13, 17, 20),",
        "surfaceContainerLow: rgb(20, 25, 28),",
    ] {
        assert!(
            theme.contains(snippet),
            "Zircon Material theme must keep the Hub palette teal/dark instead of the template default blue; missing {snippet}"
        );
    }
}

#[test]
fn layout_primitives_and_tokens_are_declared() {
    let layout = read_ui_file("layout.slint");
    for primitive in [
        "Stack",
        "Row",
        "Column",
        "Flow",
        "Page",
        "PanelGrid",
        "WorkspacePanelSection",
        "ResponsiveSlot",
        "Fill",
        "Divider",
    ] {
        assert!(
            layout.contains(&format!("component {primitive}")),
            "layout.slint must declare {primitive}"
        );
    }

    let tokens = read_ui_file("tokens.slint");
    for token in [
        "import { MaterialPalette, MaterialStyleMetrics } from \"material_bridge.slint\";",
        "space-0",
        "space-1: MaterialStyleMetrics.spacing_4",
        "space-2: MaterialStyleMetrics.spacing_8",
        "space-3: MaterialStyleMetrics.spacing_12",
        "space-4: MaterialStyleMetrics.spacing_16",
        "breakpoint-compact: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_200 * 2",
        "breakpoint-medium: MaterialStyleMetrics.size_640 * 2",
        "breakpoint-wide: MaterialStyleMetrics.size_640 * 2 + MaterialStyleMetrics.size_256",
        "page-padding: MaterialStyleMetrics.padding_28",
        "panel-gap: MaterialStyleMetrics.spacing_16",
        "toolbar-gap: MaterialStyleMetrics.spacing_12",
        "input-field: MaterialStyleMetrics.size_56",
        "table-row: MaterialStyleMetrics.size_30",
        "list-row-sm: MaterialStyleMetrics.size_56",
        "list-row-md: MaterialStyleMetrics.size_72",
        "list-row-lg: MaterialStyleMetrics.size_80",
        "workspace-row-editor-summary: MaterialStyleMetrics.size_200 + MaterialStyleMetrics.size_90",
        "workspace-row-editor-config: MaterialStyleMetrics.size_200 * 2 + MaterialStyleMetrics.size_72 + MaterialStyleMetrics.size_4",
        "workspace-row-build-summary: MaterialStyleMetrics.size_200 + MaterialStyleMetrics.size_90 + MaterialStyleMetrics.spacing_8 + MaterialStyleMetrics.size_2",
        "workspace-row-build-detail: MaterialStyleMetrics.size_360",
        "workspace-row-settings-controls: MaterialStyleMetrics.size_200 + MaterialStyleMetrics.size_80 + MaterialStyleMetrics.size_6",
        "workspace-row-settings-detail: MaterialStyleMetrics.size_200 * 2 - MaterialStyleMetrics.size_8",
        "breakpoint-short: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_80 + MaterialStyleMetrics.size_40",
        "shell-row-min: MaterialStyleMetrics.size_52",
        "shell-row-max: MaterialStyleMetrics.size_56",
        "nav-width-collapsed-min: MaterialStyleMetrics.size_64",
        "nav-width-expanded-min: MaterialStyleMetrics.size_200",
        "window-resize-border: MaterialStyleMetrics.size_6",
        "window-min-width: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_344",
        "window-min-height: MaterialStyleMetrics.size_640",
        "window-preferred-width: root.breakpoint-wide + root.nav-width-collapsed-min",
        "window-preferred-height: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_360 + MaterialStyleMetrics.size_24",
        "user-menu-min-width: root.nav-width-collapsed-max + root.control-lg + root.space-1",
        "user-menu-max-width: root.nav-width-collapsed-max * 2 + root.space-1",
        "surface-page: MaterialPalette.background",
        "surface-panel: MaterialPalette.surface_container_low",
        "accent: MaterialPalette.primary",
    ] {
        assert!(tokens.contains(token), "tokens.slint is missing {token}");
    }
}

#[test]
fn hub_surfaces_are_backed_by_material_cards() {
    let surfaces = read_ui_file("surfaces.slint");
    for snippet in [
        "MaterialPalette",
        "MaterialStyleMetrics",
        "MaterialTypography",
        "OutlineButton",
        "if root.variant == \"elevated\": ElevatedCard",
        "if root.variant != \"elevated\": OutlinedCard",
        "border-radius: MaterialStyleMetrics.border_radius_12",
        "border-color: root.variant == \"danger\" ? MaterialPalette.error",
        "@children",
    ] {
        assert!(
            surfaces.contains(snippet),
            "HubPanel must preserve its Hub API while using Material Card surfaces; missing {snippet}"
        );
    }
    let hub_panel_end = surfaces
        .find("export component HubCard")
        .expect("surfaces.slint must declare HubCard after HubPanel");
    let hub_panel = &surfaces[..hub_panel_end];
    let children_index = hub_panel
        .find("@children")
        .expect("HubPanel must expose @children");
    assert!(
        !hub_panel[children_index..].contains("TouchArea"),
        "HubPanel must not place a panel-level TouchArea above @children; it blocks nested controls"
    );

    let panel_header = surfaces
        .split("export component PanelHeader")
        .nth(1)
        .and_then(|source| source.split("export component StatusBanner").next())
        .expect("surfaces.slint must declare PanelHeader before StatusBanner");
    for snippet in [
        "if root.show-action: OutlineButton",
        "height: HubTokens.control-md;",
        "text: root.action-text;",
        "icon: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "root.action-clicked();",
    ] {
        assert!(
            panel_header.contains(snippet),
            "PanelHeader action should be a Material OutlineButton; missing {snippet}"
        );
    }
    for forbidden in ["panel-action-area", "TouchArea", "CenteredIcon"] {
        assert!(
            !panel_header.contains(forbidden),
            "PanelHeader action should not return to a custom painted click layer: {forbidden}"
        );
    }
}

#[test]
fn shared_hub_buttons_are_backed_by_material_button_primitives() {
    let shared = read_ui_file("shared.slint");
    for snippet in [
        "FilledButton,",
        "FilledIconButton,",
        "OutlineButton,",
        "TonalButton,",
        "TonalIconButton,",
        "export component PillButton",
        "FilledButton {",
        "TonalButton {",
        "export component IconButton",
        "FilledIconButton {",
        "TonalIconButton {",
    ] {
        assert!(
            shared.contains(snippet),
            "shared.slint must keep Hub button APIs backed by Material button primitives; missing {snippet}"
        );
    }

    let pill_start = shared
        .find("export component PillButton")
        .expect("shared.slint must declare PillButton");
    let icon_start = shared
        .find("export component IconButton")
        .expect("shared.slint must declare IconButton");
    let nav_start = shared
        .find("export component NavButton")
        .expect("shared.slint must declare NavButton after IconButton");
    let pill_button = &shared[pill_start..icon_start];
    let icon_button = &shared[icon_start..nav_start];
    for snippet in [
        "height: MaterialStyleMetrics.size_40;",
        "min-width: root.height * 3;",
        "preferred-width: root.height * 4;",
        "clip: true;",
    ] {
        assert!(
            pill_button.contains(snippet),
            "PillButton must derive Material text button geometry from Material metrics and proportions; missing {snippet}"
        );
    }
    assert!(
        !pill_button.contains("preferred-width: 150px;"),
        "PillButton must not return to the old fixed-width wrapper"
    );
    assert!(
        icon_button.contains("clip: true;"),
        "Hub IconButton must clip Material icon buttons to the requested atom size in compact rows"
    );
    assert!(
        !pill_button.contains("TouchArea") && !icon_button.contains("TouchArea"),
        "PillButton and IconButton should delegate pointer behavior to Material buttons instead of hand-rolled TouchArea layers"
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
fn app_window_routes_shell_chrome_through_components() {
    let app = read_ui_file("app.slint");
    let line_count = app.lines().count();
    assert!(
        line_count <= 520,
        "app.slint should keep shell composition thin; found {line_count} lines"
    );

    for component in [
        "HubTopHeader",
        "HubNavSidebar",
        "HubPageHeader",
        "HubStatusBar",
    ] {
        assert!(
            app.contains(component),
            "app.slint must route shell chrome through {component}"
        );
    }

    assert!(
        !app.contains("component HeaderEngineOption")
            && !app.contains("component HeaderEngineSelector")
            && !app.contains("for item in root.nav-items: NavButton"),
        "window chrome implementation details belong in shell.slint"
    );

    let shell = read_ui_file("shell.slint");
    for component in [
        "HubTopHeader",
        "HubNavSidebar",
        "HubPageHeader",
        "HubStatusBar",
        "HeaderEngineSelector",
        "NavRail",
    ] {
        assert!(
            shell.contains(component),
            "shell.slint must declare or compose {component}"
        );
    }

    let header_engine_option = shell
        .split("component HeaderEngineOption")
        .nth(1)
        .and_then(|source| source.split("component HeaderEngineSelector").next())
        .expect("shell.slint must declare HeaderEngineOption before HeaderEngineSelector");
    for snippet in [
        "height: HubTokens.list-row-md;",
        "ListTile {",
        "text: root.engine.title;",
        "supporting_text: root.engine.status + \" / \" + root.engine.last-build;",
        "avatar_icon:",
        "avatar_background:",
        "avatar_foreground:",
        "clicked =>",
    ] {
        assert!(
            header_engine_option.contains(snippet),
            "HeaderEngineOption must use Material ListTile for source-engine popup rows; missing {snippet}"
        );
    }
    assert!(
        !header_engine_option.contains("area := TouchArea"),
        "HeaderEngineOption should not keep a custom full-row TouchArea now that ListTile owns row interaction"
    );
}

#[test]
fn app_shell_uses_preferred_window_size_and_remaining_content_width() {
    let app = read_ui_file("app.slint");
    for snippet in [
        "resize-border-width: HubTokens.window-resize-border;",
        "min-width: HubTokens.window-min-width;",
        "min-height: HubTokens.window-min-height;",
        "preferred-width: HubTokens.window-preferred-width;",
        "preferred-height: HubTokens.window-preferred-height;",
    ] {
        assert!(
            app.contains(snippet),
            "app.slint must use native window constraints instead of fixed startup size; missing {snippet}"
        );
    }
    for forbidden in ["\n    width: 1600px;", "\n    height: 1024px;"] {
        assert!(
            !app.contains(forbidden),
            "HubWindow root must not set fixed {forbidden:?}; use preferred dimensions"
        );
    }
    for snippet in [
        "horizontal-stretch: 1;",
        "vertical-stretch: 1;",
        "min-width: 1px;",
        "preferred-width: 0px;",
        "private property <bool> header-compact: root.width < HubTokens.breakpoint-wide;",
        "private property <bool> sidebar-compact-height: root.height < HubTokens.breakpoint-short;",
    ] {
        assert!(
            app.contains(snippet),
            "app.slint is missing required responsive shell contract snippet: {snippet}"
        );
    }
    for forbidden in [
        "parent.width - root.nav-width",
        "max-width: max(1px, parent.width",
    ] {
        assert!(
            !app.contains(forbidden),
            "app.slint must let Taffy/Slint allocate remaining content width instead of hand-written width subtraction: {forbidden}"
        );
    }
}

#[test]
fn workspace_pages_use_workspace_panel_section() {
    for page in ["editor.slint", "builds.slint", "settings.slint"] {
        let source = read_ui_file(page);
        assert!(
            source.contains("WorkspacePanelSection"),
            "{page} must compose workspace panels through WorkspacePanelSection"
        );
        assert!(
            !source.contains("ResponsivePanelFlow"),
            "{page} should not directly use low-level ResponsivePanelFlow"
        );
        assert!(
            !source.contains("flow-height: root.compact"),
            "{page} should not hand-write compact flow-height formulas"
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

    let dashboard = read_ui_file("project_dashboard.slint");
    for primitive in ["Flow", "PanelGrid", "ResponsiveSlot"] {
        assert!(
            dashboard.contains(primitive),
            "project_dashboard.slint must compose dashboard layout with {primitive}"
        );
    }
}

#[test]
fn project_browser_row_click_opens_detail() {
    let components = read_ui_file("project_page_components.slint");
    assert!(
        components.contains("component ProjectBrowserRow"),
        "project_page_components.slint must own ProjectBrowserRow"
    );
    assert!(
        components
            .matches("root.detail(root.project.project-path);")
            .count()
            >= 3,
        "ProjectBrowserRow click targets must open the detail page through one detail callback"
    );
    assert!(
        components.contains("thumb-area := TouchArea")
            && components.contains("body-area := TouchArea"),
        "ProjectBrowserRow must expose explicit hit targets inside its layout cells"
    );
    assert!(
        !components.contains("root.select(root.project.project-path);"),
        "ProjectBrowserRow hit targets should not issue a separate select callback before detail navigation"
    );
    assert!(
        !components.contains(
            "area := TouchArea {\n        width: parent.width;\n        height: parent.height;"
        ),
        "ProjectBrowserRow must not rely on a root-level full-row TouchArea for browser navigation"
    );
}

#[test]
fn project_choice_rows_use_material_list_tiles() {
    let components = read_ui_file("project_page_components.slint");
    for row_name in ["EngineChoiceRow", "TemplateChoiceRow"] {
        let row = components
            .split(&format!("export component {row_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("project_page_components.slint must declare {row_name}"));
        assert!(
            row.contains("ListTile"),
            "{row_name} must delegate row layout and interaction to the imported Material ListTile"
        );
        assert!(
            row.contains("avatar_icon:")
                && row.contains("avatar_background:")
                && row.contains("avatar_foreground:")
                && row.contains("supporting_text:"),
            "{row_name} must use Material ListTile icon and text slots"
        );
        for forbidden in ["CenteredIcon", "area := TouchArea"] {
            assert!(
                !row.contains(forbidden),
                "{row_name} should not return to a custom icon/click row: {forbidden}"
            );
        }
    }

    let project_pages = read_ui_file("project_pages.slint");
    assert!(
        project_pages.contains("detail-choice-row-height: max(HubTokens.list-row-md"),
        "ProjectDetailPage engine choices must respect Material ListTile's minimum row height"
    );
}

#[test]
fn component_samples_cover_layout_and_input_primitives() {
    let data_display = read_ui_file("data_display.slint");
    for sample in [
        "CatalogPage",
        "ComponentSamples",
        "Flow",
        "PanelGrid",
        "WorkspacePanelSection",
        "ResponsiveSlot",
        "SegmentButton",
        "ToolbarSelect",
        "HubTextField",
        "OutlinedCard",
        "TextField",
        "FilledButton",
        "OutlineButton",
        "FilledIconButton",
        "OutlineIconButton",
        "material_bridge.slint",
    ] {
        assert!(
            data_display.contains(sample),
            "ComponentSamples must cover {sample}"
        );
    }
}

#[test]
fn hub_form_text_inputs_use_material_text_field_wrapper() {
    let components = read_ui_file("components.slint");
    assert!(
        components.contains("HubTextField"),
        "components.slint must re-export the Hub Material-backed text field wrapper"
    );

    let inputs = read_ui_file("inputs.slint");
    for snippet in [
        "TextField",
        "MaterialStyleMetrics",
        "export component HubTextField",
        "material-field := TextField",
        "placeholder_text:",
        "text <=> root.text;",
        "height: HubTokens.input-field;",
        "preferred-width: HubTokens.input-width;",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep HubTextField backed by Material TextField; missing {snippet}"
        );
    }

    for page in ["settings.slint", "editor.slint", "project_pages.slint"] {
        let source = read_ui_file(page);
        assert!(
            source.contains("HubTextField"),
            "{page} form fields must use the HubTextField wrapper"
        );
        assert!(
            !source.contains("LineEdit"),
            "{page} should not reintroduce std-widgets LineEdit now that HubTextField owns Material input behavior"
        );
    }

    let project_pages = read_ui_file("project_pages.slint");
    assert!(
        project_pages.contains("field-height: HubTokens.input-field;"),
        "ProjectNewPage should derive form field height from HubTokens.input-field"
    );
}

#[test]
fn hub_search_box_uses_material_search_bar_wrapper() {
    let inputs = read_ui_file("inputs.slint");
    let search_box = inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("inputs.slint must declare SearchBox before HubTextField");

    for snippet in [
        "in property <length> box-height: HubTokens.input-field;",
        "search-field := SearchBar",
        "placeholder_text: root.placeholder;",
        "leading_icon: @image-url(\"../assets/icons/ui/search.svg\");",
        "empty_text: \"\";",
        "text <=> root.text;",
        "height: root.box-height;",
        "edited(value) =>",
        "accepted(value) =>",
    ] {
        assert!(
            search_box.contains(snippet),
            "SearchBox must stay backed by the imported Material SearchBar wrapper; missing {snippet}"
        );
    }

    for forbidden in [
        "TextInput",
        "border-color: search-field.has-focus",
        "selection-background-color",
        "CenteredIcon",
        "search-field := TextField",
        "label: root.placeholder",
    ] {
        assert!(
            !search_box.contains(forbidden),
            "SearchBox should not return to a custom painted TextInput implementation: {forbidden}"
        );
    }

    for page in ["project_dashboard.slint", "project_pages.slint"] {
        let source = read_ui_file(page);
        assert!(
            source.contains("box-height: root.toolbar-height;"),
            "{page} must size SearchBox through the responsive toolbar height"
        );
    }
}

#[test]
fn hub_segment_button_uses_material_segmented_button() {
    let inputs = read_ui_file("inputs.slint");
    let segment = inputs
        .split("export component SegmentButton")
        .nth(1)
        .expect("inputs.slint must declare SegmentButton");
    for snippet in [
        "SegmentedButton",
        "export component SegmentButton",
        "material-segment := SegmentedButton",
        "current_index <=> root.selected-index;",
        "items: [{ text: root.text }];",
        "index_changed(index) =>",
    ] {
        assert!(
            inputs.contains(snippet),
            "SegmentButton must stay backed by the imported Material SegmentedButton; missing {snippet}"
        );
    }
    for forbidden in [
        "border-color: root.active",
        "background: root.active",
        "area := TouchArea",
    ] {
        assert!(
            !segment.contains(forbidden),
            "SegmentButton should not return to a custom painted toggle implementation: {forbidden}"
        );
    }
}

#[test]
fn hub_toolbar_select_uses_material_menu_primitives() {
    let inputs = read_ui_file("inputs.slint");
    let toolbar_select = inputs
        .split("export component ToolbarSelect")
        .nth(1)
        .and_then(|source| source.split("export component DropDownButton").next())
        .expect("inputs.slint must declare ToolbarSelect before DropDownButton");
    for snippet in [
        "MenuItem",
        "OutlineButton",
        "PopupMenu",
        "in property <length> select-height: HubTokens.control-md;",
        "in property <[MenuItem]> menu-items: [];",
        "trigger := OutlineButton",
        "menu := PopupMenu",
        "items: root.menu-items;",
        "root.selected(root.options[index].id);",
    ] {
        assert!(
            inputs.contains(snippet),
            "ToolbarSelect must stay backed by imported Material menu/button primitives; missing {snippet}"
        );
    }
    for forbidden in [
        "SelectOptionRow",
        "popup := PopupWindow",
        "area := TouchArea",
    ] {
        assert!(
            !toolbar_select.contains(forbidden),
            "ToolbarSelect should not return to its custom popup/list implementation: {forbidden}"
        );
    }
    for page in ["project_dashboard.slint", "project_pages.slint"] {
        let source = read_ui_file(page);
        assert!(
            source.contains("menu-items: ["),
            "{page} must feed ToolbarSelect with Material MenuItem data"
        );
        assert!(
            source.contains("select-height: root.toolbar-height;"),
            "{page} must align ToolbarSelect to the same responsive toolbar height as SearchBox"
        );
    }
}

#[test]
fn hub_dropdown_button_uses_material_button_primitives() {
    let inputs = read_ui_file("inputs.slint");
    let dropdown = inputs
        .split("export component DropDownButton")
        .nth(1)
        .and_then(|source| source.split("export component SegmentButton").next())
        .expect("inputs.slint must declare DropDownButton before SegmentButton");
    for snippet in [
        "OutlineButton",
        "TonalButton",
        "export component DropDownButton",
        "if root.active: TonalButton",
        "if !root.active: OutlineButton",
        "icon: root.icon-image;",
    ] {
        assert!(
            inputs.contains(snippet),
            "DropDownButton must stay backed by imported Material button primitives; missing {snippet}"
        );
    }
    for forbidden in [
        "border-color: root.active",
        "background: root.active",
        "area := TouchArea",
    ] {
        assert!(
            !dropdown.contains(forbidden),
            "DropDownButton should not return to a custom painted button implementation: {forbidden}"
        );
    }
}

#[test]
fn page_compact_breakpoints_use_design_tokens() {
    let mut violations = Vec::new();
    for path in slint_files() {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if matches!(
            file_name,
            "components.slint"
                | "tokens.slint"
                | "layout.slint"
                | "surfaces.slint"
                | "inputs.slint"
                | "shell.slint"
                | "navigation.slint"
                | "data_display.slint"
                | "overlays.slint"
                | "material_bridge.slint"
                | "shared.slint"
                | "project_page_components.slint"
        ) {
            continue;
        }

        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("private property <bool> compact: root.width <")
                && !trimmed.contains("HubTokens.")
            {
                violations.push(format!("{}:{}: {trimmed}", display_path(&path), index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "page compact breakpoints must use HubTokens:\n{}",
        violations.join("\n")
    );
}

#[test]
fn absolute_positioning_stays_out_of_page_layouts() {
    let mut violations = Vec::new();
    for path in slint_files() {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if matches!(file_name, "app.slint" | "inputs.slint" | "shell.slint") {
            continue;
        }

        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("x:") || trimmed.starts_with("y:") {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "business pages should use layouts; only shell/input popup anchors may use x/y:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_layout_sizes_are_tokenized() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for value in raw_px_literals(trimmed) {
                if value > 1.0 {
                    violations.push(format!(
                        "{}:{}: {}",
                        display_path(&path),
                        index + 1,
                        trimmed
                    ));
                    break;
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI should derive layout sizes from MaterialStyleMetrics/HubTokens instead of raw px literals above 1px:\n{}",
        violations.join("\n")
    );
}

fn raw_px_literals(line: &str) -> Vec<f32> {
    let mut values = Vec::new();

    for (unit_index, _) in line.match_indices("px") {
        let prefix = &line[..unit_index];
        let mut start = prefix.len();
        while start > 0 {
            let byte = prefix.as_bytes()[start - 1];
            if byte.is_ascii_digit() || byte == b'.' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == prefix.len() {
            continue;
        }

        let literal = &prefix[start..];
        if let Ok(value) = literal.parse::<f32>() {
            values.push(value);
        }
    }

    values
}

fn display_path(path: &Path) -> String {
    path.strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or(path)
        .display()
        .to_string()
}
