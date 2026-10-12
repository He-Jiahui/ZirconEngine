use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn asset_selection_projects_one_profile_and_preserves_transport_state() {
    let mut bridge = open_blend_space();

    bridge
        .dispatch_control_state("WorkbenchExtensionBlendSpaceStrafeRow", UiEventKind::Click)
        .expect("Strafe asset should dispatch")
        .expect("Strafe asset should bind");

    assert!(bridge.control_bool("WorkbenchExtensionBlendSpaceStrafeRow", "selected"));
    assert!(!bridge.control_bool("WorkbenchExtensionBlendSpaceIdleRunRow", "selected"));
    assert_eq!(
        Some("BS_Strafe_Grid  |  8 samples".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpaceAssetSummary", "text")
    );
    assert_eq!(
        Some("Strafe_L".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewAsset", "value")
    );
    assert_eq!(
        Some("BS_Strafe_Grid".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpaceAssetDropdown", "value")
    );
    assert_eq!(
        Some("-90, 240".to_string()),
        bridge.control_string(
            "WorkbenchExtensionBlendSpaceSamplePositionProperty",
            "value"
        )
    );
    assert_eq!(
        Some("0.34".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpaceSampleRateProperty", "value")
    );
    assert_eq!(
        Some("Strafe_L".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewTimeline", "track_label")
    );
    assert_eq!(
        Some("-90.0".to_string()),
        bridge.control_string("WorkbenchSampleWeightsDirectionValue", "text")
    );
    assert_eq!(
        Some("240.0".to_string()),
        bridge.control_string("WorkbenchSampleWeightsSpeedValue", "text")
    );
    assert!(bridge.control_bool(
        "WorkbenchExtensionBlendSpaceSampleStrafeLeftRow",
        "selected"
    ));
    assert_eq!(
        Some(1.0),
        bridge.control_float("WorkbenchSampleWeightsRunLeft", "value_percent")
    );

    bridge
        .dispatch_control_state(
            "WorkbenchExtensionBlendSpaceSampleStrafeRightRow",
            UiEventKind::Click,
        )
        .expect("Strafe-right sample should dispatch")
        .expect("Strafe-right sample should bind");
    assert!(bridge.control_bool(
        "WorkbenchExtensionBlendSpaceSampleStrafeRightRow",
        "selected"
    ));
    assert_eq!(
        Some("Strafe_R".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewAsset", "value")
    );
    assert_eq!(
        Some("90, 240".to_string()),
        bridge.control_string(
            "WorkbenchExtensionBlendSpaceSamplePositionProperty",
            "value"
        )
    );
    assert_eq!(
        Some(1.0),
        bridge.control_float("WorkbenchSampleWeightsRunRight", "value_percent")
    );
    assert_eq!(
        Some("Selected Strafe_R   Speed 240.0   Direction 90.0".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpaceOutputRow", "value_text")
    );

    bridge
        .dispatch_control_state(
            "WorkbenchExtensionBlendSpacePreviewButton",
            UiEventKind::Click,
        )
        .expect("Preview command should dispatch")
        .expect("Preview command should bind");
    assert_eq!(
        Some("Preview queued   BS_Strafe_Grid   Strafe_R".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpaceOutputRow", "value_text")
    );

    bridge
        .mutate_control_property(
            "WorkbenchExtensionBlendSpaceAssetDropdown",
            "value",
            UiValue::String("BS_Idle_Run".to_string()),
        )
        .expect("Asset dropdown value should update");
    bridge
        .dispatch_control_state(
            "WorkbenchExtensionBlendSpaceAssetDropdown",
            UiEventKind::Change,
        )
        .expect("Asset dropdown should dispatch")
        .expect("Asset dropdown should bind");
    assert!(bridge.control_bool("WorkbenchExtensionBlendSpaceIdleRunRow", "selected"));
    assert_eq!(
        Some("Idle".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewAsset", "value")
    );
    assert_eq!(
        Some("Idle  |  Previewing".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewStatus", "text")
    );
    assert_eq!(
        Some(1.0),
        bridge.control_float("WorkbenchSampleWeightsIdle", "value_percent")
    );

    bridge
        .dispatch_control_state("WorkbenchTransportPause", UiEventKind::Click)
        .expect("Pause should dispatch")
        .expect("Pause should bind");
    bridge
        .dispatch_control_state("WorkbenchExtensionBlendSpaceSprintRow", UiEventKind::Click)
        .expect("Sprint asset should dispatch")
        .expect("Sprint asset should bind");

    assert_eq!(
        Some("Run_Fwd  |  Paused".to_string()),
        bridge.control_string("WorkbenchExtensionBlendSpacePreviewStatus", "text")
    );
    assert_eq!(
        Some(1.0),
        bridge.control_float("WorkbenchSampleWeightsRunForward", "value_percent")
    );
    assert_eq!(
        Some(0.0),
        bridge.control_float("WorkbenchSampleWeightsRunLeft", "value_percent")
    );
}

fn open_blend_space() -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
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
}
