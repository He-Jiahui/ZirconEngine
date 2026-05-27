//! Static contracts for Zircon Hub Material foundation and layout primitives.

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
        "HorizontalDivider",
        "Vertical",
        "VerticalDivider",
        "Horizontal",
        "Grid",
        "ScrollView",
        "ElevatedCard",
        "FilledCard",
        "OutlinedCard",
        "FilledButton",
        "IconButton",
        "ListView",
        "MaterialWindow",
        "MaterialWindowAdapter",
        "NavigationRail",
        "CircularProgressIndicator",
        "SearchBar",
        "StateLayerArea",
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
    for path in slint_files() {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if file_name == "material_bridge.slint" {
            continue;
        }
        assert!(
            !normalize_newlines(source).contains("from \"@material\""),
            "{file_name} must import Slint Material through material_bridge.slint, leaving @material as a single compiler bridge"
        );
    }

    let layout = read_ui_file("layout.slint");
    assert!(
        layout.contains(
            "import { HorizontalDivider, ScrollView, VerticalDivider } from \"material_bridge.slint\";"
        ) && layout.contains("page-scroll := ScrollView"),
        "Page must use the ScrollView component through the Hub Material bridge"
    );
    assert!(
        layout.contains("if root.vertical: VerticalDivider {")
            && layout.contains("if !root.vertical: HorizontalDivider {")
            && !layout.contains("background: root.tone;"),
        "Divider must use the direct Material HorizontalDivider/VerticalDivider primitives instead of a hand-drawn line"
    );
}

#[test]
fn hub_applies_zircon_material_theme_to_material_palette() {
    let app = read_ui_file("app.slint");
    for snippet in [
        "import { ZirconMaterialTheme } from \"theme.slint\";",
        "MaterialWindow,",
        "export component HubWindow inherits MaterialWindow",
        "ZirconMaterialTheme { }",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must inherit the imported Slint Material window and install the Zircon Material theme before rendering controls; missing {snippet}"
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
        "ResponsiveState",
        "Fill",
        "Divider",
    ] {
        assert!(
            layout.contains(&format!("component {primitive}")),
            "layout.slint must declare {primitive}"
        );
    }
    let fill = layout
        .split("export component Fill")
        .nth(1)
        .and_then(|source| source.split("export component Divider").next())
        .expect("layout.slint must declare Fill before Divider");
    for snippet in [
        "in property <length> fill-spacing: HubTokens.space-0;",
        "horizontal-stretch: 1;",
        "vertical-stretch: 1;",
        "VerticalLayout {",
        "spacing: root.fill-spacing;",
        "@children",
    ] {
        assert!(
            fill.contains(snippet),
            "Fill must own child fill layout so page hosts do not repeat parent geometry bindings; missing {snippet}"
        );
    }

    let responsive_slot = layout
        .split("export component ResponsiveSlot")
        .nth(1)
        .and_then(|source| source.split("export component ResponsiveState").next())
        .expect("layout.slint must declare ResponsiveSlot before ResponsiveState");
    for snippet in [
        "in property <length> basis: HubTokens.panel-min-md;",
        "in property <float> grow: 1;",
        "in property <float> shrink: 1;",
        "in property <int> order: 0;",
    ] {
        assert!(
            responsive_slot.contains(snippet),
            "ResponsiveSlot must keep shared Taffy sizing and ordering inputs; missing {snippet}"
        );
    }
    for forbidden in [
        "flex-basis: root.basis;",
        "flex-grow: root.grow;",
        "flex-shrink: root.shrink;",
        "flex-order: root.order;",
    ] {
        assert!(
            !responsive_slot.contains(forbidden),
            "ResponsiveSlot must not set Slint flex properties inside the component definition; use sites mirror them on direct Flexbox children: {forbidden}"
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
        "project-dashboard-toolbar-search-ratio: 0.25",
        "project-dashboard-toolbar-select-ratio: 0.111111",
        "project-dashboard-card-ratio: 0.22",
        "project-dashboard-lower-compact-ratio: 0.60",
        "project-dashboard-lower-regular-ratio: 0.35",
        "project-dashboard-table-ratio: 0.58",
        "project-new-main-stack-ratio: 0.60",
        "project-new-side-stack-ratio: 0.40",
        "project-browser-toolbar-search-ratio: 0.40",
        "project-browser-toolbar-select-ratio: 0.125",
        "project-detail-cover-ratio: 0.25",
        "project-detail-info-row-ratio: 0.05",
        "project-detail-version-badge-ratio: 0.10",
        "project-detail-pin-badge-ratio: 0.0714286",
        "project-detail-main-panel-ratio: 0.50",
        "project-detail-action-panel-ratio: 0.25",
        "project-detail-main-stack-ratio: 0.60",
        "project-detail-action-stack-ratio: 0.40",
        "workspace-row-editor-summary: root.control-md + root.list-row-sm * 3 + root.toolbar-gap * 3 + root.space-4 * 2",
        "workspace-row-editor-config: MaterialStyleMetrics.size_200 * 2 + MaterialStyleMetrics.size_72 + MaterialStyleMetrics.size_4",
        "workspace-row-build-summary: root.control-md + root.list-row-md * 5 + root.toolbar-gap * 5 + root.space-4 * 2",
        "workspace-row-build-detail: root.control-md + root.list-row-md * 4 + root.toolbar-gap * 4 + root.space-4 * 2",
        "workspace-row-settings-controls: MaterialStyleMetrics.size_200 + MaterialStyleMetrics.size_80 + MaterialStyleMetrics.size_6",
        "workspace-row-settings-detail: MaterialStyleMetrics.size_200 * 2 - MaterialStyleMetrics.size_8",
        "workspace-row-cloud-metrics: root.list-row-lg + root.space-6",
        "workspace-row-team-summary: root.list-row-lg + root.space-7",
        "breakpoint-short: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_256 + MaterialStyleMetrics.size_64",
        "shell-row-min: MaterialStyleMetrics.size_52",
        "shell-row-max: MaterialStyleMetrics.size_56",
        "nav-width-collapsed-min: MaterialStyleMetrics.size_80",
        "nav-width-collapsed-max: MaterialStyleMetrics.size_80",
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
        "MaterialText",
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

    let badge = surfaces
        .split("export component Badge")
        .nth(1)
        .and_then(|source| source.split("export component StatusBadge").next())
        .expect("surfaces.slint must declare Badge before StatusBadge");
    for snippet in [
        "MaterialText {",
        "text: root.text;",
        "style: MaterialTypography.label_medium;",
        "horizontal_alignment: center;",
        "vertical_alignment: center;",
    ] {
        assert!(
            badge.contains(snippet),
            "Badge should keep its Hub tone shell while delegating text styling to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !badge.lines().any(|line| line.trim() == "Text {"),
        "Badge should not return to a raw Text element for its label"
    );
    for forbidden in [
        "font-size: MaterialTypography.label_medium.font_size;",
        "font-weight: MaterialTypography.label_medium.font_weight;",
    ] {
        assert!(
            !badge.contains(forbidden),
            "Badge should not return to raw Text/font bindings for its label: {forbidden}"
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
        "MaterialText {",
        "style: MaterialTypography.title_medium;",
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

    let status_banner = surfaces
        .split("export component StatusBanner")
        .nth(1)
        .expect("surfaces.slint must declare StatusBanner");
    for snippet in [
        "MaterialText {",
        "style: MaterialTypography.title_small;",
        "height: HubTokens.list-row-sm;",
        "vertical-stretch: 0;",
    ] {
        assert!(
            status_banner.contains(snippet),
            "StatusBanner title should use MaterialText typography; missing {snippet}"
        );
    }
    for (name, source) in [
        ("PanelHeader", panel_header),
        ("StatusBanner", status_banner),
    ] {
        assert!(
            !source.lines().any(|line| line.trim() == "Text {"),
            "{name} title typography should not return to a raw Text element"
        );
        for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
            assert!(
                !source.contains(forbidden),
                "{name} title typography should not return to raw Text font bindings: {forbidden}"
            );
        }
    }
}
