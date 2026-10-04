// 热力图专用入口消费已归一化的 source generation，先布局场与图例再画静态色块、动态标记及借用标签。
mod field;
mod geometry;
mod identity;
mod markers;
mod palette;
mod text;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use field::push_heatmap_field;
use geometry::{has_paintable_weight_heatmap_extent, WeightHeatmapGeometry};
use identity::is_weight_heatmap;
use markers::push_heat_source_markers;
use text::{legend_label_width, push_heatmap_legend_text};

/// secondary specialized 链仅在 canvas+weight-heatmap 上认领；无效外框也需认领以避免通用表面重画。
/// generation 由 typed projection 提供；字段缓存依赖静态代次及 plot 尺寸，选中 marker 则按当前 source 状态绘制。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_weight_heatmap_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_weight_heatmap(node) {
        return false;
    }
    if !has_paintable_weight_heatmap_extent(rect) {
        return true;
    }

    let generation = &node.weight_heatmap.generation;
    let geometry = WeightHeatmapGeometry::from_frame(rect, legend_label_width(generation));
    push_heatmap_field(commands, generation, &geometry, clip, order, opacity);
    push_heat_source_markers(
        commands,
        generation.sources(),
        &geometry,
        clip,
        order,
        opacity,
    );
    push_heatmap_legend_text(commands, generation, &geometry, clip, order, opacity);
    true
}

#[cfg(test)]
#[path = "template_weight_heatmap_tests/tests/mod.rs"]
mod tests;
