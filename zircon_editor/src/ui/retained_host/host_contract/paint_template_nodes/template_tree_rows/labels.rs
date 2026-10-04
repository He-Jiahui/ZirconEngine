//! 树行对象图标之后的标题区保留右侧操作空间；空标题或无法完整容纳的文本区不产生命令。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_tree_row_geometry::{
    tree_font_size, tree_label_rect_for_variant, tree_line_height,
};
use super::geometry::tree_row_contains;
use super::style::tree_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }

    let text_rect =
        tree_label_rect_for_variant(rect, icon, node.component_variant.as_str() == "content");
    if !tree_row_contains(rect, &text_rect) {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        tree_text_color(node),
        if node.font_size > 0.0 {
            super::super::super::paint_theme::logical_font_size_to_physical(
                node.font_size,
                super::super::super::paint_theme::current_host_metrics().scale_factor,
            )
        } else {
            tree_font_size()
        },
        if node.font_size > 0.0 {
            super::super::super::paint_theme::logical_font_size_to_physical(
                node.font_size,
                super::super::super::paint_theme::current_host_metrics().scale_factor,
            ) * super::super::super::paint_theme::current_host_metrics().line_height_ratio
        } else {
            tree_line_height()
        },
        UiTextRunPaintStyle {
            strong: node.font_weight >= 600,
            ..UiTextRunPaintStyle::default()
        },
        opacity,
    ));
}
