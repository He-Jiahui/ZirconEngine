use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn agent_selection_and_simulation_share_one_profile_projection() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchPerceptionGuardRow", "selected"));
    assert_eq!(
        Some("AI_Guard_01 Perception".to_string()),
        bridge.control_string("WorkbenchPerceptionCenterTitle", "text")
    );

    bridge
        .dispatch_control_state("WorkbenchPerceptionSniperRow", UiEventKind::Click)
        .expect("sniper profile should dispatch")
        .expect("sniper profile should bind");
    assert_eq!(
        Some("Sniper_Perception Map".to_string()),
        bridge.control_string("WorkbenchPerceptionCenterTitle", "text")
    );
    assert_eq!(
        Some("Sniper Perception".to_string()),
        bridge.control_string("WorkbenchPerceptionConfigDropdown", "value_text")
    );
    assert_eq!(
        Some("Enemies".to_string()),
        bridge.control_string("WorkbenchPerceptionTeamField", "value")
    );

    bridge
        .dispatch_control_state("WorkbenchPerceptionHearingPulseRow", UiEventKind::Click)
        .expect("hearing map item should dispatch")
        .expect("hearing map item should bind");
    assert!(bridge.control_bool("WorkbenchPerceptionSniperRow", "selected"));
    assert!(bridge.control_bool("WorkbenchPerceptionHearingPulseRow", "selected"));
    assert_eq!(
        Some("Sniper_Perception   inspecting Hearing Pulse".to_string()),
        bridge.control_string("WorkbenchPerceptionEventRow", "value_text")
    );
    assert!(bridge
        .select_dropdown_option("WorkbenchPerceptionConfigDropdown", "Guard_Perception")
        .expect("perception config should select"));
    for (control_id, value) in [
        ("WorkbenchPerceptionLosField", "Off"),
        ("WorkbenchPerceptionTeamField", "Friendlies"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("perception configuration should edit");
    }

    bridge
        .dispatch_control_state("WorkbenchPerceptionSimulateButton", UiEventKind::Click)
        .expect("simulation should dispatch")
        .expect("simulation should bind");
    assert_eq!(
        Some("LOS Off / Friendlies   00:08.0".to_string()),
        bridge.control_string("WorkbenchPerceptionEventRow", "value_text")
    );
    assert_eq!(
        Some("Sniper_Perception / Guard Perception".to_string()),
        bridge.control_string("WorkbenchPerceptionCenterTitle", "text")
    );
}
