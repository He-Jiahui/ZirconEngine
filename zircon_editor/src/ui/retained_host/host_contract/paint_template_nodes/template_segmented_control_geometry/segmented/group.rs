use super::super::super::super::data::FrameRect;
use super::super::metrics::SEGMENT_GROUP_LABEL_HEIGHT;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_group_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: SEGMENT_GROUP_LABEL_HEIGHT,
    }
}
