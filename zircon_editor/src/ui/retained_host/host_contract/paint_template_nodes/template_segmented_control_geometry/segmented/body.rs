use super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_geometry::bounded_extent,
};
use super::super::metrics::{segment_group_label_gap, segment_group_label_height};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_body_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let available_height = bounded_extent(rect.height);
    let label_block_height = if node.label_text.trim().is_empty() {
        0.0
    } else {
        segment_group_label_height() + segment_group_label_gap()
    }
    .min(available_height);

    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + label_block_height + node.layout_offset_y,
        width: bounded_extent(rect.width),
        height: available_height - label_block_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_segmented_body_has_no_drawable_extent() {
        let body = segmented_body_rect(
            &TemplatePaneNodeData::default(),
            &FrameRect {
                x: 12.0,
                y: 8.0,
                width: 0.0,
                height: 0.0,
            },
        );

        assert_eq!(body.width, 0.0);
        assert_eq!(body.height, 0.0);
    }

    #[test]
    fn labeled_segmented_body_stays_inside_a_short_parent_frame() {
        let node = TemplatePaneNodeData {
            label_text: "Render mode".to_string(),
            ..TemplatePaneNodeData::default()
        };
        let body = segmented_body_rect(
            &node,
            &FrameRect {
                x: 12.0,
                y: 8.0,
                width: 200.0,
                height: 10.0,
            },
        );

        assert_eq!((body.x, body.y), (12.0, 18.0));
        assert_eq!((body.width, body.height), (200.0, 0.0));
    }
}
