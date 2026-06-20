use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::INDICATOR_THICKNESS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn indicator_frame(
    node: &TemplatePaneNodeData,
) -> Option<FrameRect> {
    if !node.has_drop_target {
        return None;
    }
    let width = node.drop_target_width.max(1.0);
    let height = node.drop_target_height.max(1.0);
    match node.drop_indicator_edge.as_str() {
        "top" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height: INDICATOR_THICKNESS,
        }),
        "bottom" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y + (height - INDICATOR_THICKNESS).max(0.0),
            width,
            height: INDICATOR_THICKNESS,
        }),
        "left" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width: INDICATOR_THICKNESS,
            height,
        }),
        "right" => Some(FrameRect {
            x: node.drop_target_x + (width - INDICATOR_THICKNESS).max(0.0),
            y: node.drop_target_y,
            width: INDICATOR_THICKNESS,
            height,
        }),
        "inside" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height,
        }),
        _ => None,
    }
}
