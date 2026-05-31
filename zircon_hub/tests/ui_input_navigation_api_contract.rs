//! Static API contracts for the Hub input and navigation primitive surface.

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
fn components_exports_locked_input_and_navigation_api() {
    let components = read_ui_file("components.slint");

    for snippet in [
        "SelectOptionData,",
        "HubCheckBox,",
        "HubCheckBoxRow,",
        "HubComboBox,",
        "HubSwitch,",
        "HubTextField,",
        "HubToggleRow,",
        "SearchBox,",
        "ToolbarSelect,",
        "DropDownButton,",
        "SegmentButton,",
        "} from \"inputs.slint\";",
        "export { HubTabs, NavRail } from \"navigation.slint\";",
        "export { NavItemData } from \"shared.slint\";",
        "NavigationItem,",
        "MenuItem,",
        "MaterialStyleMetrics,",
        "MaterialPalette,",
        "MaterialTypography,",
        "} from \"material_bridge.slint\";",
    ] {
        assert!(
            components.contains(snippet),
            "components.slint must keep the stable input/navigation primitive export surface; missing {snippet}"
        );
    }
}

#[test]
fn input_primitives_keep_public_state_and_callback_contracts() {
    let inputs = read_ui_file("inputs.slint");

    let cases = [
        (
            "SearchBox",
            "HubTextField",
            &[
                "in-out property <string> text;",
                "in property <string> placeholder;",
                "in property <bool> enabled: true;",
                "out property <bool> focused: search-field.has-focus;",
                "in property <length> box-width: HubTokens.input-width;",
                "in property <length> box-height: HubVisualSpec.toolbar-density-height;",
                "callback edited(string);",
                "callback accepted(string);",
                "forward-focus: search-field;",
            ][..],
        ),
        (
            "HubTextField",
            "ToolbarSelect",
            &[
                "in-out property <string> text;",
                "in property <string> label;",
                "in property <string> placeholder;",
                "in property <string> supporting-text;",
                "in property <bool> enabled: true;",
                "out property <bool> focused: material-field.has-focus;",
                "in property <image> leading-icon;",
                "in property <image> trailing-icon;",
                "callback edited(string);",
                "callback accepted(string);",
                "forward-focus: material-field;",
            ][..],
        ),
        (
            "ToolbarSelect",
            "DropDownButton",
            &[
                "in property <string> icon;",
                "in property <image> icon-image;",
                "in property <bool> has-icon-image: false;",
                "in property <string> text;",
                "in property <length> select-width: HubVisualSpec.toolbar-density-height * 7 / 2;",
                "in property <length> select-height: HubVisualSpec.toolbar-density-height;",
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "in property <[MenuItem]> menu-items: [];",
                "in property <[SelectOptionData]> options: [];",
                "in property <int> option-count: 0;",
                "callback selected(string);",
                "callback selected-index(int);",
            ][..],
        ),
        (
            "DropDownButton",
            "SegmentButton",
            &[
                "in property <string> text;",
                "in property <image> icon-image;",
                "in property <bool> has-icon-image: false;",
                "in property <bool> active: false;",
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "in property <length> button-width: HubTokens.input-width * 2 / 5;",
                "in property <length> button-height: HubTokens.control-md;",
                "callback clicked;",
            ][..],
        ),
        (
            "SegmentButton",
            "",
            &[
                "in property <string> text;",
                "in property <bool> active;",
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "in property <length> button-height: HubTokens.control-md;",
                "callback clicked;",
            ][..],
        ),
    ];

    for (component, next_component, required) in cases {
        let component_source = inputs
            .split(&format!("export component {component}"))
            .nth(1)
            .and_then(|source| {
                if next_component.is_empty() {
                    Some(source)
                } else {
                    source
                        .split(&format!("export component {next_component}"))
                        .next()
                }
            })
            .unwrap_or_else(|| panic!("inputs.slint must declare {component}"));

        for snippet in required {
            assert!(
                component_source.contains(snippet),
                "{component} must preserve its public primitive API; missing {snippet}"
            );
        }
    }
}

#[test]
fn hub_combobox_keeps_public_menu_adapter_contract() {
    let inputs = read_ui_file("inputs.slint");
    let component_source = inputs
        .split("export component HubComboBox")
        .nth(1)
        .unwrap_or_else(|| panic!("inputs.slint must declare HubComboBox"));

    for snippet in [
        "in property <string> label;",
        "in property <bool> enabled: true;",
        "in property <image> leading-icon;",
        "in property <[MenuItem]> items: [];",
        "in-out property <int> current-index: -1;",
        "in property <length> combo-width: HubTokens.input-width;",
        "in property <length> combo-height: HubTokens.input-field;",
        "callback selected(int);",
        "HubDropDownSurface {",
        "dropdown-width: parent.width;",
        "dropdown-height: parent.height;",
        "dropdown-items: root.items;",
        "current_index <=> root.current-index;",
        "selected(index) =>",
    ] {
        assert!(
            component_source.contains(snippet),
            "HubComboBox must preserve its public Material menu adapter API; missing {snippet}"
        );
    }
}

#[test]
fn navigation_adapter_uses_one_nav_model_for_expanded_and_collapsed_routes() {
    let shared = read_ui_file("shared.slint");
    let navigation = read_ui_file("navigation.slint");
    let app = read_ui_file("app.slint");
    let sidebar = read_ui_file("shell_sidebar_components.slint");
    let binding = read_crate_file("src/app/binding.rs");
    let view_model = read_crate_file("src/app/view_model.rs");

    for snippet in [
        "export struct NavItemData {",
        "id: string,",
        "title: string,",
        "icon: string,",
        "icon-image: image,",
        "has-icon-image: bool,",
        "active: bool,",
    ] {
        assert!(
            shared.contains(snippet),
            "NavItemData must remain the shared Hub navigation DTO; missing {snippet}"
        );
    }

    for snippet in [
        "in property <[NavItemData]> items;",
        "in property <[NavigationItem]> material-items;",
        "in-out property <int> current-index: 0;",
        "private property <[NavigationItem]> enabled-material-items: root.enabled ? root.material-items : [];",
        "items: root.enabled-material-items;",
        "current_index <=> root.current-index;",
        "if root.enabled && index >= 0 && index < root.items.length",
        "root.clicked(root.items[index].id);",
        "for item in root.items: NavButton",
    ] {
        assert!(
            navigation.contains(snippet),
            "NavRail must keep collapsed Material routing and expanded Hub routing on one item model; missing {snippet}"
        );
    }

    for snippet in [
        "in property <[NavigationItem]> material-nav-items;",
        "in-out property <int> selected-nav-index: 0;",
        "material-nav-items: root.material-nav-items;",
        "selected-nav-index <=> root.selected-nav-index;",
    ] {
        assert!(
            app.contains(snippet) || sidebar.contains(snippet),
            "HubWindow/HubNavSidebar must forward Material navigation adapter data; missing {snippet}"
        );
    }

    for snippet in [
        "let nav_items = view_model::navigation_items(snapshot.selected_page, language);",
        "ui.set_selected_nav_index(view_model::selected_nav_index(&nav_items));",
        "view_model::material_navigation_items(&nav_items)",
        "ui.set_nav_items(view_model::model_from(nav_items));",
    ] {
        assert!(
            binding.contains(snippet),
            "binding.rs must derive both navigation projections from one nav_items vector; missing {snippet}"
        );
    }

    for snippet in [
        "pub(super) fn material_navigation_items(items: &[NavItemData]) -> Vec<NavigationItem>",
        "icon: item.icon_image.clone(),",
        "selected_icon: item.icon_image.clone(),",
        "text: item.title.clone(),",
        "show_badge: false,",
        "pub(super) fn selected_nav_index(items: &[NavItemData]) -> i32",
    ] {
        assert!(
            view_model.contains(snippet),
            "view_model.rs must keep Material NavigationItem as an adapter over NavItemData; missing {snippet}"
        );
    }
}
