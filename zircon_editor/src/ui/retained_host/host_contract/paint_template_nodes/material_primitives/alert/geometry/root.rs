use crate::ui::retained_host::host_contract::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
