//! 标题图标是内置资源而非临时几何；资源 loader 可异步补像素，标题命令仍保持布局占位。

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::identity::SectionTitleIcon;
use super::super::style;

const CUBE_ICON_ASSET: &str = "zircon_editor_shell/activity/cube.svg";
const TRANSFORM_ICON_ASSET: &str = "zircon_editor_shell/inspector/transform.svg";
const MESH_ICON_ASSET: &str = "zircon_editor_shell/inspector/mesh-renderer.svg";

/// 由标题命令在图标矩形完全位于标题内且可见时调用；资源键和主题色都由语义种类决定。
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
