use super::super::super::super::data::FrameRect;
use super::super::metrics::{SLIDER_HORIZONTAL_INSET, SLIDER_VALUE_WIDTH};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_rect(
    rect: &FrameRect,
) -> Option<FrameRect> {
    if rect.width < 132.0 {
        return None;
    }
    let height = (rect.height - 6.0).clamp(18.0, 24.0);
    Some(FrameRect {
        x: rect.x + rect.width - SLIDER_HORIZONTAL_INSET - SLIDER_VALUE_WIDTH,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: SLIDER_VALUE_WIDTH,
        height,
    })
}
