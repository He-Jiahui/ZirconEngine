use zircon_runtime_interface::ui::{component::UiValue, layout::UiSize};

use super::*;

#[test]
fn structured_dropdown_selection_keeps_machine_value_and_display_label() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");
    bridge
        .mutate_control_property(
            "WorkbenchMaterialDomainDropdown",
            "options",
            UiValue::Array(vec![
                UiValue::String("surface|label=Surface".to_string()),
                UiValue::String("post_process|label=Post Process".to_string()),
            ]),
        )
        .expect("structured options should project");

    assert!(bridge
        .select_dropdown_option("WorkbenchMaterialDomainDropdown", "post_process")
        .expect("structured option selection should apply"));
    assert_eq!(
        bridge.control_string("WorkbenchMaterialDomainDropdown", "value"),
        Some("post_process".to_string())
    );
    assert_eq!(
        bridge.control_string("WorkbenchMaterialDomainDropdown", "value_text"),
        Some("Post Process".to_string())
    );
}
