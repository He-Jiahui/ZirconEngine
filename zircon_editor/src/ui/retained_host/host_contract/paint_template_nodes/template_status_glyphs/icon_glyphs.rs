mod kind;
mod snap;
mod target;
mod world;

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use kind::StatusIconKind;
use snap::push_snap_icon;
use target::push_target_icon;
use world::push_world_icon;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        StatusIconKind::Snap => push_snap_icon(commands, rect, clip, order, color, opacity),
        StatusIconKind::World => push_world_icon(commands, rect, clip, order, color, opacity),
        StatusIconKind::Target => push_target_icon(commands, rect, clip, order, color, opacity),
    }
}
