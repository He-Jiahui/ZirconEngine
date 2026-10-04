//! 模板节点专用链在 status/viewport 等控件之后尝试本标题；即使几何不可画也认领已识别语义，避免 fallback 画通用面板。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_section_title_glyphs::{push_section_icon, section_title_icon};
use super::geometry::{frame_is_within, has_paintable_section_title_extent, section_icon_rect};
use super::identity::is_workbench_section_title;
use super::surface::push_section_title_surface;
use super::text::push_section_label;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

const SECTION_TITLE_COMMAND_CAPACITY: usize = 4;

/// 返回是否已认领节点。只在有效可见矩形内追加命令，图标缺席时文字回收其左侧空间。
/// 背景、图标和粗体叠字占用固定相对层级；调用方提供足够安全的基础 order。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_title_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_section_title(node) {
        return false;
    }
    if !has_paintable_section_title_extent(rect) || intersect(rect, clip).is_none() {
        return true;
    }

    commands.reserve(SECTION_TITLE_COMMAND_CAPACITY);
    push_section_title_surface(commands, rect, clip, order, opacity);
    let icon = section_title_icon(node);
    let icon_painted = if let Some(icon) = icon {
        let icon_rect = section_icon_rect(rect);
        if frame_is_within(rect, &icon_rect) && intersect(&icon_rect, clip).is_some() {
            push_section_icon(commands, &icon_rect, clip, order + 2, icon, opacity);
            true
        } else {
            false
        }
    } else {
        false
    };
    push_section_label(commands, node, rect, clip, order + 3, icon_painted, opacity);
    true
}

#[cfg(test)]
#[path = "tests/commands_optimization_batch_20260830cu_editor_tests.rs"]
mod optimization_batch_20260830cu_editor_tests;
