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
