use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const INSPECTOR_MESH_ICON: &str = "zircon_editor_shell/inspector/mesh-renderer.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_icon_asset_pixels(
        commands,
        INSPECTOR_MESH_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}
