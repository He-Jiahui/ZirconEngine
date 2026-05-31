//! Static contracts for Hub Material popup, drawer, and window wrappers.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
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

#[test]
fn hub_overlay_wrappers_delegate_to_material_popup_drawer_and_window() {
    let components = read_ui_file("components.slint");
    for snippet in [
        "HubDrawer,",
        "HubDropDownSurface,",
        "HubModalDrawer,",
        "HubPopupMenu,",
        "HubPopupWindow,",
        "HubWindowView,",
        "PopupPanel",
        "from \"overlays.slint\";",
    ] {
        assert!(
            components.contains(snippet),
            "components.slint must re-export overlay primitive {snippet}"
        );
    }

    let overlays = read_ui_file("overlays.slint");
    for snippet in [
        "Drawer as MaterialDrawer,",
        "DropDownMenu,",
        "ModalDrawer as MaterialModalDrawer,",
        "MaterialWindow,",
        "PopupMenu,",
        "export component HubPopupWindow inherits PopupWindow",
        "close-policy: close-on-click-outside;",
        "popup-width: HubTokens.panel-min-sm;",
        "@children",
        "export component HubPopupMenu inherits PopupMenu",
        "items: root.menu-items;",
        "export component HubDropDownSurface inherits DropDownMenu",
        "items: root.dropdown-items;",
        "export component HubDrawer inherits MaterialDrawer",
        "min-width: root.drawer-width;",
        "export component HubModalDrawer inherits MaterialModalDrawer",
        "width: root.drawer-width;",
        "export component HubWindowView inherits MaterialWindow",
        "no-frame: true;",
        "resize-border-width: HubTokens.window-resize-border;",
        "preferred-width: HubTokens.window-preferred-width;",
        "background: HubVisualSpec.page-background;",
    ] {
        assert!(
            overlays.contains(snippet),
            "overlays.slint must keep popup/drawer/window wrappers backed by the local Material template; missing {snippet}"
        );
    }

    for wrapper_name in [
        "HubPopupMenu",
        "HubPopupWindow",
        "HubDropDownSurface",
        "HubDrawer",
        "HubModalDrawer",
        "HubWindowView",
    ] {
        let wrapper = overlays
            .split(&format!("export component {wrapper_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("overlays.slint must declare {wrapper_name}"));
        for forbidden in ["TouchArea", "area.has-hover", "\n    Window {"] {
            assert!(
                !wrapper.contains(forbidden),
                "{wrapper_name} must not clone Material overlay/window behavior locally: {forbidden}"
            );
        }
    }
}

#[test]
fn app_window_inherits_shared_window_view() {
    let app = read_ui_file("app.slint");
    assert!(
        app.contains("export component HubWindow inherits HubWindowView"),
        "HubWindow must compose the shared window-view wrapper before adding shell/page content"
    );
    for forbidden in [
        "export component HubWindow inherits MaterialWindow",
        "resize-border-width: HubTokens.window-resize-border;",
        "preferred-width: HubTokens.window-preferred-width;",
        "preferred-height: HubTokens.window-preferred-height;",
    ] {
        assert!(
            !app.contains(forbidden),
            "app.slint should not own window-view constraints after HubWindowView extraction: {forbidden}"
        );
    }
}
