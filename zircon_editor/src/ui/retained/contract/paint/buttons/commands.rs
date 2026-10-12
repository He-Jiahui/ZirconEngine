//! primary 专用链的普通按钮入口；接管与实际可见性分开，避免无空间时又被通用表面和文本绘制。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::content::push_button_content;
use super::geometry::{button_paint_rect, has_paintable_button_extent};
use super::identity::{button_kind, is_workbench_button};
use super::layers::content_order;
use super::style::button_opacity;
use super::surface::push_button_surface;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

/// 返回是否接管节点；true 可以不追加命令。调用方传入最终布局框、祖先裁剪与基础层级，不应据此判断资产或内容是否成功显示。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_button(node) {
        return false;
    }
    if !has_paintable_button_extent(rect) {
        return true;
    }
    let rect = button_paint_rect(node, rect);
    if intersect(&rect, clip).is_none() {
        return true;
    }

    let kind = button_kind(node);
    let opacity = button_opacity(node, opacity);
    push_button_surface(commands, node, &rect, clip, order, kind, opacity);
    push_button_content(
        commands,
        node,
        &rect,
        clip,
        content_order(order),
        kind,
        opacity,
    );
    true
}
