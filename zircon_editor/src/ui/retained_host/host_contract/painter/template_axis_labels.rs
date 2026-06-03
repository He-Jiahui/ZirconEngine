use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const AXIS_LABEL_FONT_SIZE: f32 = 11.0;
const AXIS_LABEL_COLOR: [u8; 4] = [129, 136, 140, 255];
const AXIS_LABEL_SCALE_COLOR: [u8; 4] = [126, 132, 136, 255];
const AXIS_LABEL_DISABLED: [u8; 4] = [82, 93, 100, 255];
const AXIS_LABEL_LINK_COLOR: [u8; 4] = [145, 157, 164, 255];
const AXIS_LABEL_LINK_DISABLED: [u8; 4] = [82, 93, 100, 255];
const LINK_LOBE_WIDTH: f32 = 6.0;
const LINK_LOBE_HEIGHT: f32 = 7.0;
const LINK_OVERLAP: f32 = 2.0;

pub(super) fn push_axis_label_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match axis_label_kind(node) {
        Some(AxisLabelKind::Axis(axis)) => {
            push_axis_text(commands, node, rect, clip, order, axis, opacity);
            true
        }
        Some(AxisLabelKind::ScaleLink) => {
            push_scale_link(commands, node, rect, clip, order, opacity);
            true
        }
        None => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AxisLabelKind {
    Axis(&'static str),
    ScaleLink,
}

fn axis_label_kind(node: &TemplatePaneNodeData) -> Option<AxisLabelKind> {
    if !matches!(node.role.as_str(), "Label" | "Icon" | "SvgIcon") {
        return None;
    }
    let control_id = node.control_id.as_str();
    if control_id == "WorkbenchTransformScaleLink" {
        return Some(AxisLabelKind::ScaleLink);
    }
    transform_axis_label(control_id).map(AxisLabelKind::Axis)
}

fn transform_axis_label(control_id: &str) -> Option<&'static str> {
    let field = control_id.strip_prefix("WorkbenchTransform")?;
    if field.ends_with("AxisX") {
        Some("X")
    } else if field.ends_with("AxisY") {
        Some("Y")
    } else if field.ends_with("AxisZ") {
        Some("Z")
    } else {
        None
    }
}

fn push_axis_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    axis: &str,
    opacity: f32,
) {
    let label = if node.text.trim().is_empty() {
        axis
    } else {
        node.text.trim()
    };
    let line_height = AXIS_LABEL_FONT_SIZE * 1.2;
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: rect.width.max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label.to_string(),
        axis_label_color(node),
        AXIS_LABEL_FONT_SIZE,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn axis_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        AXIS_LABEL_DISABLED
    } else if node.label_color.a > 0 {
        [
            node.label_color.r,
            node.label_color.g,
            node.label_color.b,
            node.label_color.a,
        ]
    } else if node
        .control_id
        .as_str()
        .starts_with("WorkbenchTransformScaleAxis")
    {
        AXIS_LABEL_SCALE_COLOR
    } else {
        AXIS_LABEL_COLOR
    }
}

fn push_scale_link(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = if node.disabled {
        AXIS_LABEL_LINK_DISABLED
    } else {
        AXIS_LABEL_LINK_COLOR
    };
    let (start_x, start_y) = scale_link_origin(node, rect);
    for lobe in [
        FrameRect {
            x: start_x,
            y: start_y,
            width: LINK_LOBE_WIDTH,
            height: LINK_LOBE_HEIGHT,
        },
        FrameRect {
            x: start_x + LINK_LOBE_WIDTH - LINK_OVERLAP,
            y: start_y,
            width: LINK_LOBE_WIDTH,
            height: LINK_LOBE_HEIGHT,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            lobe,
            Some(clip.clone()),
            order,
            None,
            Some(color),
            1.0,
            3.0,
            opacity,
        ));
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: start_x + LINK_LOBE_WIDTH - LINK_OVERLAP + 1.0,
            y: start_y + LINK_LOBE_HEIGHT * 0.5,
            width: LINK_OVERLAP,
            height: 1.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn scale_link_origin(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let total_width = LINK_LOBE_WIDTH * 2.0 - LINK_OVERLAP;
    (
        rect.x + (rect.width - total_width).max(0.0) * 0.5 + node.layout_offset_x,
        rect.y + (rect.height - LINK_LOBE_HEIGHT).max(0.0) * 0.5 + node.layout_offset_y,
    )
}

#[cfg(test)]
mod tests {
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
}
