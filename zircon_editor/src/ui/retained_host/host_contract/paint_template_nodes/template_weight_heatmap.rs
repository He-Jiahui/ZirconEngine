mod field;
mod geometry;
mod identity;
mod markers;
mod palette;
mod text;

use super::super::data::{FrameRect, TemplatePaneNodeData, TemplatePaneWeightHeatmapSourceData};
use super::render_commands::HostPaintCommand;
use field::push_heatmap_field;
use geometry::WeightHeatmapGeometry;
use identity::is_weight_heatmap;
use markers::push_heat_source_markers;
use text::push_heatmap_legend_text;

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

    let geometry = WeightHeatmapGeometry::from_frame(rect);
    let sources = heatmap_sources(node);
    push_heatmap_field(commands, node, &sources, &geometry, clip, order, opacity);
    push_heat_source_markers(commands, &sources, &geometry, clip, order, opacity);
    push_heatmap_legend_text(commands, node, &geometry, clip, order, opacity);
    true
}

fn heatmap_sources(node: &TemplatePaneNodeData) -> Vec<TemplatePaneWeightHeatmapSourceData> {
    (0..node.weight_heatmap.sources.row_count())
        .filter_map(|row| node.weight_heatmap.sources.row_data(row))
        .collect()
}

#[cfg(test)]
#[path = "template_weight_heatmap_tests/mod.rs"]
mod tests;
