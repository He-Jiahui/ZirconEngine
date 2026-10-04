use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn tree_graph_details_and_validation_share_one_behavior_profile() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchBehaviorSelectorRow", "selected"));
    assert!(bridge.control_bool("WorkbenchBehaviorNodeRow01", "selected"));

    bridge
        .dispatch_control_state("WorkbenchBehaviorAttackRow", UiEventKind::Click)
        .expect("attack task should dispatch")
        .expect("attack task should bind");
    assert!(bridge.control_bool("WorkbenchBehaviorAttackRow", "selected"));
    assert!(bridge.control_bool("WorkbenchBehaviorNodeRow02", "selected"));
    assert_eq!(
        Some("BT_Enemy / Attack Target".to_string()),
        bridge.control_string("WorkbenchBehaviorCenterTitle", "text")
    );
    assert_eq!(
        Some("Executing".to_string()),
        bridge.control_string("WorkbenchBehaviorStateField", "value")
    );

    bridge
        .dispatch_control_state("WorkbenchBehaviorNodeRow03", UiEventKind::Click)
        .expect("cooldown decorator should dispatch")
        .expect("cooldown decorator should bind");
    assert!(bridge.control_bool("WorkbenchBehaviorAttackRow", "selected"));
    assert!(bridge.control_bool("WorkbenchBehaviorNodeRow03", "selected"));
    assert_eq!(
        Some("Cooldown 0.8 s".to_string()),
        bridge.control_string("WorkbenchBehaviorStateField", "value")
    );
    for (control_id, value) in [
        ("WorkbenchBehaviorBlackboardField", "BB_Custom"),
        ("WorkbenchBehaviorAiField", "AIController_Custom"),
        ("WorkbenchBehaviorStateField", "Paused"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("behavior detail should edit");
    }

    bridge
        .dispatch_control_state("WorkbenchBehaviorValidateButton", UiEventKind::Click)
        .expect("behavior validation should dispatch")
        .expect("behavior validation should bind");
    assert_eq!(
        Some("Validated BB_Custom / AIController_Custom   Paused".to_string()),
        bridge.control_string("WorkbenchBehaviorOutputRow", "text")
    );
}
