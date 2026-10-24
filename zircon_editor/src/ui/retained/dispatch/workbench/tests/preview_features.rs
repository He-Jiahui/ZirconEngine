use super::*;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn default_preview_gate_hides_mock_navigation_and_rejects_direct_mounts() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    assert!(!bridge.preview_features_enabled);
    assert!(bridge.control_frame("WorkbenchModuleMore").is_none());
    bridge
        .apply_workbench_module_workspace("workbench.module.effect.select")
        .unwrap();
    bridge
        .apply_workbench_extension_workspace("workbench.extension.shader_editor.open")
        .unwrap();
    assert!(
        !bridge.has_control("WorkbenchModuleEffectWorkspace")
            && !bridge.has_control("WorkbenchExtensionShaderWorkspace")
    );
    assert!(bridge
        .dispatch_workbench_module_overflow_menu_item_state(
            "UnrealWindowMenuPopup",
            "menu.item.effect",
        )
        .unwrap()
        .is_none());
    assert_eq!(
        bridge
            .control_string("WorkbenchStatusReady", "text")
            .as_deref(),
        Some("Not implemented (preview)")
    );
}

#[test]
fn opt_in_is_badged_and_disabling_returns_to_the_scene() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    assert!(bridge.set_preview_features(true).unwrap());
    assert!(!bridge.set_preview_features(true).unwrap());
    bridge
        .apply_workbench_module_workspace("workbench.module.effect.select")
        .unwrap();
    assert!(bridge.has_control("WorkbenchModuleEffectWorkspace"));
    let items = bridge.control_string_array("UnrealWindowMenuPopup", "menu_items");
    for action in ["menu.item.effect", "menu.item.ability"] {
        let item = items
            .iter()
            .find(|item| item.contains(&format!("action={action}")))
            .expect("the real Window menu should expose its opted-in workspace");
        assert!(
            item.contains("(Preview)"),
            "the opted-in workspace must remain badged: {item}"
        );
    }
    bridge.set_preview_features(false).unwrap();
    assert!(
        !bridge.has_control("WorkbenchModuleEffectWorkspace")
            && !bridge.has_control("WorkbenchExtensionShaderWorkspace")
    );
    assert!(bridge.control_frame("WorkbenchModuleMore").is_none());
    assert!(!bridge
        .control_string_array("UnrealWindowMenuPopup", "menu_items")
        .iter()
        .any(|item| item.contains("menu.item.effect")));
    assert!(bridge.control_frame("UnrealViewportToolbar").is_some());
}
