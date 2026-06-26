use super::super::super::super::data::FrameRect;
use super::super::metrics::segment_selected_inset;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_rect(
    segment: &FrameRect,
) -> FrameRect {
    inset_rect(segment, segment_selected_inset())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_rect(
    selected_rect: &FrameRect,
    underline_height: f32,
) -> FrameRect {
    FrameRect {
        x: selected_rect.x,
        y: selected_rect.y + (selected_rect.height - underline_height).max(0.0),
        width: selected_rect.width,
        height: underline_height.min(selected_rect.height).max(1.0),
    }
}

fn inset_rect(rect: &FrameRect, inset: f32) -> FrameRect {
    FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2.0).max(1.0),
        height: (rect.height - inset * 2.0).max(1.0),
    }
}
