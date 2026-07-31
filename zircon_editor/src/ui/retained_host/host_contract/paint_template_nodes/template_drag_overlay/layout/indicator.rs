use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::DragOverlayMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn indicator_frame(
    node: &TemplatePaneNodeData,
    metrics: &DragOverlayMetrics,
) -> Option<FrameRect> {
    if !node.has_drop_target {
        return None;
    }
    let width = node.drop_target_width.max(0.0);
    let height = node.drop_target_height.max(0.0);
    let thickness = metrics.indicator_thickness.min(width.min(height)).max(0.0);
    if width <= 0.0 || height <= 0.0 || thickness <= 0.0 {
        return None;
    }
    match node.drop_indicator_edge.as_str() {
        "top" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height: thickness,
        }),
        "bottom" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y + (height - thickness).max(0.0),
            width,
            height: thickness,
        }),
        "left" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width: thickness,
            height,
        }),
        "right" => Some(FrameRect {
            x: node.drop_target_x + (width - thickness).max(0.0),
            y: node.drop_target_y,
            width: thickness,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> DragOverlayMetrics {
        DragOverlayMetrics {
            border_width: 1.0,
            preview_radius: 4.0,
            icon_radius: 4.0,
            font_size: 13.33,
            line_height: 16.0,
            icon_left: 12.0,
            icon_size: 16.0,
            text_left_with_icon: 35.0,
            text_right_inset: 12.0,
            indicator_thickness: 2.0,
        }
    }

    #[test]
    fn indicator_scales_into_a_small_drop_target() {
        let node = TemplatePaneNodeData {
            has_drop_target: true,
            drop_target_x: 6.0,
            drop_target_y: 10.0,
            drop_target_width: 1.0,
            drop_target_height: 1.0,
            drop_indicator_edge: "bottom".into(),
            ..TemplatePaneNodeData::default()
        };
        let indicator = indicator_frame(&node, &metrics()).expect("small target has an indicator");

        assert_eq!(indicator.x, 6.0);
        assert_eq!(indicator.y, 10.0);
        assert_eq!(indicator.width, 1.0);
        assert_eq!(indicator.height, 1.0);
    }

    #[test]
    fn indicator_skips_a_collapsed_drop_target() {
        let node = TemplatePaneNodeData {
            has_drop_target: true,
            drop_target_width: 0.0,
            drop_target_height: 4.0,
            drop_indicator_edge: "left".into(),
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(indicator_frame(&node, &metrics()), None);
    }
}
