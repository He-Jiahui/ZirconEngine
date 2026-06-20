use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::POPUP_ROW_ORDER_OFFSET;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_separator(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let separator = FrameRect {
        x: row_rect.x + 8.0,
        y: row_rect.y + row_rect.height * 0.5,
        width: (row_rect.width - 16.0).max(1.0),
        height: 1.0,
    };
    if intersect(&separator, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        separator,
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET + 2,
        Some(PALETTE.border),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
