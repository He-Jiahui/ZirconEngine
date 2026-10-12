//! tooltip 布局先求内容驱动的气泡，再按固定层级追加阴影、文字、箭头与可选 info 标记。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_tooltip_style;
use super::super::template_tooltip_glyphs::{
    push_tooltip_arrow, push_tooltip_info_icon, tooltip_arrow_size, tooltip_icon_size,
};
use super::identity::is_workbench_tooltip;
use super::layers::{arrow_order, icon_order, text_order};
use super::layout::{
    frame_is_within, has_paintable_tooltip_extent, paint_rect, tooltip_bubble_rect,
};
use super::surface::push_tooltip_surface;
use super::text::push_tooltip_text;

/// 返回是否已认领工作台 tooltip；无可画几何时仍禁止通用 fallback 重画它。
/// 传入的 rect 是完整浮层容器，不只是气泡，阴影/箭头/图标需要使用其中的额外空间。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_tooltip(node) {
        return false;
    }

    let rect = paint_rect(rect);
    if !has_paintable_tooltip_extent(&rect) {
        return true;
    }

    let style = select_workbench_tooltip_style(node);
    let bubble = tooltip_bubble_rect(node, &rect);
    if !frame_is_within(&rect, &bubble) {
        return true;
    }
    let arrow_size = tooltip_arrow_size(node);

    push_tooltip_surface(
        commands,
        node,
        &rect,
        &bubble,
        clip,
        order,
        style.shadow,
        style.surface,
        style.border,
        opacity,
    );
    push_tooltip_text(
        commands,
        node,
        &bubble,
        clip,
        text_order(order),
        style.title,
        style.body,
        opacity,
    );
    push_tooltip_arrow(
        commands,
        &rect,
        &bubble,
        clip,
        arrow_order(order),
        arrow_size,
        style.arrow,
        style.border,
        opacity,
    );
    if !node.icon_name.is_empty() {
        push_tooltip_info_icon(
            commands,
            node,
            &rect,
            clip,
            icon_order(order),
            tooltip_icon_size(node),
            style.icon,
            opacity,
        );
    }

    true
}
