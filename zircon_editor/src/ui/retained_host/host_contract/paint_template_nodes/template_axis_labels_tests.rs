use super::super::super::data::TemplateNodeFrameData;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn axis_label_kind_matches_transform_axis_labels_and_scale_link() {
    assert_eq!(
        axis_label_kind(&label_node("WorkbenchTransformPositionAxisX", "X")),
        Some(AxisLabelKind::Axis("X"))
    );
    assert_eq!(
        axis_label_kind(&label_node("WorkbenchTransformScaleLink", "")),
        Some(AxisLabelKind::ScaleLink)
    );
    assert_eq!(
        axis_label_kind(&label_node("WorkbenchTransformScaleX", "1.00")),
        None
    );
}

#[test]
fn scale_link_label_paints_link_glyph_without_text_fallback() {
    let bytes = paint_template_nodes_for_test(
        48,
        40,
        model_rc(vec![label_node("WorkbenchTransformScaleLink", "")]),
    );

    assert!(changed_pixel_count(&bytes, 48, 12, 14, 20, 12) > 0);
    assert_eq!(changed_pixel_count(&bytes, 48, 34, 8, 8, 24), 0);
}

#[test]
fn transform_axis_label_paints_compact_axis_text() {
    let bytes = paint_template_nodes_for_test(
        48,
        40,
        model_rc(vec![label_node("WorkbenchTransformRotationAxisY", "Y")]),
    );

    assert!(changed_pixel_count(&bytes, 48, 8, 10, 14, 20) > 0);
    assert_eq!(changed_pixel_count(&bytes, 48, 28, 10, 12, 20), 0);
}

#[test]
fn transform_axis_label_uses_audited_axis_tones() {
    let mut position_axis = label_node("WorkbenchTransformPositionAxisX", "X");
    position_axis.label_color =
        crate::ui::retained_host::primitives::Color::from_rgb_u8(86, 104, 113);
    assert_eq!(axis_label_color(&position_axis), [86, 104, 113, 255]);
    assert_eq!(
        axis_label_color(&label_node("WorkbenchTransformRotationAxisY", "Y")),
        AXIS_LABEL_COLOR
    );
    assert_eq!(
        axis_label_color(&label_node("WorkbenchTransformScaleAxisZ", "Z")),
        AXIS_LABEL_SCALE_COLOR
    );
}

#[test]
fn scale_link_label_honors_audited_icon_offset() {
    let mut node = label_node("WorkbenchTransformScaleLink", "");
    node.layout_offset_x = -12.0;
    let (start_x, start_y) = scale_link_origin(&node, &frame_rect(&node.frame));

    assert_eq!(start_x, 0.0);
    assert_eq!(start_y, 16.5);
}

fn label_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 18.0,
            height: 24.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn frame_rect(frame: &TemplateNodeFrameData) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
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
