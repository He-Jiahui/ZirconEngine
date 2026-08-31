use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const INSPECTOR_CHECK_ICON: &str = "zircon_editor_shell/controls/check.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_check_tick(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_icon_asset_pixels(
        commands,
        INSPECTOR_CHECK_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}
