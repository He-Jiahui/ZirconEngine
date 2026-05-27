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
        "NavigationItem,",
        "ResponsiveState } from \"components.slint\";",
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
