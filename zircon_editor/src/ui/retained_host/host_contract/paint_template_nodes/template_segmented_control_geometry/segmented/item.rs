use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::super::metrics::{
    segment_divider_inset_y, segment_divider_width, segment_text_inset_x, segment_text_inset_y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_divider_rect(
    segment: &FrameRect,
) -> FrameRect {
    let inset_y = segment_divider_inset_y();
    FrameRect {
        x: segment.x,
        y: segment.y + inset_y,
        width: bounded_extent(segment.width).min(bounded_extent(segment_divider_width())),
        height: bounded_extent(segment.height - inset_y * 2.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_label_rect(
    segment: &FrameRect,
) -> FrameRect {
    let inset_x = segment_text_inset_x();
    let inset_y = segment_text_inset_y();
    FrameRect {
        x: segment.x + inset_x,
        y: segment.y + inset_y,
        width: bounded_extent(segment.width - inset_x * 2.0),
        height: bounded_extent(segment.height - inset_y * 2.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_rect(
    rect: &FrameRect,
    index: usize,
    count: usize,
) -> FrameRect {
    let count = count.max(1);
    let width = bounded_extent(rect.width) / count as f32;
    FrameRect {
        x: rect.x + width * index as f32,
        y: rect.y,
        width: if index + 1 == count {
            rect.x + rect.width - (rect.x + width * index as f32)
        } else {
            width
        }
        .max(0.0),
        height: bounded_extent(rect.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_segment_has_no_divider_or_label_extent() {
        let segment = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 0.0,
        };

        let divider = segment_divider_rect(&segment);
        let label = segment_label_rect(&segment);
        let allocated = segment_rect(&segment, 0, 0);

        assert_eq!((divider.width, divider.height), (0.0, 0.0));
        assert_eq!((label.width, label.height), (0.0, 0.0));
        assert_eq!((allocated.width, allocated.height), (0.0, 0.0));
    }
}
