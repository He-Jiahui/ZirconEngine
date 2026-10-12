//! 将内置按钮图标连接到编辑器 SVG 资源；None 不产生命令，资源缺失遵循统一资产加载行为。

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::identity::ButtonGlyph;

/// 内容域已确定图标语义、槽位、颜色和层级后调用；资源加载可以暂时不输出像素。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    glyph: ButtonGlyph,
    color: [u8; 4],
    opacity: f32,
) {
    let Some(asset_name) = button_glyph_asset_name(glyph) else {
        return;
    };
    push_icon_asset_pixels(
        commands,
        asset_name,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}

fn button_glyph_asset_name(glyph: ButtonGlyph) -> Option<&'static str> {
    match glyph {
        ButtonGlyph::Plus => Some("zircon_editor_shell/controls/add.svg"),
        ButtonGlyph::Trash => Some("zircon_editor_shell/controls/delete.svg"),
        ButtonGlyph::ChevronDown => Some("zircon_editor_shell/toolbar/dropdown.svg"),
        ButtonGlyph::None => None,
    }
}
