//! 为内容编排提供语义选择、占位和资产入口；领先图标与尾随菜单箭头是两个不同布局角色。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_button_glyphs::{
    button_icon_size, push_button_glyph, ButtonGlyph,
};
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::identity::{button_identity_contains, button_identity_values};
use super::layout::content_centered_y;
use super::metrics::{button_chevron_reserve, button_icon_gap, trailing_glyph_inset};

/// 从节点的显示身份字段推断内置图标，优先危险操作，再菜单，再添加；显式资产名称由内容入口优先处理。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_glyph(
    node: &TemplatePaneNodeData,
) -> ButtonGlyph {
    let values = button_identity_values(node);
    if ["delete", "trash", "danger"]
        .iter()
        .any(|needle| button_identity_contains(&values, needle))
    {
        ButtonGlyph::Trash
    } else if ["dropdown", "drop-down", "menu"]
        .iter()
        .any(|needle| button_identity_contains(&values, needle))
    {
        ButtonGlyph::ChevronDown
    } else if ["icon", "add", "plus"]
        .iter()
        .any(|needle| button_identity_contains(&values, needle))
    {
        ButtonGlyph::Plus
    } else {
        ButtonGlyph::None
    }
}

/// 为领先图标及其与标签的间距预留槽宽；有无标签由调用方一次确定。
pub(super) fn button_glyph_width(
    node: &TemplatePaneNodeData,
    glyph: ButtonGlyph,
    has_label: bool,
) -> f32 {
    if has_leading_asset_icon(node) || has_leading_glyph(glyph) {
        button_icon_size()
            + if has_label {
                button_icon_gap(node)
            } else {
                0.0
            }
    } else {
        0.0
    }
}

pub(super) fn chevron_width(glyph: ButtonGlyph) -> f32 {
    if has_trailing_chevron(glyph) {
        button_chevron_reserve()
    } else {
        0.0
    }
}

pub(super) fn has_leading_glyph(glyph: ButtonGlyph) -> bool {
    matches!(glyph, ButtonGlyph::Plus | ButtonGlyph::Trash)
}

pub(super) fn has_leading_asset_icon(node: &TemplatePaneNodeData) -> bool {
    !node.icon_name.trim().is_empty()
}

pub(super) fn has_trailing_chevron(glyph: ButtonGlyph) -> bool {
    glyph == ButtonGlyph::ChevronDown
}

pub(super) fn leading_glyph_rect(rect: &FrameRect, x: f32) -> FrameRect {
    let icon_size = button_icon_size();
    FrameRect {
        x,
        y: content_centered_y(rect, icon_size),
        width: icon_size,
        height: icon_size,
    }
}

pub(super) fn trailing_glyph_rect(rect: &FrameRect) -> FrameRect {
    let icon_size = button_icon_size();
    FrameRect {
        x: rect.x + rect.width - trailing_glyph_inset() - icon_size,
        y: content_centered_y(rect, icon_size),
        width: icon_size,
        height: icon_size,
    }
}

pub(super) fn push_content_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    glyph: ButtonGlyph,
    color: [u8; 4],
    opacity: f32,
) {
    push_button_glyph(commands, rect, clip, order, glyph, color, opacity);
}

/// 返回统一资产入口是否立即追加命令；false 允许内容入口使用内置语义图标，异步资源随后由宿主重绘。
pub(super) fn push_content_asset_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) -> bool {
    push_icon_asset_pixels(
        commands,
        node.icon_name.as_str(),
        rect,
        clip,
        order,
        Some(color),
        opacity,
    )
}
