//! Static contracts for Zircon Hub navigation primitives.

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
fn expanded_nav_button_uses_reference_state_layer_row() {
    let shared = read_ui_file("shared.slint");
    let navigation = read_ui_file("navigation.slint");
    let nav_button_label = navigation
        .split("component NavButtonLabel")
        .nth(1)
        .and_then(|source| source.split("export component NavButton").next())
        .expect("navigation.slint must declare NavButtonLabel before NavButton");
    for snippet in [
        "inherits MaterialText",
        "in property <string> value;",
        "in property <bool> active: false;",
        "text: root.value;",
        "color: root.active ? MaterialPalette.on_surface : HubVisualSpec.nav-idle-foreground;",
        "style: MaterialTypography.label_large;",
        "overflow: elide;",
        "vertical_alignment: center;",
    ] {
        assert!(
            nav_button_label.contains(snippet),
            "NavButtonLabel must own expanded navigation row label typography; missing {snippet}"
        );
    }

    let nav_button = navigation
        .split("export component NavButton")
        .nth(1)
        .and_then(|source| source.split("export component NavRail").next())
        .expect("navigation.slint must declare NavButton before NavRail");

    assert!(
        !shared.contains("export component NavButton"),
        "shared.slint should keep NavItemData but not own the expanded navigation row implementation"
    );

    for snippet in [
        "StateLayerArea,",
        "StateLayerArea {",
        "border-radius: HubVisualSpec.compact-radius;",
        "in property <bool> enabled: true;",
        "in property <bool> focused: false;",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : (root.item.active ? HubTokens.border-width : 0px);",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : HubVisualSpec.nav-active-stroke;",
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "background: root.item.active ? HubVisualSpec.nav-active-fill : transparent;",
        "colorize: root.item.active ? HubVisualSpec.neutral-icon-foreground : HubVisualSpec.nav-idle-foreground;",
        "source: root.item.has-icon-image ? root.item.icon-image : @image-url(\"../assets/icons/nav/projects.svg\");",
        "NavButtonLabel {",
        "value: root.item.title;",
        "active: root.item.active;",
        "clicked =>",
        "if (root.enabled) {",
        "root.clicked(root.item.id);",
    ] {
        assert!(
            navigation.contains(snippet) || nav_button.contains(snippet),
            "NavButton must preserve the Hub navigation API while matching the reference square-rounded state-layer row; missing {snippet}"
        );
    }

    for forbidden in [
        "area := TouchArea",
        "CenteredIcon",
        "ListTile {",
        "padding-left: MaterialStyleMetrics.padding_16;",
        "MaterialText {",
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
fn collapsed_nav_rail_uses_material_navigation_rail() {
    let navigation = read_ui_file("navigation.slint");
    for snippet in [
        "NavigationRail as MaterialNavigationRail",
        "in property <[NavigationItem]> material-items;",
        "in-out property <int> current-index: 0;",
        "in property <bool> enabled: true;",
        "private property <[NavigationItem]> enabled-material-items: root.enabled ? root.material-items : [];",
        "min-width: root.collapsed ? MaterialStyleMetrics.size_80 : 0px;",
        "if root.collapsed: MaterialNavigationRail",
        "items: root.enabled-material-items;",
        "current_index <=> root.current-index;",
        "alignment: start;",
        "has_menu: false;",
        "index_changed(index) =>",
        "if root.enabled && index >= 0 && index < root.items.length",
        "root.clicked(root.items[index].id);",
        "if !root.collapsed: VerticalLayout",
        "width: parent.width - root.rail-padding * 2;",
        "collapsed: false;",
        "enabled: root.enabled;",
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
        "NavigationItem,",
        "ResponsiveState } from \"components.slint\";",
        "in property <[NavigationItem]> material-nav-items;",
        "in-out property <int> selected-nav-index: 0;",
        "private property <bool> nav-auto-collapsed: responsive-state.compact;",
        "private property <bool> nav-effective-collapsed: root.nav-collapsed || root.nav-auto-collapsed;",
        "private property <length> nav-pad: root.nav-effective-collapsed ? max(HubTokens.space-2, min(HubTokens.space-3, root.nav-width / 7)) : HubTokens.space-4;",
        "material-nav-items: root.material-nav-items;",
        "selected-nav-index <=> root.selected-nav-index;",
        "collapsed: root.nav-effective-collapsed;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must expose and forward Material navigation adapter data; missing {snippet}"
        );
    }

    let shell_sidebar_components = read_ui_file("shell_sidebar_components.slint");
    let sidebar = shell_sidebar_components
        .split("export component HubNavSidebar")
        .nth(1)
        .expect("shell_sidebar_components.slint must export HubNavSidebar");
    for snippet in [
        "in property <[NavigationItem]> material-nav-items;",
        "in-out property <int> selected-nav-index: 0;",
        "material-items: root.material-nav-items;",
        "current-index <=> root.selected-nav-index;",
    ] {
        assert!(
            sidebar.contains(snippet),
            "HubNavSidebar must forward Material navigation data into NavRail from shell_sidebar_components.slint; missing {snippet}"
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
fn hub_tabs_wrap_material_tab_bars() {
    let components = read_ui_file("components.slint");
    assert!(
        components.contains("HubTabs"),
        "components.slint must re-export the Hub Material-backed tabs wrapper"
    );

    let navigation = read_ui_file("navigation.slint");
    for snippet in [
        "SecondaryTabBar,",
        "TabBar,",
        "export component HubTabs",
        "in property <[NavigationItem]> items;",
        "in-out property <int> current-index: 0;",
        "in property <bool> secondary: false;",
        "if !root.secondary: TabBar",
        "if root.secondary: SecondaryTabBar",
        "items: root.items;",
        "current_index <=> root.current-index;",
        "index_changed(index) =>",
        "root.selected(index);",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : transparent;",
    ] {
        assert!(
            navigation.contains(snippet),
            "HubTabs must delegate primary/secondary tab layout to the local Material TabBar API; missing {snippet}"
        );
    }

    let tabs = navigation
        .split("export component HubTabs")
        .nth(1)
        .expect("navigation.slint must declare HubTabs");
    for forbidden in ["TouchArea", "area.has-hover", "SegmentButton"] {
        assert!(
            !tabs.contains(forbidden),
            "HubTabs should not emulate tabs with hand-rolled rows or segment buttons: {forbidden}"
        );
    }
}

#[test]
fn compact_tab_strip_delegates_to_material_hub_tabs() {
    let compact_tabs = read_ui_file("compact_page_components.slint");
    let compact_strip = compact_tabs
        .split("export component HubCompactTabStrip")
        .nth(1)
        .and_then(|source| source.split("export component HubWorkspaceTabStrip").next())
        .expect("compact_page_components.slint must declare HubCompactTabStrip before HubWorkspaceTabStrip");

    for snippet in [
        "import { NavigationItem } from \"material_bridge.slint\";",
        "import { HubTabs } from \"navigation.slint\";",
        "private property <[NavigationItem]> first-only-tabs:",
        "private property <[NavigationItem]> visible-tabs:",
        "private property <int> visible-current-index:",
        "HubTabs {",
        "width: root.tab-width * root.visible-tabs.length;",
        "current-index: root.visible-current-index;",
        "items: root.visible-tabs;",
        "secondary: true;",
        "tabs-height: HubTokens.control-md;",
        "root.show-second && root.show-third && root.show-fourth ? root.all-tabs",
        "root.show-fourth ? root.first-fourth-tabs : root.first-only-tabs",
        "if index == 0",
        "root.current-index = 0;",
        "if root.show-second",
        "root.current-index = 1;",
        "else if root.show-third",
        "root.current-index = 2;",
        "else if root.show-fourth",
        "root.current-index = 3;",
    ] {
        assert!(
            compact_tabs.contains(snippet) || compact_strip.contains(snippet),
            "HubCompactTabStrip must preserve its Hub-facing tab API while delegating visible tabs to HubTabs; missing {snippet}"
        );
    }

    for forbidden in [
        "component CompactTabButton",
        "CompactTabButton {",
        "StateLayerArea",
        "TouchArea",
        "HorizontalLayout",
        "MaterialText",
        "MaterialTypography",
        "border-color: root.active",
        "view-toggle-active",
    ] {
        assert!(
            !compact_tabs.contains(forbidden),
            "HubCompactTabStrip should not reintroduce a local hand-painted tab implementation after adopting HubTabs: {forbidden}"
        );
    }
}

#[test]
fn workspace_pages_consume_workspace_tab_strip_wrapper() {
    let components = read_ui_file("components.slint");
    assert!(
        components.contains("HubCompactTabStrip, HubWorkspaceTabStrip"),
        "components.slint must re-export the compact tab primitive and the workspace-page tabs wrapper"
    );

    let compact_tabs = read_ui_file("compact_page_components.slint");
    for snippet in [
        "export component HubWorkspaceTabStrip inherits HubCompactTabStrip",
        "first-label: \"Overview\";",
        "second-label: \"Details\";",
        "third-label: \"History\";",
        "fourth-label: \"Timeline\";",
        "tab-width: HubTokens.control-md * 4;",
    ] {
        assert!(
            compact_tabs.contains(snippet),
            "HubWorkspaceTabStrip must own the common workspace tab defaults; missing {snippet}"
        );
    }

    let workspace_tabs = compact_tabs
        .split("export component HubWorkspaceTabStrip")
        .nth(1)
        .expect("compact_page_components.slint must declare HubWorkspaceTabStrip");
    for forbidden in [
        "CompactTabButton",
        "StateLayerArea",
        "TouchArea",
        "HorizontalLayout",
    ] {
        assert!(
            !workspace_tabs.contains(forbidden),
            "HubWorkspaceTabStrip should stay a semantic wrapper over HubCompactTabStrip, not a second hand-built tab strip: {forbidden}"
        );
    }

    for (page, labels) in [
        (
            "builds.slint",
            [
                "first-label: \"Overview\";",
                "second-label: \"Pipeline\";",
                "third-label: \"History\";",
                "fourth-label: \"Timeline\";",
            ],
        ),
        (
            "cloud.slint",
            [
                "first-label: \"Overview\";",
                "second-label: root.ui-text.cloud-packages;",
                "third-label: root.ui-text.cloud-services;",
                "fourth-label: root.ui-text.operation-timeline;",
            ],
        ),
        (
            "editor.slint",
            [
                "first-label: \"Overview\";",
                "second-label: \"Source Paths\";",
                "third-label: \"Timeline\";",
                "fourth-label: \"Actions\";",
            ],
        ),
        (
            "settings.slint",
            [
                "first-label: \"General\";",
                "second-label: root.ui-text.build-defaults;",
                "third-label: root.ui-text.default-paths;",
                "fourth-label: root.ui-text.configuration-health;",
            ],
        ),
    ] {
        let source = read_ui_file(page);
        assert!(
            source.contains("HubWorkspaceTabStrip")
                && source.contains("HubWorkspaceTabStrip {")
                && !source.contains("HubCompactTabStrip"),
            "{page} must consume the workspace-level tab-strip wrapper instead of the lower-level compact tab primitive"
        );
        assert!(
            !source.contains("tab-width: HubTokens.control-md * 4;"),
            "{page} should not repeat the shared workspace tab width after the wrapper extraction"
        );
        for label in labels {
            assert!(
                source.contains(label),
                "{page} should keep its semantic tab label binding after moving to HubWorkspaceTabStrip: {label}"
            );
        }
    }
}
