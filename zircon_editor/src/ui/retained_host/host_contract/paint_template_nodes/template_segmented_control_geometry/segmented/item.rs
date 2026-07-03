use super::super::super::super::data::FrameRect;
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
        width: segment_divider_width().max(1.0),
        height: (segment.height - inset_y * 2.0).max(1.0),
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
        width: (segment.width - inset_x * 2.0).max(1.0),
        height: (segment.height - inset_y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_rect(
    rect: &FrameRect,
    index: usize,
    count: usize,
) -> FrameRect {
    let count = count.max(1);
    let width = rect.width / count as f32;
    FrameRect {
        x: rect.x + width * index as f32,
        y: rect.y,
        width: if index + 1 == count {
            rect.x + rect.width - (rect.x + width * index as f32)
        } else {
            width
        }
        .max(1.0),
        height: rect.height,
    }
}
