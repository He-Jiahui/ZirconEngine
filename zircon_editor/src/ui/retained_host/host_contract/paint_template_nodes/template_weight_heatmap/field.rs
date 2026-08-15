use crate::ui::weight_heatmap::WeightHeatmapGeneration;

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::{heat_color, OUTER_BORDER, OUTER_SURFACE};

const MAX_HEATMAP_LEGEND_STEPS: usize = 64;

pub(super) fn push_heatmap_field(
    commands: &mut Vec<HostPaintCommand>,
    generation: &WeightHeatmapGeneration,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !geometry.is_drawable() {
        return;
    }
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

    let field = generation.static_field_for_plot_size(geometry.plot.width, geometry.plot.height);
    let cell_width = geometry.plot.width / field.columns() as f32;
    let cell_height = geometry.plot.height / field.rows() as f32;
    for row in 0..field.rows() {
        for column in 0..field.columns() {
            commands.push(HostPaintCommand::quad(
                FrameRect {
                    x: geometry.plot.x + column as f32 * cell_width,
                    y: geometry.plot.y + row as f32 * cell_height,
                    width: cell_width + 0.35,
                    height: cell_height + 0.35,
                },
                Some(clip.clone()),
                order + 1,
                Some(heat_color(field.intensity_at(row, column))),
                None,
                0.0,
                0.0,
                opacity,
            ));
        }
    }

    let legend_steps = bounded_legend_steps(field.rows(), geometry.legend.height);
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

fn bounded_legend_steps(rows: usize, legend_height: f32) -> usize {
    rows.max(12)
        .min(MAX_HEATMAP_LEGEND_STEPS)
        .min(pixel_axis_budget(legend_height))
        .max(1)
}

fn pixel_axis_budget(extent: f32) -> usize {
    if extent.is_finite() && extent > 0.0 {
        (extent.ceil() as usize).max(1)
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legend_steps_are_bounded_by_pixels_and_constant_budget() {
        assert_eq!(bounded_legend_steps(10, 100.0), 12);
        assert_eq!(bounded_legend_steps(usize::MAX, 10_000.0), 64);
        assert_eq!(bounded_legend_steps(usize::MAX, 3.0), 3);
    }
}
