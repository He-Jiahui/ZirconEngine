use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn context_parameter_and_simulation_keep_distinct_state_domains() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchVfxEmitterRow", "selected"));
    assert!(bridge.control_bool("WorkbenchVfxSpawnRow", "selected"));

    bridge
        .dispatch_control_state("WorkbenchVfxCurveRow", UiEventKind::Click)
        .expect("curve context should dispatch")
        .expect("curve context should bind");
    bridge
        .dispatch_control_state("WorkbenchVfxMaterialRow", UiEventKind::Click)
        .expect("material parameter should dispatch")
        .expect("material parameter should bind");
    assert!(bridge.control_bool("WorkbenchVfxCurveRow", "selected"));
    assert!(bridge.control_bool("WorkbenchVfxMaterialRow", "selected"));
    for (control_id, value) in [
        ("WorkbenchVfxSystemField", "P_CustomBurst"),
        ("WorkbenchVfxBoundsField", "250 cm"),
        ("WorkbenchVfxSortField", "Age"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("VFX property should edit");
    }

    bridge
        .dispatch_control_state("WorkbenchVfxSimulateButton", UiEventKind::Click)
        .expect("simulation should dispatch")
        .expect("simulation should bind");
    assert_eq!(
        Some("Simulation: 250 cm / Age   Material M_Bolt_01   60 fps".to_string()),
        bridge.control_string("WorkbenchVfxOutputRow", "text")
    );
    assert_eq!(
        Some("P_CustomBurst / Curve: Spawn Rate".to_string()),
        bridge.control_string("WorkbenchVfxCenterTitle", "text")
    );
}
