//! 图标按钮只绘制实例声明的资产图标；缺失名称或资源不产生手工像素回退，避免与资产设计脱节。

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_icon_assets::push_icon_asset_pixels;

/// 消费实例 icon_name；未声明或加载失败时保持空内容，真实资产重新可用时由统一加载器请求重绘。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if node.icon_name.trim().is_empty() {
        return;
    }
    push_icon_asset_pixels(
        commands,
        node.icon_name.as_str(),
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}

#[cfg(test)]
#[path = "tests/template_icon_button_glyphs.rs"]
mod tests;
