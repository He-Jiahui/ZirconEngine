use super::*;
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

#[test]
fn toolbar_click_opens_measured_main_menu_and_background_has_no_action() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1200.0, 800.0))
        .expect("workbench bridge should build");
    assert!(!bridge.control_bool(MAIN_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge
        .dispatch_control_state(MAIN_MENU_CONTROL_ID, UiEventKind::Click)
        .expect("background dispatch should be harmless")
        .is_none());
    assert!(!bridge.control_bool(MAIN_MENU_CONTROL_ID, "popup_open"));
    bridge.mutate_control_property(MAIN_MENU_CONTROL_ID, "menu_items",
        UiValue::Array(vec![UiValue::String(
            "Open a scene with a deliberately long project owned display label|action=menu.item.open_scene,icon=folder|Ctrl+O".to_string()
        )])).unwrap();
    assert!(bridge
        .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
        .expect("actual toolbar click should open its menu")
        .is_some());
    assert!(bridge.control_bool(MAIN_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge.control_frame(MAIN_MENU_CONTROL_ID).unwrap().width > 240.0);
    assert!(bridge
        .main_menu_item_binding(MAIN_MENU_CONTROL_ID, "")
        .is_none());
    assert!(bridge
        .main_menu_item_binding(MAIN_MENU_CONTROL_ID, "unknown")
        .is_none());
}
