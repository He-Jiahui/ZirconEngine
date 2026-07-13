use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::LEGEND_TEXT;

pub(super) fn push_heatmap_legend_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_label(
        commands,
        node.weight_heatmap.high_label.to_string(),
        geometry.legend.x + geometry.legend.width + 2.0,
        geometry.legend.y - 1.0,
        clip,
        order + 5,
        opacity,
    );
    push_label(
        commands,
        node.weight_heatmap.low_label.to_string(),
        geometry.legend.x + geometry.legend.width + 2.0,
        geometry.legend.y + geometry.legend.height - 11.0,
        clip,
        order + 5,
        opacity,
    );
}

fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    text: String,
    x: f32,
    y: f32,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if text.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x,
            y,
            width: 18.0,
            height: 11.0,
        },
        Some(clip.clone()),
        order,
        text,
        LEGEND_TEXT,
        8.0,
        10.0,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
