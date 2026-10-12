use crate::ui::retained_host::primitives::Color;

use super::*;

fn node(control_id: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn declared_label_color_is_ignored_when_alpha_is_zero() {
    let mut axis = node("WorkbenchTransformPositionAxisX");
    axis.label_color = Color::from_argb_u8(0, 255, 255, 255);

    assert_eq!(declared_label_color(&axis), None);
}

#[test]
fn scale_axis_matches_only_transform_scale_axis_prefix() {
    assert!(is_scale_axis(&node("WorkbenchTransformScaleAxisX")));
    assert!(!is_scale_axis(&node("WorkbenchTransformScaleLink")));
}
