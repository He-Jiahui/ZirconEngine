use super::super::super::data::TemplateNodeFrameData;
use super::super::paint_theme::PALETTE;
use super::super::template_axis_value_field_style::{
    AXIS_FIELD_BACKGROUND, AXIS_FIELD_BORDER, AXIS_FIELD_DISABLED_BACKGROUND,
    AXIS_FIELD_DISABLED_BORDER, AXIS_FIELD_HOVER_BACKGROUND,
};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn axis_value_field_kind_matches_transform_axis_inputs_only() {
    assert!(is_workbench_axis_value_field(&axis_node(
        "WorkbenchTransformPositionX",
        "128.4",
    )));
    assert!(is_workbench_axis_value_field(&axis_node(
        "WorkbenchTransformRotationZ",
        "0 deg",
    )));
    assert!(is_workbench_axis_value_field(&axis_node(
        "WorkbenchTransformScaleY",
        "1.00",
    )));
    assert!(!is_workbench_axis_value_field(&label_node(
        "WorkbenchTransformPositionAxisX",
        "X",
    )));
    assert!(!is_workbench_axis_value_field(&axis_node(
        "WorkbenchInputText",
        "Text field",
    )));
}

#[test]
fn axis_value_field_paints_compact_field_and_value() {
    let bytes = paint_template_nodes_for_test(
        96,
        48,
        model_rc(vec![axis_node("WorkbenchTransformPositionX", "128.4")]),
    );

    assert_eq!(pixel_at(&bytes, 96, 22, 8), AXIS_FIELD_BORDER);
    assert_eq!(pixel_at(&bytes, 96, 60, 18), AXIS_FIELD_BACKGROUND);
    assert!(changed_pixel_count(&bytes, 96, 16, 12, 44, 18) > 0);
}

#[test]
fn focused_axis_value_field_uses_focus_border() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.focused = true;

    let bytes = paint_template_nodes_for_test(96, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 96, 22, 8), PALETTE.focus_ring);
    assert_eq!(pixel_at(&bytes, 96, 18, 18), AXIS_FIELD_HOVER_BACKGROUND);
}

#[test]
fn disabled_axis_value_field_uses_muted_surface() {
    let mut node = axis_node("WorkbenchTransformScaleZ", "1.00");
    node.disabled = true;

    let bytes = paint_template_nodes_for_test(96, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 96, 22, 8), AXIS_FIELD_DISABLED_BORDER);
    assert_eq!(pixel_at(&bytes, 96, 60, 18), AXIS_FIELD_DISABLED_BACKGROUND,);
}

#[test]
fn axis_value_field_uses_declared_value_color_when_present() {
    let mut node = axis_node("WorkbenchTransformPositionX", "128.4");
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(146, 158, 164);

    assert_eq!(axis_field_text_color(&node), [146, 158, 164, 255]);
}

fn axis_node(control_id: &str, value: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        value_text: value.into(),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 58.0,
            height: 24.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn label_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        ..TemplatePaneNodeData::default()
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
