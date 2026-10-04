//! 打开的菜单行与下拉选项的统一补充绘制入口。菜单列表优先于选项列表，避免同一节点重复画两套弹层。
//! 此入口返回unit；在专用Dropdown与fallback链上都可能被调用，调用方需依赖popup_open和实际投影数据避免重复提交。

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod content;
mod geometry;
mod layers;
mod menu;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) mod metrics;
mod options;
mod surface;
mod text;

use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use geometry::has_paintable_popup_row_extent;
use menu::push_menu_row_commands;
use options::push_option_row_commands;

#[cfg(test)]
use super::template_popup_row_adornments::{
    menu_item_has_flag, menu_row_adornment_kind, PopupRowAdornmentKind,
};
#[cfg(test)]
use content::popup_row_content_style;
#[cfg(test)]
use menu::popup_menu_row_style;
#[cfg(test)]
use options::popup_option_row_style;

/// 调用方须分别提供控件或弹层rect、可用bounds及绘制clip：菜单直接按rect布局，选项由template_popup_layout计算最终弹层位置。
/// 只有open且有有效投影行时提交；优先菜单，再退回选项。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_popup_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    bounds: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !node.popup_open
        || !has_paintable_popup_row_extent(rect)
        || !has_paintable_popup_row_extent(clip)
        || intersect(rect, clip).is_none()
    {
        return;
    }
    if node.structured_menu_items.row_count() > 0 {
        push_menu_row_commands(commands, node, rect, clip, order, opacity);
    } else if node.structured_options.row_count() > 0 {
        push_option_row_commands(commands, node, rect, bounds, clip, order, opacity);
    }
}

#[cfg(test)]
#[path = "template_popup_rows_tests/tests/mod.rs"]
mod tests;
