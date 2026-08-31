use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::identity::SectionTitleIcon;
use super::super::style;

const CUBE_ICON_ASSET: &str = "zircon_editor_shell/activity/cube.svg";
const TRANSFORM_ICON_ASSET: &str = "zircon_editor_shell/inspector/transform.svg";
const MESH_ICON_ASSET: &str = "zircon_editor_shell/inspector/mesh-renderer.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    icon: SectionTitleIcon,
    opacity: f32,
) {
    let color = style::section_icon_color(icon);
    let asset = match icon {
        SectionTitleIcon::Cube => CUBE_ICON_ASSET,
        SectionTitleIcon::Transform => TRANSFORM_ICON_ASSET,
        SectionTitleIcon::Mesh => MESH_ICON_ASSET,
    };
    push_icon_asset_pixels(commands, asset, rect, clip, order, Some(color), opacity);
}
