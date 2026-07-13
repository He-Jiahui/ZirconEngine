use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::{heat_color, OUTER_BORDER, OUTER_SURFACE};

pub(super) fn push_heatmap_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        geometry.outer.clone(),
        Some(clip.clone()),
        order,
        Some(OUTER_SURFACE),
        Some(OUTER_BORDER),
        1.0,
        2.0,
        opacity,
    ));

    let columns = node.weight_heatmap.columns.max(1) as usize;
    let rows = node.weight_heatmap.rows.max(1) as usize;
    let cell_width = geometry.plot.width / columns as f32;
    let cell_height = geometry.plot.height / rows as f32;
    for row in 0..rows {
        for column in 0..columns {
            let x = (column as f32 + 0.5) / columns as f32;
            let y = 1.0 - (row as f32 + 0.5) / rows as f32;
            let intensity = heat_intensity(node, x, y);
            commands.push(HostPaintCommand::quad(
                FrameRect {
                    x: geometry.plot.x + column as f32 * cell_width,
                    y: geometry.plot.y + row as f32 * cell_height,
                    width: cell_width + 0.35,
                    height: cell_height + 0.35,
                },
                Some(clip.clone()),
                order + 1,
                Some(heat_color(intensity)),
                None,
                0.0,
                0.0,
                opacity,
            ));
        }
    }

    let legend_steps = rows.max(12);
    let legend_height = geometry.legend.height / legend_steps as f32;
    for step in 0..legend_steps {
        let intensity = 1.0 - (step as f32 + 0.5) / legend_steps as f32;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: geometry.legend.x,
                y: geometry.legend.y + step as f32 * legend_height,
                width: geometry.legend.width,
                height: legend_height + 0.35,
            },
            Some(clip.clone()),
            order + 2,
            Some(heat_color(intensity)),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn heat_intensity(node: &TemplatePaneNodeData, x: f32, y: f32) -> f32 {
    let mut intensity = 0.0f32;
    for row in 0..node.weight_heatmap.sources.row_count() {
        let Some(source) = node.weight_heatmap.sources.row_data(row) else {
            continue;
        };
        let dx = x - source.x;
        let dy = y - source.y;
        let influence = source.weight * (-8.0 * (dx * dx + dy * dy)).exp();
        intensity = intensity.max(influence);
    }
    intensity.clamp(0.0, 1.0)
}
