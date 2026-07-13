use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::{SELECTED_SOURCE, SOURCE_MARKER};

pub(super) fn push_heat_source_markers(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for row in 0..node.weight_heatmap.sources.row_count() {
        let Some(source) = node.weight_heatmap.sources.row_data(row) else {
            continue;
        };
        let x = geometry.x_for_normalized(source.x);
        let y = geometry.y_for_normalized(source.y);
        push_diamond(
            commands,
            x,
            y,
            if source.selected { 5 } else { 3 },
            if source.selected {
                SELECTED_SOURCE
            } else {
                SOURCE_MARKER
            },
            clip,
            order + 4,
            opacity,
        );
    }
}

fn push_diamond(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    radius: i32,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for offset in -radius..=radius {
        let half_width = radius - offset.abs();
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: x - half_width as f32,
                y: y + offset as f32,
                width: (half_width * 2 + 1) as f32,
                height: 1.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
