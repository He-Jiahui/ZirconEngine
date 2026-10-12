use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn effect_asset_selection_search_and_apply_share_one_projection() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchEffectHealthRegenRow", "selected"));
    bridge
        .dispatch_control_state("WorkbenchEffectDamageFireRow", UiEventKind::Click)
        .expect("damage effect should dispatch")
        .expect("damage effect should bind");
    assert_eq!(
        Some("GE_DamageFire".to_string()),
        bridge.control_string("WorkbenchEffectCenterTitle", "text")
    );
    assert_eq!(
        Some("Effect.Damage.Fire".to_string()),
        bridge.control_string("WorkbenchEffectTagField", "value")
    );
    assert_eq!(
        Some("25.0".to_string()),
        bridge.control_string("WorkbenchEffectMagnitudeField", "value")
    );

    bridge
        .dispatch_control_state("WorkbenchEffectModifierHealingRow", UiEventKind::Click)
        .expect("healing modifier should dispatch")
        .expect("healing modifier should bind");
    assert!(bridge.control_bool("WorkbenchEffectDamageFireRow", "selected"));
    assert!(bridge.control_bool("WorkbenchEffectModifierHealingRow", "selected"));
    assert!(bridge.control_bool("WorkbenchEffectGraphRow", "selected"));
    assert!(bridge.control_bool("WorkbenchEffectAttributePreviewRow", "selected"));

    for (control_id, value) in [
        ("WorkbenchEffectNameField", "GE_CustomBurn"),
        ("WorkbenchEffectTagField", "Effect.Damage.Custom"),
        ("WorkbenchEffectMagnitudeField", "42.5"),
        ("WorkbenchEffectStackField", "Aggregate by Target"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("effect property should edit");
    }
    assert!(bridge
        .select_dropdown_option("WorkbenchEffectPolicyDropdown", "instant")
        .expect("effect policy should select"));

    bridge
        .dispatch_control_state("WorkbenchEffectApplyButton", UiEventKind::Click)
        .expect("effect apply should dispatch")
        .expect("effect apply should bind");
    assert_eq!(
        Some("Applied GE_CustomBurn   Instant   Aggregate by Target".to_string()),
        bridge.control_string("WorkbenchEffectOutputRow", "text")
    );
    assert_eq!(
        Some("GE_CustomBurn".to_string()),
        bridge.control_string("WorkbenchEffectCenterTitle", "text")
    );
    assert_eq!(
        Some("Preview Level 1       Effect.Damage.Custom 42.5".to_string()),
        bridge.control_string("WorkbenchEffectAttributePreviewRow", "text")
    );

    bridge
        .mutate_control_property(
            EFFECT_SEARCH_CONTROL,
            "query",
            UiValue::String("health".to_string()),
        )
        .expect("effect query should update");
    bridge
        .dispatch_control_state(EFFECT_SEARCH_CONTROL, UiEventKind::Change)
        .expect("effect search should dispatch")
        .expect("effect search should bind");
    assert!(bridge
        .control_frame("WorkbenchEffectHealthRegenRow")
        .is_some());
    assert!(bridge
        .control_frame("WorkbenchEffectDamageFireRow")
        .is_none());
    assert!(bridge.control_bool("WorkbenchEffectHealthRegenRow", "selected"));
    assert_eq!(
        Some("GE_HealthRegen".to_string()),
        bridge.control_string("WorkbenchEffectCenterTitle", "text")
    );
}
