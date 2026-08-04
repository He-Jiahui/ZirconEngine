use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::bounded_extent;
use super::super::metrics::segment_group_label_height;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_group_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: bounded_extent(rect.width),
        height: bounded_extent(segment_group_label_height()).min(bounded_extent(rect.height)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_label_stays_inside_a_collapsed_or_invalid_segmented_frame() {
        let label = segmented_group_label_rect(&FrameRect {
            x: 12.0,
            y: 8.0,
            width: f32::NAN,
            height: 0.0,
        });

        assert_eq!(label.x, 12.0);
        assert_eq!(label.y, 8.0);
        assert_eq!((label.width, label.height), (0.0, 0.0));
    }
}
