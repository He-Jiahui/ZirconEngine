use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn widget_canvas_and_preview_keep_distinct_state_domains() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchHudWidgetTextRow", "selected"));
    assert!(bridge.control_bool("WorkbenchHudMinimapRow", "selected"));

    assert!(bridge
        .select_dropdown_option(HUD_SCREEN_DROPDOWN, "pause_menu")
        .expect("HUD screen should select"));

    bridge
        .dispatch_control_state("WorkbenchHudWidgetButtonRow", UiEventKind::Click)
        .expect("weapon panel should dispatch")
        .expect("weapon panel should bind");
    bridge
        .dispatch_control_state("WorkbenchHudAmmoPanelRow", UiEventKind::Click)
        .expect("ammo panel should dispatch")
        .expect("ammo panel should bind");
    assert!(bridge.control_bool("WorkbenchHudWidgetButtonRow", "selected"));
    assert!(bridge.control_bool("WorkbenchHudAmmoPanelRow", "selected"));
    assert_eq!(
        bridge.control_string("WorkbenchHudCenterTitle", "text"),
        Some("Pause Menu / WeaponPanel".to_string())
    );
    for (control_id, value) in [
        ("WorkbenchHudDpiField", "1.50"),
        ("WorkbenchHudLocaleField", "zh-CN"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("HUD property should edit");
    }

    bridge
        .dispatch_control_state("WorkbenchHudPreviewButton", UiEventKind::Click)
        .expect("HUD preview should dispatch")
        .expect("HUD preview should bind");
    assert_eq!(
        Some("Preview: AmmoPanel   1.50x / zh-CN   bindings valid".to_string()),
        bridge.control_string("WorkbenchHudValidationRow", "value_text")
    );
}
