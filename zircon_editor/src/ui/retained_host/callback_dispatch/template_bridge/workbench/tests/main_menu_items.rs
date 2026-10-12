use super::super::super::popup_primitives::template_popup_menu_item_state;
use super::*;

const FILE_MENU_CONTROL_ID: &str = "UnrealFileMenuPopup";
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

#[test]
fn file_menu_click_opens_measured_popup_and_background_has_no_action() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1200.0, 800.0))
        .expect("workbench bridge should build");
    assert!(!bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge
        .dispatch_control_state(FILE_MENU_CONTROL_ID, UiEventKind::Click)
        .expect("background dispatch should be harmless")
        .is_none());
    assert!(!bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    bridge.mutate_control_property(FILE_MENU_CONTROL_ID, "menu_items",
        UiValue::Array(vec![UiValue::String(
            "Open a scene with a deliberately long project owned display label|action=menu.item.open_scene,icon=folder|Ctrl+O".to_string()
        )])).unwrap();
    assert!(bridge
        .dispatch_control_state("MenuSlot0", UiEventKind::Click)
        .expect("actual toolbar click should open its menu")
        .is_some());
    assert!(bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge.control_frame(FILE_MENU_CONTROL_ID).unwrap().width > 240.0);
    assert!(bridge
        .main_menu_item_binding(FILE_MENU_CONTROL_ID, "")
        .is_none());
    assert!(bridge
        .main_menu_item_binding(FILE_MENU_CONTROL_ID, "unknown")
        .is_none());
}

#[test]
fn window_menus_switch_exclusively_and_cancel_clears_the_trigger() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    bridge
        .dispatch_control_state("MenuSlot0", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    bridge
        .dispatch_control_state("MenuSlot1", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(!bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
    assert!(bridge.control_bool("MenuSlot1", "selected"));
    assert!(bridge.close_popup("UnrealEditMenuPopup").unwrap());
    assert!(!bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
    assert!(!bridge.control_bool("MenuSlot1", "selected"));
    bridge
        .dispatch_control_state("MenuSlot1", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
    bridge
        .dispatch_control_state("MenuSlot1", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(!bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
}

#[test]
fn every_enabled_authored_window_menu_item_has_an_execution_binding() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    for menu in [
        "UnrealFileMenuPopup",
        "UnrealEditMenuPopup",
        "UnrealWindowMenuPopup",
        "UnrealModesMenuPopup",
        "UnrealAddMenuPopup",
        "WorkbenchRunModeMenu",
    ] {
        assert!(bridge.has_control(menu), "missing authored menu {menu}");
        let items = bridge.control_string_array(menu, "menu_items");
        assert!(
            !items.is_empty(),
            "menu {menu} should expose its authored items"
        );
        for raw in items {
            let item = template_popup_menu_item_state(&raw).expect("structured menu item");
            if item.disabled || item.separator {
                continue;
            }
            let binding = bridge
                .main_menu_item_binding(menu, &item.action_id)
                .or_else(|| bridge.run_mode_menu_item_binding(menu, &item.action_id));
            assert!(
                binding.is_some(),
                "enabled menu item {menu}/{} has no execution binding",
                item.action_id
            );
        }
    }
}

#[test]
fn content_drawer_toggle_and_close_keep_the_trigger_state_in_sync() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    assert!(!bridge.control_bool("UnrealContentDrawerToggle", "selected"));
    bridge
        .dispatch_control_state("UnrealContentDrawerToggle", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool("UnrealContentDrawerToggle", "selected"));
    assert!(bridge.control_frame("ContentDrawerClose").unwrap().height > 0.0);
    bridge
        .dispatch_control_state("ContentDrawerClose", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(!bridge.control_bool("UnrealContentDrawerToggle", "selected"));
    bridge
        .dispatch_control_state("UnrealContentDrawerToggle", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool("UnrealContentDrawerToggle", "selected"));
}

#[test]
fn closed_authored_menus_are_not_visible_or_pointer_targets_at_startup() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    for menu in [
        "UnrealFileMenuPopup",
        "UnrealEditMenuPopup",
        "UnrealWindowMenuPopup",
        "UnrealToolsMenuPopup",
        "UnrealBuildMenuPopup",
        "UnrealHelpMenuPopup",
        "UnrealModesMenuPopup",
        "UnrealAddMenuPopup",
        "UnrealBlueprintsMenuPopup",
        "UnrealCinematicsMenuPopup",
        "UnrealSettingsMenuPopup",
        "WorkbenchRunModeMenu",
    ] {
        assert!(
            !bridge.control_bool(menu, "popup_open"),
            "menu {menu} starts open"
        );
        let id = bridge.control_node_id(menu).expect("authored menu node");
        let node = bridge.surface().tree.node(id).expect("menu tree node");
        assert!(
            !node.is_render_visible(),
            "closed menu {menu} participates in drawing"
        );
        assert!(
            !node.is_self_hit_test_visible(),
            "closed menu {menu} intercepts pointer input"
        );
    }
}
