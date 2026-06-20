use super::super::super::data::FrameRect;

const AXIS_FIELD_MAX_HEIGHT: f32 = 26.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let height = rect.height.min(AXIS_FIELD_MAX_HEIGHT).round().max(0.0);
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - height).max(0.0) * 0.5).round(),
        width: rect.width.round().max(0.0),
        height,
    }
}
