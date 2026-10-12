use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use super::*;

#[test]
fn task_phase_graph_and_playtest_share_one_ability_profile() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchAbilityTaskActivateRow", "selected"));
    assert!(bridge.control_bool("WorkbenchAbilityPhaseActivateRow", "selected"));

    bridge
        .dispatch_control_state("WorkbenchAbilityTaskCostRow", UiEventKind::Click)
        .expect("cost task should dispatch")
        .expect("cost task should bind");
    assert!(bridge.control_bool("WorkbenchAbilityTaskCostRow", "selected"));
    assert!(bridge.control_bool("WorkbenchAbilityPhaseCostRow", "selected"));
    assert_eq!(
        Some("GA_DashAttack / Cost".to_string()),
        bridge.control_string("WorkbenchAbilityCenterTitle", "text")
    );

    bridge
        .dispatch_control_state("WorkbenchAbilityGraphRow", UiEventKind::Click)
        .expect("full graph should dispatch")
        .expect("full graph should bind");
    assert!(bridge.control_bool("WorkbenchAbilityAssetRow", "selected"));
    assert!(bridge.control_bool("WorkbenchAbilityGraphRow", "selected"));
    assert_eq!(
        Some("Full graph ready   1.22 s   GA_DashAttack".to_string()),
        bridge.control_string("WorkbenchAbilityOutputRow", "value_text")
    );

    assert!(bridge
        .select_dropdown_option("WorkbenchAbilityNetPolicyDropdown", "client_predicted")
        .expect("net policy should select"));
    bridge
        .mutate_control_property(
            "WorkbenchAbilityNameField",
            "value",
            UiValue::String("GA_CustomDash".to_string()),
        )
        .expect("ability name should edit");
    bridge
        .mutate_control_property(
            "WorkbenchAbilityCooldownField",
            "value",
            UiValue::String("1.25s".to_string()),
        )
        .expect("cooldown should edit");

    bridge
        .dispatch_control_state("WorkbenchAbilityPlaytestButton", UiEventKind::Click)
        .expect("playtest should dispatch")
        .expect("playtest should bind");
    assert_eq!(
        Some(
            "Playtest queued   Client Predicted   full graph   GA_CustomDash   cooldown 1.25s"
                .to_string(),
        ),
        bridge.control_string("WorkbenchAbilityOutputRow", "value_text")
    );
}

#[test]
fn task_navigation_preserves_edited_ability_properties() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge
        .select_dropdown_option("WorkbenchAbilityNetPolicyDropdown", "client_predicted")
        .expect("net policy should select"));
    bridge
        .mutate_control_property(
            "WorkbenchAbilityNameField",
            "value",
            UiValue::String("GA_CustomDash".to_string()),
        )
        .expect("ability name should edit");
    bridge
        .mutate_control_property(
            "WorkbenchAbilityCooldownField",
            "value",
            UiValue::String("1.25s".to_string()),
        )
        .expect("cooldown should edit");

    bridge
        .dispatch_control_state("WorkbenchAbilityTaskCostRow", UiEventKind::Click)
        .expect("cost task should dispatch")
        .expect("cost task should bind");

    assert_eq!(
        bridge.control_string("WorkbenchAbilityNetPolicyDropdown", "value"),
        Some("client_predicted".to_string())
    );
    assert_eq!(
        bridge.control_string("WorkbenchAbilityNetPolicyDropdown", "value_text"),
        Some("Client Predicted".to_string())
    );
    assert_eq!(
        bridge.control_string("WorkbenchAbilityNameField", "value"),
        Some("GA_CustomDash".to_string())
    );
    assert_eq!(
        bridge.control_string("WorkbenchAbilityCooldownField", "value"),
        Some("1.25s".to_string())
    );
}
