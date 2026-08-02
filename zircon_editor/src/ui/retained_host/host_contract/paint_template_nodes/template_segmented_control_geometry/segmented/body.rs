use super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_geometry::bounded_extent,
};
use super::super::metrics::{segment_group_label_gap, segment_group_label_height};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_body_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let label_block_height = if node.label_text.trim().is_empty() {
        0.0
    } else {
        segment_group_label_height() + segment_group_label_gap()
    };

    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + label_block_height + node.layout_offset_y,
        width: bounded_extent(rect.width),
        height: bounded_extent(rect.height - label_block_height),
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
}
