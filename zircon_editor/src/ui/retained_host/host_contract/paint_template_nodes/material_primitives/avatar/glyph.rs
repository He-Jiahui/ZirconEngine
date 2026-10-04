use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;

const AVATAR_FALLBACK_ICON: &str = "zircon_editor_shell/controls/person.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_fallback_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let _ = push_icon_asset_pixels(
        commands,
        AVATAR_FALLBACK_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}
