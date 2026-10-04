use super::alert_color_token;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

#[test]
fn mixed_case_material_alert_variant_preserves_color_precedence() {
    let node = TemplatePaneNodeData {
        component_variant: "colorWaRnInG colorSuccess".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(alert_color_token(&node), "success");
}
