use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn parameter_graph_and_compile_share_one_material_profile() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchMaterialBaseColorRow", "selected"));
    assert!(bridge.control_bool("WorkbenchMaterialNodeRow01", "selected"));

    assert!(bridge
        .select_dropdown_option(MATERIAL_DOMAIN_DROPDOWN, "post_process")
        .expect("material domain should select"));
    assert!(bridge
        .select_dropdown_option(MATERIAL_BLEND_DROPDOWN, "masked")
        .expect("material blend should select"));
    bridge
        .mutate_control_property(
            "WorkbenchMaterialPreviewField",
            "value",
            UiValue::String("Plane".to_string()),
        )
        .expect("material preview should edit");

    bridge
        .dispatch_control_state("WorkbenchMaterialNodeRow02", UiEventKind::Click)
        .expect("roughness node should dispatch")
        .expect("roughness node should bind");
    assert!(bridge.control_bool("WorkbenchMaterialRoughnessRow", "selected"));
    assert!(bridge.control_bool("WorkbenchMaterialNodeRow02", "selected"));
    assert_eq!(
        Some("M_Rock_Cliff / Roughness".to_string()),
        bridge.control_string("WorkbenchMaterialCenterTitle", "text")
    );

    bridge
        .dispatch_control_state("WorkbenchMaterialCompileButton", UiEventKind::Click)
        .expect("material compile should dispatch")
        .expect("material compile should bind");
    assert_eq!(
        Some("Roughness compiled   Post Process / Masked   2 warnings".to_string()),
        bridge.control_string("WorkbenchMaterialOutputRow", "text")
    );
    assert_eq!(
        Some("M_Rock_Cliff / Plane".to_string()),
        bridge.control_string("WorkbenchMaterialCenterTitle", "text")
    );
}
