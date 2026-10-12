//! 根层级及受保护场景行的锁标识；是否出现由 actions 的行语义判定决定。

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;

const TREE_LOCK_ICON: &str = "zircon_editor_shell/scene/lock.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_lock_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_icon_asset_pixels(
        commands,
        TREE_LOCK_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}
