use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn search_filters_blend_assets_and_preserves_a_visible_selection() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");
    bridge
        .dispatch_control_state("WorkbenchAbilityAnimationTools", UiEventKind::Click)
        .expect("Blend Space opener should dispatch")
        .expect("Blend Space opener should bind");
    bridge
        .dispatch_workbench_ability_editor_menu_item_state(
            "WorkbenchAbilityAnimationToolsMenu",
            "menu.item.ability.blend_space",
        )
        .expect("Blend Space menu item should dispatch")
        .expect("Blend Space menu item should bind");

    bridge
        .mutate_control_property(
            SEARCH_CONTROL,
            "query",
            UiValue::String("strafe".to_string()),
        )
        .expect("search query should update");
    bridge
        .dispatch_control_state(SEARCH_CONTROL, UiEventKind::Change)
        .expect("search edit should dispatch")
        .expect("search edit should bind");

    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceIdleRunRow")
        .is_none());
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceStrafeRow")
        .is_some());
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceSprintRow")
        .is_none());
    assert!(bridge.control_bool("WorkbenchExtensionBlendSpaceStrafeRow", "selected"));
    assert_eq!(
        Some("BS_Strafe_Grid  |  8 samples".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpaceAssetSummary", "text")
    );
    assert_eq!(
        Some("Strafe_L".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewAsset", "value")
    );
    assert!(bridge.control_frame(EMPTY_SEARCH_CONTROL).is_none());

    bridge
        .mutate_control_property(
            SEARCH_CONTROL,
            "query",
            UiValue::String("missing".to_string()),
        )
        .expect("search query should update");
    bridge
        .dispatch_control_state(SEARCH_CONTROL, UiEventKind::Submit)
        .expect("search commit should dispatch")
        .expect("search commit should bind");

    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceStrafeRow")
        .is_none());
    assert!(bridge.control_frame(EMPTY_SEARCH_CONTROL).is_some());

    bridge
        .mutate_control_property(SEARCH_CONTROL, "query", UiValue::String(String::new()))
        .expect("search query should clear");
    bridge
        .dispatch_control_state(SEARCH_CONTROL, UiEventKind::Change)
        .expect("cleared search should dispatch")
        .expect("cleared search should bind");

    for (control_id, _) in ASSET_ROWS {
        assert!(bridge.control_frame(control_id).is_some());
    }
    assert!(bridge.control_bool("WorkbenchExtensionBlendSpaceStrafeRow", "selected"));
    assert!(bridge.control_frame(EMPTY_SEARCH_CONTROL).is_none());
}

#[test]
fn asset_search_is_ascii_case_insensitive_without_allocating_per_row() {
    assert!(contains_ascii_case_insensitive("BS_Idle_Run", "idle"));
    assert!(contains_ascii_case_insensitive("BS_Sprint_Lean", "SPRINT"));
    assert!(!contains_ascii_case_insensitive("BS_Strafe_Grid", "run"));
}
