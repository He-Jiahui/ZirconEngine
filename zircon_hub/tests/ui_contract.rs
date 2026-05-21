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

    let layout = read_ui_file("layout.slint");
    assert!(
        layout.contains(
            "import { HorizontalDivider, ScrollView, VerticalDivider } from \"@material\";"
        ) && layout.contains("page-scroll := ScrollView"),
        "Page must use the ScrollView component from the direct @material template import"
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
        "workspace-row-editor-summary: root.control-md + root.list-row-md * 3 + root.toolbar-gap * 3 + root.space-4 * 2",
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
    for snippet in ["MaterialText {", "style: MaterialTypography.title_small;"] {
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

#[test]
fn shared_hub_buttons_are_backed_by_material_button_primitives() {
    let shared = read_ui_file("shared.slint");
    for snippet in [
        "FilledButton,",
        "FilledIconButton,",
        "IconButton as MaterialIconButton,",
        "OutlineButton,",
        "TonalButton,",
        "TonalIconButton,",
        "export component PillButton",
        "FilledButton {",
        "TonalButton {",
        "export component IconButton",
        "FilledIconButton {",
        "TonalIconButton {",
        "export component WindowButton",
        "MaterialIconButton {",
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

    let window_start = shared
        .find("export component WindowButton")
        .expect("shared.slint must declare WindowButton");
    let panel_start = shared
        .find("export component Panel")
        .expect("shared.slint must declare Panel after WindowButton");
    let window_button = &shared[window_start..panel_start];
    for snippet in [
        "MaterialIconButton {",
        "icon: root.has-icon-image ? root.icon-image : @image-url(\"../assets/icons/ui/close.svg\");",
        "inline: true;",
        "has_error: root.danger;",
        "clicked =>",
    ] {
        assert!(
            window_button.contains(snippet),
            "WindowButton must delegate title-bar icon layout and interaction to Material IconButton; missing {snippet}"
        );
    }
    for forbidden in ["CenteredIcon", "area := TouchArea", "area.has-hover"] {
        assert!(
            !window_button.contains(forbidden),
            "WindowButton should not return to custom painted title-bar icon behavior: {forbidden}"
        );
    }
}

#[test]
fn shared_typography_wrappers_use_material_text() {
    let shared = read_ui_file("shared.slint");
    for snippet in [
        "MaterialText,",
        "export component FieldLabel inherits MaterialText",
        "style: MaterialTypography.label_large;",
        "export component MutedText inherits MaterialText",
        "style: MaterialTypography.body_small;",
    ] {
        assert!(
            shared.contains(snippet),
            "shared typography wrappers should delegate text metrics to MaterialText; missing {snippet}"
        );
    }

    let field_label = shared
        .split("export component FieldLabel")
        .nth(1)
        .and_then(|source| source.split("export component MutedText").next())
        .expect("shared.slint must declare FieldLabel before MutedText");
    let muted_text = shared
        .split("export component MutedText")
        .nth(1)
        .expect("shared.slint must declare MutedText");
    for forbidden in [
        "inherits Text",
        "font-size:",
        "font-weight:",
        "font_size:",
        "font_weight:",
    ] {
        assert!(
            !field_label.contains(forbidden) && !muted_text.contains(forbidden),
            "shared typography wrappers should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn expanded_nav_button_uses_material_list_tile() {
    let shared = read_ui_file("shared.slint");
    let nav_button = shared
        .split("export component NavButton")
        .nth(1)
        .and_then(|source| source.split("export component StatusPill").next())
        .expect("shared.slint must declare NavButton before StatusPill");

    for snippet in [
        "ListTile,",
        "ListTile {",
        "text: root.collapsed ? \"\" : root.item.title;",
        "supporting_text: \"\";",
        "avatar_icon: root.item.has-icon-image ? root.item.icon-image : @image-url(\"../assets/icons/nav/projects.svg\");",
        "avatar_background: transparent;",
        "avatar_foreground: root.item.active ? MaterialPalette.on_secondary_container : MaterialPalette.primary;",
        "clicked =>",
        "root.clicked(root.item.id);",
    ] {
        assert!(
            shared.contains(snippet) || nav_button.contains(snippet),
            "NavButton must preserve the Hub navigation API while delegating its row body to Material ListTile; missing {snippet}"
        );
    }

    for forbidden in [
        "area := TouchArea",
        "CenteredIcon",
        "padding-left: MaterialStyleMetrics.padding_16;",
        "font-size: MaterialTypography.label_large.font_size;",
        "background: root.item.active ? MaterialPalette.secondary_container : (area.has-hover",
    ] {
        assert!(
            !nav_button.contains(forbidden),
            "NavButton should not return to the custom painted expanded navigation row: {forbidden}"
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
fn data_display_lists_use_material_scroll_view() {
    let data_display = read_ui_file("data_display.slint");

    for snippet in [
        "ScrollView,",
        "table-scroll := ScrollView {",
        "catalog-scroll := ScrollView {",
        "export component PanelListViewport inherits ScrollView",
        "page-surface := PageScrollSurface",
        "content-height: page-surface.content-height;",
        "panel-height: max(HubTokens.list-row-lg * 4, root.content-height);",
        "viewport_y <=> root.scroll-y;",
        "viewport_width: table-scroll.visible_width;",
        "viewport_width: catalog-scroll.visible_width;",
        "viewport_width: root.visible_width;",
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
        "panel-height: max(HubTokens.list-row-lg * 4, root.height - HubTokens.page-padding * 2)",
        "root.height - HubTokens.page-padding * 2",
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
}

#[test]
fn data_display_catalog_empty_state_uses_material_text() {
    let data_display = read_ui_file("data_display.slint");
    let catalog_panel = data_display
        .split("export component CatalogListPanel")
        .nth(1)
        .and_then(|source| source.split("export component CatalogPage").next())
        .expect("data_display.slint must declare CatalogListPanel before CatalogPage");

    for snippet in [
        "if root.row-count == 0: Rectangle",
        "MaterialText {",
        "text: root.empty-title;",
        "style: MaterialTypography.label_large;",
        "if root.empty-detail != \"\": MutedText",
    ] {
        assert!(
            catalog_panel.contains(snippet),
            "CatalogListPanel empty state should use MaterialText title typography; missing {snippet}"
        );
    }

    assert!(
        !catalog_panel.lines().any(|line| line.trim() == "Text {"),
        "CatalogListPanel empty state should not return to a raw Text title"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !catalog_panel.contains(forbidden),
            "CatalogListPanel empty state should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn collapsed_nav_rail_uses_material_navigation_rail() {
    let navigation = read_ui_file("navigation.slint");
    for snippet in [
        "NavigationRail as MaterialNavigationRail",
        "in property <[NavigationItem]> material-items;",
        "in-out property <int> current-index: 0;",
        "min-width: root.collapsed ? MaterialStyleMetrics.size_80 : 0px;",
        "if root.collapsed: MaterialNavigationRail",
        "items: root.material-items;",
        "current_index <=> root.current-index;",
        "alignment: start;",
        "has_menu: false;",
        "index_changed(index) =>",
        "root.clicked(root.items[index].id);",
        "if !root.collapsed: VerticalLayout",
        "collapsed: false;",
    ] {
        assert!(
            navigation.contains(snippet),
            "collapsed NavRail must delegate to the local Material NavigationRail while expanded rows keep Hub semantics; missing {snippet}"
        );
    }

    let collapsed_start = navigation
        .find("if root.collapsed: MaterialNavigationRail")
        .expect("navigation.slint must declare the collapsed Material rail branch");
    let expanded_start = navigation
        .find("if !root.collapsed: VerticalLayout")
        .expect("navigation.slint must declare the expanded Hub row branch");
    let collapsed_branch = &navigation[collapsed_start..expanded_start];
    assert!(
        !collapsed_branch.contains("NavButton"),
        "collapsed navigation must not return to the custom NavButton loop"
    );

    let app = read_ui_file("app.slint");
    for snippet in [
        "NavigationItem } from \"components.slint\";",
        "in property <[NavigationItem]> material-nav-items;",
        "in-out property <int> selected-nav-index: 0;",
        "material-nav-items: root.material-nav-items;",
        "selected-nav-index <=> root.selected-nav-index;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must expose and forward Material navigation adapter data; missing {snippet}"
        );
    }

    let shell = read_ui_file("shell.slint");
    for snippet in [
        "in property <[NavigationItem]> material-nav-items;",
        "in-out property <int> selected-nav-index: 0;",
        "material-items: root.material-nav-items;",
        "current-index <=> root.selected-nav-index;",
    ] {
        assert!(
            shell.contains(snippet),
            "HubNavSidebar must forward Material navigation data into NavRail; missing {snippet}"
        );
    }

    let binding = read_crate_file("src/app/binding.rs");
    for snippet in [
        "let nav_items = view_model::navigation_items(",
        "ui.set_selected_nav_index(view_model::selected_nav_index(&nav_items));",
        "ui.set_material_nav_items(view_model::model_from(",
        "view_model::material_navigation_items(&nav_items)",
        "ui.set_nav_items(view_model::model_from(nav_items));",
    ] {
        assert!(
            binding.contains(snippet),
            "binding.rs must keep Material nav data derived from the same Hub nav model; missing {snippet}"
        );
    }

    let view_model = read_crate_file("src/app/view_model.rs");
    for snippet in [
        "NavigationItem,",
        "pub(super) fn material_navigation_items(items: &[NavItemData]) -> Vec<NavigationItem>",
        "selected_icon: item.icon_image.clone(),",
        "show_badge: false,",
        "pub(super) fn selected_nav_index(items: &[NavItemData]) -> i32",
    ] {
        assert!(
            view_model.contains(snippet),
            "view_model.rs must adapt Hub nav rows to Material NavigationItem without changing page business state; missing {snippet}"
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
        "enabled: root.enabled;",
        "clicked =>",
        "StatusBadge {",
        "IconButton {",
    ] {
        assert!(
            info_row.contains(snippet),
            "InfoRow must delegate its information-row body to Material ListTile; missing {snippet}"
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
        "title: root.record.detail;",
        "detail: root.record.output-path;",
        "trailing-text: root.record.status;",
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
        "text: root.ui-text.no-build-history;",
    ] {
        assert!(
            builds.contains(snippet),
            "BuildsPage must surface active Source Engine build history; missing {snippet}"
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

#[test]
fn builds_current_task_status_uses_material_text() {
    let builds = read_ui_file("builds.slint");

    for snippet in [
        "MaterialText,",
        "MaterialText {",
        "text: root.status-label;",
        "style: MaterialTypography.headline_small;",
    ] {
        assert!(
            builds.contains(snippet),
            "BuildsPage current task status should delegate typography to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !builds.lines().any(|line| line.trim() == "Text {"),
        "BuildsPage should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !builds.contains(forbidden),
            "BuildsPage should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn editor_source_engine_row_uses_material_list_tile() {
    let editor = read_ui_file("editor.slint");
    let source_engine_row = editor
        .split("component SourceEngineRow")
        .nth(1)
        .and_then(|source| source.split("component BuildHistoryRow").next())
        .expect("editor.slint must declare SourceEngineRow before BuildHistoryRow");

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
    ] {
        assert!(
            !source_engine_row.contains(forbidden),
            "SourceEngineRow should not return to a custom full-row TouchArea implementation: {forbidden}"
        );
    }
}

#[test]
fn project_workflow_pages_surface_selected_project_scope() {
    let shared = read_ui_file("shared.slint");
    assert!(
        shared.contains("scope: string,"),
        "PluginData must expose project/engine scope for Hub workflow grouping"
    );
    for snippet in [
        "asset-catalog-selected: string,",
        "asset-empty-selected-detail: string,",
        "asset-empty-global-detail: string,",
        "plugin-catalog-selected: string,",
        "plugin-empty-selected-detail: string,",
        "plugin-empty-global-detail: string,",
        "team-workspace-selected: string,",
        "team-empty-selected-detail: string,",
        "team-empty-global-detail: string,",
        "cloud-overview-selected: string,",
        "cloud-local-selected-detail: string,",
        "install-to-device: string,",
        "select-project-before-opening: string,",
        "select-project-before-packaging: string,",
        "select-project-before-installing: string,",
        "current-project: string,",
        "active-source-engine: string,",
        "no-project-selected: string,",
    ] {
        assert!(
            shared.contains(snippet),
            "UiTextData must expose selected-project-aware Assets/Plugins/Team/Cloud and Builds action copy; missing {snippet}"
        );
    }

    let assets_page = read_ui_file("assets.slint");
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.asset-catalog-selected : root.ui-text.asset-catalog",
        "root.has-selected-project ? root.ui-text.asset-empty-selected-detail : root.ui-text.asset-empty-global-detail",
        "empty-detail: root.assets-empty-detail;",
    ] {
        assert!(
            assets_page.contains(snippet),
            "AssetsPage must explain whether the list is scoped to the selected project or global local roots; missing {snippet}"
        );
    }

    let asset_projection = read_crate_file("src/app/view_model/assets.rs");
    for snippet in [
        "asset_source_priority(&left.source)",
        "SELECTED_PROJECT_ASSET_SOURCE => 0",
        "PROJECT_ASSET_SOURCE => 1",
        "Source Engine / {source}",
    ] {
        assert!(
            asset_projection.contains(snippet),
            "AssetData projection must prioritize selected-project assets and keep source-engine group labels; missing {snippet}"
        );
    }

    let asset_catalog = read_crate_file("src/assets/catalog.rs");
    for snippet in [
        "source_priority(&left.source)",
        "SELECTED_PROJECT_ASSET_SOURCE => 0",
        "PROJECT_ASSET_SOURCE => 1",
    ] {
        assert!(
            asset_catalog.contains(snippet),
            "Asset catalog discovery must emit selected-project assets before project/engine groups; missing {snippet}"
        );
    }

    let plugins = read_ui_file("plugins.slint");
    assert!(
        plugins.contains("root.plugin.scope +"),
        "PluginsPage rows must display whether a plugin came from the selected project or engine"
    );
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.plugin-catalog-selected : root.ui-text.plugin-catalog",
        "root.has-selected-project ? root.ui-text.plugin-empty-selected-detail : root.ui-text.plugin-empty-global-detail",
        "empty-detail: root.plugins-empty-detail;",
    ] {
        assert!(
            plugins.contains(snippet),
            "PluginsPage must explain whether discovery includes selected-project plugin manifests; missing {snippet}"
        );
    }

    let plugin_projection = read_crate_file("src/app/view_model/plugins.rs");
    for snippet in [
        "PROJECT_PLUGIN_SCOPE =>",
        "ENGINE_PLUGIN_SCOPE =>",
        "\"Selected Project\"",
        "\"Source Engine\"",
    ] {
        assert!(
            plugin_projection.contains(snippet),
            "PluginData projection must label selected-project and Source Engine plugin scopes; missing {snippet}"
        );
    }

    let team = read_ui_file("team.slint");
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.team-workspace-selected : root.ui-text.team-workspace",
        "root.has-selected-project ? root.ui-text.team-empty-selected-detail : root.ui-text.team-empty-global-detail",
        "title: root.workspace-title;",
        "text: root.members-empty-detail;",
    ] {
        assert!(
            team.contains(snippet),
            "TeamPage must explain whether local Git data is scoped to the selected project or Source Engine fallback; missing {snippet}"
        );
    }

    let builds = read_ui_file("builds.slint");
    for snippet in [
        "import { MutedText, ProjectDetailData, SourceBuildHistoryRowData, SourceEngineData, UiTextData } from \"shared.slint\";",
        "in property <ProjectDetailData> project;",
        "root.project.can-open ? root.project.title : root.ui-text.select-project-before-opening",
        "root.project.project-path != \"\" ? root.project.project-path : root.ui-text.reserved-project-export",
        "callback package-project();",
        "callback install-device();",
        "id: \"package-project\"",
        "id: \"install-device\"",
        "title: root.ui-text.install-to-device,",
        "root.project.can-open ? root.project.title : root.ui-text.select-project-before-packaging",
        "root.project.can-open ? root.project.title : root.ui-text.select-project-before-installing",
        "enabled: root.project.can-open,",
    ] {
        assert!(
            builds.contains(snippet),
            "BuildsPage must surface selected-project build/package context; missing {snippet}"
        );
    }

    let cloud_page = read_ui_file("cloud.slint");
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.cloud-overview-selected : root.ui-text.cloud-overview",
        "root.has-selected-project ? root.ui-text.cloud-local-selected-detail : root.ui-text.cloud-local-only",
        "title: root.overview-title;",
        "subtitle: root.overview-detail;",
    ] {
        assert!(
            cloud_page.contains(snippet),
            "CloudPage must make local package/install/output status read as selected-project scoped when a project is selected; missing {snippet}"
        );
    }

    let app = read_ui_file("app.slint");
    assert!(
        app.contains("project: root.project-detail;"),
        "HubWindow must pass selected project detail into BuildsPage"
    );
    for snippet in [
        "package-project => { root.quick-action(\"package-project\"); }",
        "install-device => { root.quick-action(\"install-device\"); }",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must route Builds page package/install controls through the existing quick-action dispatcher; missing {snippet}"
        );
    }
    assert!(
        app.contains("has-selected-project: root.project-detail.selected;"),
        "HubWindow must pass selected project state into AssetsPage/PluginsPage empty-state copy"
    );

    let project_projection = read_crate_file("src/app/view_model/projects.rs");
    for snippet in [
        "return empty_project_detail(snapshot.settings.language);",
        "fn empty_project_detail(language: HubLanguage) -> ProjectDetailData",
        "\"No project selected\",",
        "\"Select a project to package or launch\",",
        "can_open: false,",
        "can_delete: false,",
    ] {
        assert!(
            project_projection.contains(snippet),
            "ProjectDetailData must remain an explicit no-selection state until selected_project_path resolves; missing {snippet}"
        );
    }
    assert!(
        !project_projection.contains(".or_else(|| project_browser_projects(snapshot).into_iter().next())"),
        "ProjectDetailData must not silently fall back to the first project; only quick actions use latest-recent fallback"
    );

    let binding = read_crate_file("src/app/binding.rs");
    assert!(
        binding.contains("view_model::quick_actions(\n        snapshot, language,"),
        "binding.rs must derive QuickActionData from HubSnapshot so quick actions can describe selected/recent project scope"
    );

    let view_model = read_crate_file("src/app/view_model/quick_actions.rs");
    for snippet in [
        "quick_action_project_target(snapshot)",
        "QuickActionProjectTarget::LatestRecent",
        "Build latest recent project's Source Engine",
        "Select or add a project before building",
        "Package latest recent project",
        "Select or add a project before packaging",
        "quick_action_enabled(action, &project_target)",
    ] {
        assert!(
            view_model.contains(snippet),
            "QuickActionData projection must describe selected/latest project targeting and disable project-only actions without a project; missing {snippet}"
        );
    }

    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    for snippet in [
        "pub(super) fn selected_or_latest_recent_project_for_action(",
        "let selected_before = self.selected_project_path.clone();",
        "let active_engine_before = self.config.active_engine_id.clone();",
        "let project = self.selected_or_latest_recent_project();",
        "self.activate_project_engine_for_path(&project.path);",
        "self.config.active_engine_id != active_engine_before",
        "self.refresh_selected_project_scoped_views()?;",
    ] {
        assert!(
            workspace.contains(snippet),
            "Quick action latest-recent fallback must refresh selected-project scoped views; missing {snippet}"
        );
    }
    assert!(
        workspace
            .matches("selected_or_latest_recent_project_for_action()?")
            .count()
            >= 2,
        "Package and Open Editor quick actions must use the refreshed selected/latest project helper"
    );
    let runtime = read_crate_file("src/app/runtime.rs");
    for snippet in [
        "Some(HubQuickAction::BuildProject) => self.build_selected_project_engine(ui)",
        "fn build_selected_project_engine(&mut self, ui: &HubWindow) -> Result<(), HubError>",
        "self.selected_or_latest_recent_project_for_action()?",
        "self.build_editor_runtime_after_sync(ui)",
    ] {
        assert!(
            runtime.contains(snippet),
            "Build Project quick action must target the selected/latest project's bound Source Engine before building; missing {snippet}"
        );
    }
    let open_editor = workspace
        .split("pub(super) fn open_selected_project_or_editor")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn create_project").next())
        .expect("project_workspace.rs must declare open_selected_project_or_editor before create_project");
    assert!(
        open_editor.contains("selected_or_latest_recent_project_for_action()?"),
        "Open Editor quick action must use the same selected-project/latest-recent target rule as package/install"
    );
    assert!(
        !open_editor.contains("selected_recent_project()"),
        "Open Editor quick action must not fall back to no-project editor launch while recent projects exist"
    );
    let confirm_delete = workspace
        .split("pub(super) fn confirm_delete_project")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn activate_project_engine_for_path")
                .next()
        })
        .expect("project_workspace.rs must declare confirm_delete_project before activate_project_engine_for_path");
    for snippet in [
        "self.remove_project_from_hub_path(&path);",
        "self.project_subpage = ProjectSubpage::ProjectBrowser;",
        "self.refresh_selected_project_scoped_views()?;",
    ] {
        assert!(
            confirm_delete.contains(snippet),
            "Confirm Delete must clear selected-project scoped Assets/Plugins/Team/Cloud views after removing the selected project; missing {snippet}"
        );
    }
    let remove_project = workspace
        .split("pub(super) fn remove_project_from_hub_path")
        .nth(1)
        .and_then(|source| source.split("fn require_engine").next())
        .expect(
            "project_workspace.rs must declare remove_project_from_hub_path before require_engine",
        );
    for snippet in [
        "self.pending_delete_project_path",
        ".is_some_and(|pending| pending == path)",
        "self.pending_delete_project_path = None;",
        "self.selected_project_path = None;",
    ] {
        assert!(
            remove_project.contains(snippet),
            "Removing a project from Hub must clear both selected-project and pending-delete state for the same path; missing {snippet}"
        );
    }
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
fn cloud_package_status_uses_package_manifest_for_selected_project_scope() {
    let cloud = read_crate_file("src/app/view_model/cloud.rs");
    for snippet in [
        "const PACKAGE_MANIFEST_FILE: &str = \"zircon-package.toml\";",
        "const PACKAGE_SOURCE_PROJECT_KEY: &str = \"source_project\";",
        "selected_project_package_count(package_root, path)",
        "package_matches_selected_project",
        "manifest.parse::<toml::Value>()",
        "paths_match_for_summary(Path::new(source_project), selected_project_path)",
        "{package_count} local {noun} for selected project",
    ] {
        assert!(
            cloud.contains(snippet),
            "Cloud package status must scope selected-project packages through zircon-package.toml source_project; missing {snippet}"
        );
    }
}

#[test]
fn app_window_routes_shell_chrome_through_components() {
    let app = read_ui_file("app.slint");
    let page_header_call = app
        .split("HubPageHeader {")
        .nth(1)
        .and_then(|source| source.split("ProjectsPage {").next())
        .expect("app.slint must compose HubPageHeader before routed pages");
    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "context-compact: root.status-compact;",
        "context-badge-width: HubTokens.status-badge-width;",
    ] {
        assert!(
            page_header_call.contains(snippet),
            "HubPageHeader must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }

    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "compact: root.status-compact;",
        "context-badge-width: HubTokens.status-badge-width;",
    ] {
        assert!(
            app.contains(snippet),
            "HubStatusBar must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }

    let top_header_call = app
        .split("HubTopHeader {")
        .nth(1)
        .and_then(|source| source.split("HubNavSidebar {").next())
        .expect("app.slint must compose HubTopHeader before HubNavSidebar");
    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
    ] {
        assert!(
            top_header_call.contains(snippet),
            "HubTopHeader must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }

    let nav_sidebar_call = app
        .split("HubNavSidebar {")
        .nth(1)
        .and_then(|source| source.split("clicked(id) =>").next())
        .expect("app.slint must compose HubNavSidebar before routed pages");
    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
    ] {
        assert!(
            nav_sidebar_call.contains(snippet),
            "HubNavSidebar must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }
    assert!(
        app.contains("private property <length> nav-status-height: root.shell-row-height * 4;"),
        "app.slint must leave enough token-derived sidebar status height for project and engine context"
    );

    let shell = read_ui_file("shell.slint");
    let page_header = shell
        .split("export component HubPageHeader")
        .nth(1)
        .and_then(|source| source.split("export component HubStatusBar").next())
        .expect("shell.slint must declare HubPageHeader before HubStatusBar");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <bool> context-compact: false;",
        "in property <length> context-badge-width: HubTokens.status-badge-width;",
        "root.project.selected ? root.project.title : root.ui-text.no-project-selected",
        "root.source-engine.title == \"\" ? root.ui-text.no-source-engines : root.source-engine.title",
        "if root.selected-page != \"projects\" && !root.context-compact: HorizontalLayout",
        "badge-width: root.context-badge-width;",
        "MaterialText {",
        "text: root.selected-page-title;",
        "style: MaterialTypography.headline_medium;",
        "text: root.status-label;",
        "style: MaterialTypography.label_large;",
    ] {
        assert!(
            page_header.contains(snippet),
            "HubPageHeader must surface selected-project and active Source Engine context on non-Projects pages without compact overlap; missing {snippet}"
        );
    }
    assert!(
        !page_header.contains("root.width <") && !page_header.contains("root.width /"),
        "HubPageHeader must receive compact state and token badge width from app.slint instead of deriving layout from its own width"
    );
    assert!(
        !page_header.contains("font-size:") && !page_header.contains("font-weight:"),
        "HubPageHeader typography should stay on MaterialText styles instead of raw Text font bindings"
    );

    let status_bar = shell
        .split("export component HubStatusBar")
        .nth(1)
        .expect("shell.slint must declare HubStatusBar");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "in property <bool> compact: false;",
        "in property <length> context-badge-width: HubTokens.status-badge-width;",
        "root.project.selected ? root.project.title : root.ui-text.no-project-selected",
        "root.ui-text.active-source-engine + \": \" + root.source-engine.title",
        "if !root.compact: Badge",
        "badge-width: root.context-badge-width;",
        "tone: root.project.selected ? \"accent\" : \"neutral\";",
        "MaterialText {",
        "text: root.status-detail;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            status_bar.contains(snippet),
            "HubStatusBar must surface selected-project and active Source Engine badges without crowding compact widths; missing {snippet}"
        );
    }
    assert!(
        !status_bar.contains("root.width <") && !status_bar.contains("root.width /"),
        "HubStatusBar must not derive responsive layout from its own resolved width; app.slint should pass compact state and token badge width"
    );
    assert!(
        !status_bar.lines().any(|line| line.trim() == "Text {") && !status_bar.contains("font-size:"),
        "HubStatusBar status detail should stay on MaterialText typography instead of raw Text font bindings"
    );

    let engine_selector = shell
        .split("component HeaderEngineSelector")
        .nth(1)
        .and_then(|source| source.split("export component HubTopHeader").next())
        .expect("shell.slint must declare HeaderEngineSelector before HubTopHeader");
    for snippet in [
        "MaterialText {",
        "text: root.ui-text.source-engines;",
        "style: MaterialTypography.label_large;",
        "text: root.ui-text.registered;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            engine_selector.contains(snippet),
            "HeaderEngineSelector popup chrome should use MaterialText typography; missing {snippet}"
        );
    }

    let top_header = shell
        .split("export component HubTopHeader")
        .nth(1)
        .and_then(|source| source.split("export component HubNavSidebar").next())
        .expect("shell.slint must declare HubTopHeader before HubNavSidebar");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "MaterialText {",
        "text: \"ZIRCON HUB\";",
        "style: MaterialTypography.title_medium;",
        "root.project.selected ? root.project.title : root.ui-text.game-engine",
        "text: root.brand-subtitle;",
        "text: root.ui-text.local-user-initials;",
        "style: MaterialTypography.label_medium_prominent;",
        "text: root.ui-text.local-user;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            top_header.contains(snippet),
            "HubTopHeader brand and user chrome should use MaterialText typography; missing {snippet}"
        );
    }

    let nav_sidebar = shell
        .split("export component HubNavSidebar")
        .nth(1)
        .and_then(|source| source.split("export component HubPageHeader").next())
        .expect("shell.slint must declare HubNavSidebar before HubPageHeader");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "MaterialText {",
        "text: root.ui-text.engine-status;",
        "style: MaterialTypography.label_large;",
        "root.project.selected ? root.project.title : root.ui-text.no-project-selected",
        "text: root.ui-text.current-project;",
        "text: root.project-title;",
        "color: root.project.selected ? MaterialPalette.on_surface : MaterialPalette.on_surface_variant;",
        "text: root.ui-text.check-for-updates;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            nav_sidebar.contains(snippet),
            "HubNavSidebar status chrome should use MaterialText typography; missing {snippet}"
        );
    }

    for (name, source) in [
        ("HeaderEngineSelector", engine_selector),
        ("HubTopHeader", top_header),
        ("HubNavSidebar", nav_sidebar),
    ] {
        assert!(
            !source.lines().any(|line| {
                let trimmed = line.trim();
                trimmed == "Text {" || trimmed.ends_with(": Text {")
            }) && !source.contains("font-size:")
                && !source.contains("font-weight:"),
            "{name} shell typography should not return to raw Text font bindings"
        );
    }

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
fn sidebar_collapse_uses_material_state_layer() {
    let shell = read_ui_file("shell.slint");
    let sidebar = shell
        .split("export component HubNavSidebar")
        .nth(1)
        .and_then(|source| source.split("export component HubPageHeader").next())
        .expect("shell.slint must declare HubNavSidebar before HubPageHeader");

    for snippet in [
        "StateLayerArea,",
        "collapse-state := StateLayerArea {",
        "border_radius: MaterialStyleMetrics.border_radius_12;",
        "root.toggle-collapse();",
    ] {
        assert!(
            shell.contains(snippet) || sidebar.contains(snippet),
            "HubNavSidebar collapse control must use Material StateLayerArea; missing {snippet}"
        );
    }

    for forbidden in ["collapse-area := TouchArea", "collapse-area.has-hover"] {
        assert!(
            !sidebar.contains(forbidden),
            "HubNavSidebar collapse control should not return to custom hover/click handling: {forbidden}"
        );
    }

    assert!(
        shell.matches("TouchArea").count() <= 1 && shell.contains("drag-area := TouchArea"),
        "shell.slint should reserve TouchArea for window dragging after Materializing collapse controls"
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
    for page in [
        "editor.slint",
        "builds.slint",
        "settings.slint",
        "cloud.slint",
        "team.slint",
    ] {
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
        assert!(
            source.contains("page-surface := PageScrollSurface")
                && source.contains("content-width: page-surface.content-width;"),
            "{page} should derive content width from PageScrollSurface instead of recomputing page padding"
        );
        assert!(
            !source.contains("root.width - HubTokens.page-padding * 2"),
            "{page} should not return to page-local content-width subtraction"
        );
        assert!(
            !source.contains("width: root.content-width;"),
            "{page} should let PageScrollSurface and stretch layout size workspace rows instead of assigning width directly"
        );
    }

    let layout = read_ui_file("layout.slint");
    for snippet in [
        "out property <length> content-width: max(1px, root.width - root.page-padding * 2);",
        "out property <length> content-height: max(HubTokens.control-md, root.viewport-height - root.page-padding * 2);",
        "in property <bool> compact-stack: true;",
        "in property <int> compact-rows: 2;",
        "preferred-width: 0px;",
        "flex-direction: root.compact && root.compact-stack ? FlexboxLayoutDirection.column : FlexboxLayoutDirection.row;",
        "flex-wrap: root.compact && root.compact-stack ? FlexboxLayoutWrap.no-wrap : FlexboxLayoutWrap.wrap;",
    ] {
        assert!(
            layout.contains(snippet),
            "WorkspacePanelSection must support compact stacking and compact wrapped metric rows; missing {snippet}"
        );
    }

    let cloud = read_ui_file("cloud.slint");
    assert!(
        cloud.contains("compact-stack: false;")
            && cloud.contains("compact-rows: root.metric-row-count;")
            && cloud.contains("metric-row-height: root.metrics-compact ? HubTokens.list-row-md + HubTokens.space-2 : HubTokens.workspace-row-cloud-metrics;")
            && cloud.contains("row-height: root.metric-row-height;")
            && cloud.contains("metrics-four-columns:")
            && cloud.contains("metrics-two-columns:")
            && cloud.contains("metric-row-count:")
            && cloud.contains("metric-slot-basis: root.metrics-four-columns ? root.metric-min-width : (root.metrics-two-columns ? HubTokens.panel-min-md : root.content-width);")
            && cloud.contains("metric-slot-min-width:")
            && cloud.contains("metric-slot-grow: 1;")
            && cloud.contains("basis: root.metric-slot-basis;")
            && cloud.contains("grow: root.metric-slot-grow;")
            && cloud.contains("min-width: root.metric-slot-min-width;")
            && cloud.contains("content-height: page-surface.content-height;")
            && cloud.contains("services-panel-height: max(HubTokens.list-row-lg * 3, root.content-height - root.header-height - root.metric-section-height - HubTokens.panel-gap * 2);"),
        "CloudPage should use wrapped WorkspacePanelSection metrics and PageScrollSurface content height for list sizing"
    );
    for forbidden in [
        "metric-card-width",
        "root.content-width - HubTokens.panel-gap",
        "root.content-width - root.metric-gap",
        "root.content-width - root.metric-gap *",
        "root.height - HubTokens.page-padding",
        "root.height - HubTokens.page-padding * 2",
        "root.height - HubTokens.page-padding * 2 - HubTokens.bottom-status-height",
    ] {
        assert!(
            !cloud.contains(forbidden),
            "CloudPage should not return to page-local metric/list sizing formulas: {forbidden}"
        );
    }

    let team = read_ui_file("team.slint");
    assert!(
        team.contains("row-height: HubTokens.workspace-row-team-summary;")
            && team.contains("content-height: page-surface.content-height;")
            && team.contains("members-panel-height: max(root.member-row-height * 3, root.content-height - root.header-height - root.summary-section-height - HubTokens.panel-gap * 2);"),
        "TeamPage summary cards and member list should use tokenized row height plus PageScrollSurface content height"
    );
    for forbidden in [
        "root.height - HubTokens.page-padding",
        "root.height - HubTokens.page-padding * 2",
        "root.height - HubTokens.page-padding * 2 - HubTokens.bottom-status-height",
    ] {
        assert!(
            !team.contains(forbidden),
            "TeamPage should not return to page-local member-list height formulas: {forbidden}"
        );
    }

    let editor = read_ui_file("editor.slint");
    let settings = read_ui_file("settings.slint");
    let builds = read_ui_file("builds.slint");
    for forbidden in ["flex-basis:", "flex-grow:", "flex-shrink:"] {
        for (page, source) in [
            ("EditorPage", &editor),
            ("SettingsPage", &settings),
            ("BuildsPage", &builds),
        ] {
            assert!(
                !source.contains(forbidden),
                "{page} should route Taffy sizing through ResponsiveSlot instead of direct {forbidden}"
            );
        }
    }
    for snippet in [
        "side-panel-min-width: HubTokens.panel-min-md + HubTokens.control-lg * 2;",
        "overview-min-width: HubTokens.panel-min-lg + HubTokens.control-lg * 2;",
        "basis: root.overview-min-width;",
        "basis: root.side-panel-min-width;",
        "grow: 2;",
        "grow: 1;",
        "min-width: root.compact ? root.content-width : root.overview-min-width;",
        "min-width: root.compact ? root.content-width : root.side-panel-min-width;",
    ] {
        assert!(
            editor.contains(snippet),
            "EditorPage is missing tokenized ResponsiveSlot sizing snippet: {snippet}"
        );
    }
    for forbidden in [
        "summary-main-width",
        "summary-side-width",
        "root.content-width - root.summary-side-width",
        "split-content-width",
        "overview-width",
        "actions-width",
        "config-width",
        "root.split-content-width",
        "root.content-width - root.side-panel-min-width",
    ] {
        assert!(
            !editor.contains(forbidden),
            "EditorPage should not return to page-local remaining width formulas: {forbidden}"
        );
    }
    for snippet in [
        "basis: HubTokens.panel-min-md + HubTokens.control-sm;",
        "row-preferred-width: HubTokens.input-width + HubTokens.control-md * 3 + HubTokens.toolbar-gap;",
        "basis: HubTokens.panel-min-lg + HubTokens.control-lg;",
        "basis: HubTokens.panel-min-md;",
        "grow: 2;",
        "grow: 1;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-lg;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-md;",
    ] {
        assert!(
            settings.contains(snippet),
            "SettingsPage is missing ResponsiveSlot sizing snippet: {snippet}"
        );
    }
    for forbidden in [
        "detail-side-width",
        "detail-primary-width",
        "detail-path-list-width",
        "root.content-width - root.detail-side-width",
        "path-list-width: root.detail-path-list-width",
        "path-list-width:",
        "row-preferred-width: root.path-list-width",
        "root.width - MaterialStyleMetrics.padding_16",
    ] {
        assert!(
            !settings.contains(forbidden),
            "SettingsPage should not return to page-local remaining width formulas: {forbidden}"
        );
    }
    for snippet in [
        "side-panel-min-width: HubTokens.panel-min-md + HubTokens.control-lg * 2;",
        "overview-min-width: HubTokens.panel-min-lg + HubTokens.control-lg * 2;",
        "basis: root.overview-min-width;",
        "basis: root.side-panel-min-width;",
        "grow: 2;",
        "grow: 1;",
        "min-width: root.compact ? root.content-width : root.overview-min-width;",
        "min-width: root.compact ? root.content-width : root.side-panel-min-width;",
    ] {
        assert!(
            builds.contains(snippet),
            "BuildsPage is missing tokenized ResponsiveSlot sizing snippet: {snippet}"
        );
    }
    for forbidden in [
        "detail-main-width",
        "detail-side-width",
        "root.content-width - root.detail-side-width",
        "split-content-width",
        "overview-width",
        "side-panel-width",
        "root.split-content-width",
    ] {
        assert!(
            !builds.contains(forbidden),
            "BuildsPage should not return to page-local remaining width formulas: {forbidden}"
        );
    }

    let dashboard = read_ui_file("project_dashboard.slint");
    let project_pages = read_ui_file("project_pages.slint");
    for forbidden in ["flex-basis:", "flex-grow:", "flex-shrink:"] {
        for (page, source) in [
            ("ProjectDashboardPage", &dashboard),
            ("Project secondary pages", &project_pages),
        ] {
            assert!(
                !source.contains(forbidden),
                "{page} should use Flow item preferred sizes and ResponsiveSlot wrappers instead of direct {forbidden}"
            );
        }
    }
    for snippet in [
        "for card in root.project-cards: ProjectCard",
        "card-width-basis: root.card-basis;",
        "dashboard-toolbar-select-basis: root.dashboard-toolbar-wrap ? root.toolbar-control-min-width",
        "basis: root.dashboard-toolbar-select-basis;",
        "min-width: root.dashboard-toolbar-select-basis;",
        "dashboard-card-basis: max(HubTokens.panel-min-sm * 2 / 3, root.content-width * 23 / 100);",
        "dashboard-main-basis: HubTokens.panel-min-lg + HubTokens.control-lg;",
        "dashboard-side-basis: HubTokens.panel-min-md;",
        "basis: root.compact ? root.content-width : root.dashboard-main-basis;",
        "basis: root.compact ? root.content-width : root.dashboard-side-basis;",
        "grow: 2;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-md;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-sm;",
    ] {
        assert!(
            dashboard.contains(snippet),
            "ProjectDashboardPage is missing dashboard Taffy sizing snippet: {snippet}"
        );
    }
    assert!(
        !dashboard.contains("dashboard-column-width"),
        "ProjectDashboardPage lower panels should not return to page-local remaining width formulas"
    );
    for forbidden in [
        "dashboard-toolbar-select-width",
        "root.content-width - root.toolbar-height",
        "root.content-width - root.page-gap * 3",
    ] {
        assert!(
            !dashboard.contains(forbidden),
            "ProjectDashboardPage should not return to toolbar/card remaining-width formulas: {forbidden}"
        );
    }
    for snippet in [
        "narrow-flow: root.content-width < HubTokens.panel-min-lg + HubTokens.panel-min-md + root.page-gap;",
        "flex-wrap: root.narrow-flow ? FlexboxLayoutWrap.wrap : FlexboxLayoutWrap.no-wrap;",
        "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
        "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-md;",
        "basis: root.toolbar-wrap ? root.content-width : root.content-width * 2 / 5;",
        "toolbar-select-basis: root.toolbar-wrap ? root.toolbar-control-min-width",
        "basis: root.toolbar-select-basis;",
        "min-width: root.toolbar-select-basis;",
        "viewport_y <=> root.scroll-y;",
    ] {
        assert!(
            project_pages.contains(snippet),
            "Project secondary pages are missing ResponsiveSlot/scroll sizing snippet: {snippet}"
        );
    }
    for forbidden in [
        "column-width",
        "toolbar-select-width",
        "root.content-width - root.page-gap",
    ] {
        assert!(
            !project_pages.contains(forbidden),
            "Project secondary pages should not return to page-local remaining width formulas: {forbidden}"
        );
    }
}

#[test]
fn cloud_and_team_workspace_typography_uses_material_text() {
    let cloud = read_ui_file("cloud.slint");
    let team = read_ui_file("team.slint");

    let cloud_metric = cloud
        .split("component CloudMetric")
        .nth(1)
        .and_then(|source| source.split("component CloudServiceRow").next())
        .expect("cloud.slint must declare CloudMetric before CloudServiceRow");
    for snippet in [
        "MaterialText,",
        "MaterialText {",
        "text: root.status;",
        "style: MaterialTypography.title_small;",
    ] {
        assert!(
            cloud.contains(snippet) || cloud_metric.contains(snippet),
            "CloudMetric should delegate status typography to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !cloud_metric.lines().any(|line| line.trim() == "Text {"),
        "CloudMetric should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !cloud_metric.contains(forbidden),
            "CloudMetric should not return to raw Text font bindings: {forbidden}"
        );
    }

    let team_summary = team
        .split("component TeamSummaryItem")
        .nth(1)
        .and_then(|source| source.split("component TeamSummaryCard").next())
        .expect("team.slint must declare TeamSummaryItem before TeamSummaryCard");
    let team_empty = team
        .split("if root.member-count == 0: HubPanel")
        .nth(1)
        .and_then(|source| source.split("for member in root.members").next())
        .expect("team.slint must declare member empty state before member rows");
    for (name, source, snippets) in [
        (
            "TeamSummaryItem",
            team_summary,
            &[
                "MaterialText {",
                "text: root.primary;",
                "style: MaterialTypography.title_small;",
            ][..],
        ),
        (
            "Team empty state",
            team_empty,
            &[
                "MaterialText {",
                "text: root.ui-text.no-team-members-found;",
                "style: MaterialTypography.label_large;",
            ][..],
        ),
    ] {
        for snippet in snippets {
            assert!(
                team.contains("MaterialText,") && source.contains(snippet),
                "{name} should delegate typography to MaterialText; missing {snippet}"
            );
        }
        assert!(
            !source.lines().any(|line| line.trim() == "Text {"),
            "{name} should not return to raw Text nodes"
        );
        for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
            assert!(
                !source.contains(forbidden),
                "{name} should not return to raw Text font bindings: {forbidden}"
            );
        }
    }
}

#[test]
fn project_pages_use_material_scroll_view() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_pages = read_ui_file("project_pages.slint");

    for (page, source) in [
        ("ProjectDashboardPage", &dashboard),
        ("Project secondary pages", &project_pages),
    ] {
        for snippet in [
            "ScrollView,",
            "vertical_scrollbar_policy: ScrollBarPolicy.as-needed;",
            "horizontal_scrollbar_policy: ScrollBarPolicy.always-off;",
        ] {
            assert!(
                source.contains(snippet),
                "{page} must route page/list scrolling through the Material ScrollView API; missing {snippet}"
            );
        }
        for forbidden in [
            "std-widgets.slint",
            "viewport-y <=>",
            "visible-width",
            "visible-height",
            "mouse-drag-pan-enabled",
        ] {
            assert!(
                !source.contains(forbidden),
                "{page} should not return to std-widgets ScrollView properties: {forbidden}"
            );
        }
    }

    for (page, source) in [
        ("ProjectDashboardPage", &dashboard),
        ("Project secondary pages", &project_pages),
    ] {
        for snippet in [
            "PageScrollSurface,",
            "page-surface := PageScrollSurface {",
            "scroll-y <=> root.scroll-y;",
            "content-width: page-surface.content-width;",
            "page-padding: root.page-pad;",
            "bottom-padding: root.page-pad;",
        ] {
            assert!(
                source.contains(snippet),
                "{page} must route top-level page scrolling through the shared Material PageScrollSurface; missing {snippet}"
            );
        }
        for forbidden in [
            "content-width: max(1px, root.width",
            "root.width - root.page-pad",
            "page-scroll := ScrollView",
            "dashboard-scroll := ScrollView",
            "width: root.content-width;",
        ] {
            assert!(
                !source.contains(forbidden),
                "{page} should derive page content sizing from PageScrollSurface instead of hand-written page formulas: {forbidden}"
            );
        }
    }

    for snippet in [
        "action-scroll := ScrollView {",
        "viewport_y <=> root.quick-actions-scroll-y;",
        "viewport_width: action-scroll.visible_width;",
    ] {
        assert!(
            dashboard.contains(snippet),
            "ProjectDashboardPage must keep dashboard and quick-action scrolling on Material ScrollView; missing {snippet}"
        );
    }

    for snippet in [
        "browser-scroll := ScrollView {",
        "viewport_y <=> root.scroll-y;",
        "viewport_width: browser-scroll.visible_width;",
    ] {
        assert!(
            project_pages.contains(snippet),
            "Project Browser must keep list scrolling on Material ScrollView while New/Detail use PageScrollSurface; missing {snippet}"
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
        "root.select(root.project.project-path);",
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
        "root.select(root.project.project-path);",
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
fn dashboard_project_card_and_empty_titles_use_material_text() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_card = dashboard
        .split("component ProjectCard")
        .nth(1)
        .and_then(|source| source.split("component ProjectFlow").next())
        .expect("project_dashboard.slint must declare ProjectCard before ProjectFlow");
    for snippet in [
        "MaterialText,",
        "MaterialText {",
        "text: root.project.title;",
        "style: MaterialTypography.title_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            dashboard.contains(snippet) || project_card.contains(snippet),
            "ProjectCard title should delegate typography to MaterialText; missing {snippet}"
        );
    }
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !project_card.contains(forbidden),
            "ProjectCard title should not return to raw Text font bindings: {forbidden}"
        );
    }

    let project_flow = dashboard
        .split("component ProjectFlow")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDashboardPage").next())
        .expect("project_dashboard.slint must declare ProjectFlow before ProjectDashboardPage");
    for snippet in [
        "if root.project-card-count == 0: HubPanel",
        "MaterialText {",
        "text: root.ui-text.projects-empty-title;",
        "style: MaterialTypography.title_medium;",
        "MutedText {",
    ] {
        assert!(
            project_flow.contains(snippet),
            "ProjectFlow empty state title should delegate typography to MaterialText; missing {snippet}"
        );
    }
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !project_flow.contains(forbidden),
            "ProjectFlow empty state title should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn project_workflow_typography_uses_material_text() {
    let components = read_ui_file("project_page_components.slint");
    let project_pages = read_ui_file("project_pages.slint");

    for snippet in [
        "MaterialText,",
        "style: MaterialTypography.title_large;",
        "style: MaterialTypography.label_medium;",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            components.contains(snippet),
            "Project workflow shared components should delegate typography to MaterialText; missing {snippet}"
        );
    }

    for component_name in [
        "PageHeader",
        "ProjectSettingSummaryRow",
        "ProjectBrowserRow",
    ] {
        let component = components
            .split(&format!("export component {component_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| {
                panic!("project_page_components.slint must declare {component_name}")
            });
        assert!(
            component.contains("MaterialText {"),
            "{component_name} should use MaterialText for visible text"
        );
        assert!(
            !component.lines().any(|line| line.trim() == "Text {"),
            "{component_name} should not return to raw Text nodes"
        );
        for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
            assert!(
                !component.contains(forbidden),
                "{component_name} should not return to raw Text font bindings: {forbidden}"
            );
        }
    }

    for snippet in [
        "MaterialText,",
        "text: root.ui-text.source-engine;",
        "text: root.ui-text.modified-prefix + root.project.modified;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            project_pages.contains(snippet),
            "Project workflow pages should use MaterialText for section/status labels; missing {snippet}"
        );
    }
    assert!(
        !project_pages.lines().any(|line| line.trim() == "Text {"),
        "project_pages.slint should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !project_pages.contains(forbidden),
            "project_pages.slint should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn project_browser_row_selects_and_detail_button_opens_detail() {
    let components = read_ui_file("project_page_components.slint");
    let browser_row = components
        .split("export component ProjectBrowserRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("project_page_components.slint must own ProjectBrowserRow");
    assert!(
        browser_row
            .matches("root.select(root.project.project-path);")
            .count()
            >= 1,
        "ProjectBrowserRow row state must still select the non-detail row body"
    );
    assert_eq!(
        browser_row
            .matches("root.open-detail(root.project.project-path);")
            .count(),
        1,
        "ProjectBrowserRow detail navigation should be limited to the trailing detail hit branch"
    );
    assert!(
        browser_row.contains("StateLayerArea {")
            && browser_row.contains("row-state :=")
            && browser_row.contains("border_radius: MaterialStyleMetrics.border_radius_12;"),
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
        state_body.contains("root.select(root.project.project-path);"),
        "ProjectBrowserRow StateLayerArea should select clicks outside the detail zone"
    );
    assert!(
        state_body.contains("horizontal-stretch: 1;") && state_body.contains("min-width: 1px;"),
        "ProjectBrowserRow StateLayerArea should let layout reserve the trailing detail zone instead of subtracting row width by hand"
    );
    assert!(
        !state_body.contains("row-state.mouse-x")
            && !state_body.contains("root.open-detail(root.project.project-path);"),
        "ProjectBrowserRow row StateLayerArea should not infer detail clicks from mouse coordinates"
    );
    assert!(
        browser_row.contains("detail-hit-width: max(root.row-height, root.detail-button-size * 2);"),
        "ProjectBrowserRow detail hit area should be derived from row/detail tokens instead of a fixed pixel coordinate"
    );
    assert!(
        browser_row[detail_button_index..].contains("width: root.detail-hit-width;")
            && browser_row[detail_button_index..].contains("more-vertical.svg")
            && browser_row[detail_button_index..].contains("detail-state := StateLayerArea {")
            && browser_row[detail_button_index..].contains("Icon {")
            && browser_row[detail_button_index..]
                .contains("clicked => { root.open-detail(root.project.project-path); }"),
        "ProjectBrowserRow trailing detail shell should expose the full tokenized hit zone through a Material StateLayerArea with a centered icon"
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
fn project_choice_rows_use_material_list_tiles() {
    let components = read_ui_file("project_page_components.slint");
    let data_display = read_ui_file("data_display.slint");
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
fn material_and_taffy_coverage_uses_real_hub_surfaces() {
    let components = read_ui_file("components.slint");
    let data_display = read_ui_file("data_display.slint");
    let layout = read_ui_file("layout.slint");
    let inputs = read_ui_file("inputs.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let shared = read_ui_file("shared.slint");
    let material_bridge = read_ui_file("material_bridge.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let editor = read_ui_file("editor.slint");
    let builds = read_ui_file("builds.slint");
    let settings = read_ui_file("settings.slint");

    for (name, source) in [
        ("components.slint", &components),
        ("data_display.slint", &data_display),
    ] {
        for removed_sample in ["ButtonStates", "Button States", "ComponentSamples"] {
            assert!(
                !source.contains(removed_sample),
                "{name} should not reintroduce the removed development sample surface: {removed_sample}"
            );
        }
    }

    for (name, source) in [
        ("project_dashboard.slint", &dashboard),
        ("project_pages.slint", &project_pages),
    ] {
        assert!(
            !source.contains("ComponentSamples"),
            "{name} must not expose the internal ComponentSamples surface in user-facing Hub pages"
        );
    }

    for snippet in [
        "export component Flow",
        "export component PanelGrid",
        "export component WorkspacePanelSection",
        "export component ResponsiveSlot",
    ] {
        assert!(
            layout.contains(snippet),
            "layout.slint must expose the Taffy primitive used by real Hub pages: {snippet}"
        );
    }

    for snippet in [
        "export component SegmentButton",
        "material-segment := SegmentedButton",
        "export component ToolbarSelect",
        "trigger := OutlineButton",
        "menu := PopupMenu",
        "export component HubTextField",
        "material-field := TextField",
        "export component SearchBox",
        "search-field := SearchBar",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep the Hub wrapper backed by the Material primitive: {snippet}"
        );
    }

    for snippet in [
        "if root.variant != \"elevated\": OutlinedCard",
        "if root.show-action: OutlineButton",
        "MaterialText {",
    ] {
        assert!(
            surfaces.contains(snippet),
            "surfaces.slint must keep cards/actions/text on Material primitives: {snippet}"
        );
    }

    for snippet in [
        "FilledButton,",
        "FilledIconButton,",
        "OutlineButton,",
        "OutlineIconButton,",
        "if root.primary &&",
        "if root.active: FilledIconButton",
    ] {
        assert!(
            shared.contains(snippet),
            "shared.slint must keep public Hub button APIs wired to Material buttons: {snippet}"
        );
    }

    for snippet in [
        "OutlinedCard",
        "TextField",
        "FilledButton",
        "OutlineButton",
        "FilledIconButton",
        "OutlineIconButton",
        "Vertical",
    ] {
        assert!(
            material_bridge.contains(snippet) && components.contains(snippet),
            "material_bridge.slint and components.slint must re-export Material primitive {snippet}"
        );
    }

    for snippet in [
        "CatalogPage",
        "PanelListViewport",
        "InfoRow",
        "ActionRow",
        "BuildHistoryRow",
        "ListTile",
        "ScrollView",
    ] {
        assert!(
            data_display.contains(snippet),
            "data_display.slint must keep real list/table surfaces backed by Material wrappers: {snippet}"
        );
    }

    for (page, source, snippets) in [
        (
            "project_dashboard.slint",
            &dashboard,
            &[
                "Flow",
                "PanelGrid",
                "ResponsiveSlot",
                "SearchBox",
                "ToolbarSelect",
                "ActionRow",
            ][..],
        ),
        (
            "project_pages.slint",
            &project_pages,
            &[
                "ResponsiveSlot",
                "SearchBox",
                "ToolbarSelect",
                "HubTextField",
            ][..],
        ),
        (
            "editor.slint",
            &editor,
            &[
                "WorkspacePanelSection",
                "ResponsiveSlot",
                "HubTextField",
                "InfoRow",
                "ActionRow",
            ][..],
        ),
        (
            "builds.slint",
            &builds,
            &[
                "WorkspacePanelSection",
                "ResponsiveSlot",
                "InfoRow",
                "ActionRow",
                "BuildHistoryRow",
            ][..],
        ),
        (
            "settings.slint",
            &settings,
            &[
                "WorkspacePanelSection",
                "ResponsiveSlot",
                "HubTextField",
                "SegmentButton",
            ][..],
        ),
    ] {
        for snippet in snippets {
            assert!(
                source.contains(snippet),
                "{page} must consume the real Material/Taffy wrapper instead of relying on a sample surface: {snippet}"
            );
        }
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
    assert!(
        project_pages.contains(
            "summary-row-height: max(HubTokens.control-sm, min(root.field-height, root.flow-height / 14));"
        ),
        "ProjectNewPage create summary should stay compact enough to keep core create controls visible"
    );
    assert!(
        project_pages.contains(
            "section-label-height: MaterialTypography.body_small.font_size * 3 / 2;"
        ) && project_pages.contains(
            "engine-visible-count: root.engine-count > 0 ? root.engine-count : 1;"
        ) && project_pages.contains(
            "engine-section-height: root.section-label-height + MaterialStyleMetrics.spacing_8 + root.choice-row-height * root.engine-visible-count + MaterialStyleMetrics.spacing_8 * (root.engine-visible-count - 1);"
        ) && project_pages.contains("height: root.engine-section-height;"),
        "ProjectNewPage source-engine selector should size from Material text and row metrics instead of stretching a blank gap"
    );
    for forbidden in [
        "value: root.selected-engine-label;",
        "value: root.selected-template-label;",
    ] {
        assert!(
            !project_pages.contains(forbidden),
            "ProjectNewPage should not duplicate engine/template selections in the compact create summary: {forbidden}"
        );
    }
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
fn hub_ui_files_route_typography_through_material_wrappers() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if uses_raw_text_or_direct_font_binding(trimmed) {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI files must route visible typography through MaterialText/shared wrappers instead of raw Text/font bindings:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_files_do_not_use_character_icon_literals() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if let Some(literal) = character_icon_literal(trimmed) {
                violations.push(format!(
                    "{}:{}: {} uses {literal:?}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI controls must use SVG/Material icon slots instead of single-character text glyphs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_direct_touch_area_is_reserved_for_window_dragging() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || !trimmed.contains("TouchArea") {
                continue;
            }
            if file_name == "shell.slint" && trimmed == "drag-area := TouchArea {" {
                continue;
            }
            violations.push(format!(
                "{}:{}: {}",
                display_path(&path),
                index + 1,
                trimmed
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI interaction surfaces must use Material controls/ListTile/StateLayerArea; direct TouchArea is reserved for shell window dragging:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_files_do_not_use_percentage_sizing() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if percentage_size_binding(trimmed) {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI sizing should use tokens, stretch, or explicit parent/content-width contracts instead of percent-based layout bindings:\n{}",
        violations.join("\n")
    );
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

fn uses_raw_text_or_direct_font_binding(line: &str) -> bool {
    line == "Text {"
        || line.ends_with(": Text {")
        || line.ends_with(":= Text {")
        || line.contains("inherits Text")
        || line.contains("font-size:")
        || line.contains("font-weight:")
        || line.contains("font_size:")
        || line.contains("font_weight:")
}

fn character_icon_literal(line: &str) -> Option<&str> {
    for property in ["text:", "fallback-text:"] {
        if let Some(value) = line.strip_prefix(property) {
            let literal = value.trim().trim_end_matches(';').trim();
            if let Some(unquoted) = literal
                .strip_prefix('"')
                .and_then(|inner| inner.strip_suffix('"'))
            {
                if matches!(
                    unquoted,
                    "+" | ">" | "<" | "[]" | "::" | "==" | "v" | "!" | "?" | "..."
                ) {
                    return Some(unquoted);
                }
            }
        }
    }
    None
}

fn percentage_size_binding(line: &str) -> bool {
    [
        "width:",
        "height:",
        "min-width:",
        "min-height:",
        "max-width:",
        "max-height:",
        "preferred-width:",
        "preferred-height:",
    ]
    .iter()
    .any(|property| line.starts_with(property) && line.contains('%'))
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
