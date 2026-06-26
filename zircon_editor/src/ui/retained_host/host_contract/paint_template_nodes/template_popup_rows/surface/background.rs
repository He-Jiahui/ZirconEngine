use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::paint_theme::{METRICS, PALETTE};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::POPUP_ROW_ORDER_OFFSET;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_background(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET,
        Some(PALETTE.popup),
        Some(PALETTE.border),
        METRICS.border_width,
        0.0,
        opacity,
    ));
}
