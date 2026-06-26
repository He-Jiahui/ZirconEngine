use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::segments::{local_rect, push_segments};

const TREE_LOCK_ICON: &str = "zircon_editor_shell/scene/lock.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_lock_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        TREE_LOCK_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }

    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 4.0, 6.0, 7.0, 6.0),
            local_rect(rect, 5.0, 3.0, 5.0, 1.0),
            local_rect(rect, 4.0, 4.0, 1.0, 3.0),
            local_rect(rect, 10.0, 4.0, 1.0, 3.0),
        ],
    );
}
