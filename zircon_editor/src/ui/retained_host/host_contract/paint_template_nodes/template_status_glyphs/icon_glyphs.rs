mod kind;

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::geometry::has_paintable_status_glyph_extent;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use kind::StatusIconKind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    color: [u8; 4],
    opacity: f32,
) {
    if !has_paintable_status_glyph_extent(rect) {
        return;
    }
    let icon_name = match kind {
        StatusIconKind::Snap => "snap",
        StatusIconKind::World => "globe",
        StatusIconKind::Target => "target",
    };
    push_icon_asset_pixels(commands, icon_name, rect, clip, order, Some(color), opacity);
}
