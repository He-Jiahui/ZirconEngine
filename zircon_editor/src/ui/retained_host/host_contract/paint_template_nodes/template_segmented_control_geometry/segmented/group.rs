use super::super::super::super::data::FrameRect;
use super::super::metrics::segment_group_label_height;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_group_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: segment_group_label_height(),
    }
}
