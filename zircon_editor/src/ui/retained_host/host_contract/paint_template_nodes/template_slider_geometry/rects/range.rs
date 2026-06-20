use super::super::super::super::data::FrameRect;
use super::super::metrics::SLIDER_VALUE_WIDTH;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_value_rect(
    rect: &FrameRect,
    track_rect: &FrameRect,
) -> Option<FrameRect> {
    if rect.height < 42.0 || track_rect.width < SLIDER_VALUE_WIDTH {
        return None;
    }
    Some(FrameRect {
        x: track_rect.x,
        y: track_rect.y + 10.0,
        width: SLIDER_VALUE_WIDTH,
        height: 20.0,
    })
}
