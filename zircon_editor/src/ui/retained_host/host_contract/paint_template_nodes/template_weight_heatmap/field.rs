use super::super::super::data::{
    FrameRect, TemplatePaneNodeData, TemplatePaneWeightHeatmapSourceData,
};
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::{OUTER_BORDER, OUTER_SURFACE, heat_color};

const MAX_HEATMAP_CELLS: usize = 4_096;
const MAX_HEATMAP_INFLUENCE_EVALUATIONS: usize = 65_536;
const MAX_HEATMAP_LEGEND_STEPS: usize = 64;

pub(super) fn push_heatmap_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    sources: &[TemplatePaneWeightHeatmapSourceData],
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

    let (columns, rows) = bounded_grid_dimensions(
        node.weight_heatmap.columns,
        node.weight_heatmap.rows,
        geometry.plot.width,
        geometry.plot.height,
        heatmap_cell_budget(sources.len()),
    );
    let cell_width = geometry.plot.width / columns as f32;
    let cell_height = geometry.plot.height / rows as f32;
    for row in 0..rows {
        for column in 0..columns {
            let x = (column as f32 + 0.5) / columns as f32;
            let y = 1.0 - (row as f32 + 0.5) / rows as f32;
            let intensity = heat_intensity(sources, x, y);
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

    let legend_steps = bounded_legend_steps(rows, geometry.legend.height);
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

fn heatmap_cell_budget(source_count: usize) -> usize {
    if source_count == 0 {
        MAX_HEATMAP_CELLS
    } else {
        (MAX_HEATMAP_INFLUENCE_EVALUATIONS / source_count).clamp(1, MAX_HEATMAP_CELLS)
    }
}

fn bounded_grid_dimensions(
    requested_columns: i32,
    requested_rows: i32,
    plot_width: f32,
    plot_height: f32,
    max_cells: usize,
) -> (usize, usize) {
    let max_cells = max_cells.max(1);
    let mut columns = (requested_columns.max(1) as usize)
        .min(pixel_axis_budget(plot_width))
        .min(max_cells);
    let mut rows = (requested_rows.max(1) as usize)
        .min(pixel_axis_budget(plot_height))
        .min(max_cells);
    if columns.saturating_mul(rows) <= max_cells {
        return (columns, rows);
    }

    let square_root = (max_cells as f64).sqrt().floor() as usize;
    if columns <= square_root {
        rows = rows.min(max_cells / columns);
    } else if rows <= square_root {
        columns = columns.min(max_cells / rows);
    } else {
        let scale = (max_cells as f64 / (columns as f64 * rows as f64)).sqrt();
        columns = ((columns as f64 * scale).floor() as usize).max(1);
        rows = ((rows as f64 * scale).floor() as usize).max(1);
        rows = rows.min(max_cells / columns);
    }
    (columns.max(1), rows.max(1))
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

fn heat_intensity(sources: &[TemplatePaneWeightHeatmapSourceData], x: f32, y: f32) -> f32 {
    let mut intensity = 0.0f32;
    for source in sources {
        let dx = x - source.x;
        let dy = y - source.y;
        let influence = source.weight * (-8.0 * (dx * dx + dy * dy)).exp();
        intensity = intensity.max(influence);
    }
    intensity.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_dimensions_respect_cell_and_pixel_budgets() {
        let (columns, rows) =
            bounded_grid_dimensions(i32::MAX, i32::MAX, 5_000.0, 5_000.0, MAX_HEATMAP_CELLS);

        assert!(columns <= 5_000);
        assert!(rows <= 5_000);
        assert!(columns * rows <= MAX_HEATMAP_CELLS);
    }

    #[test]
    fn grid_dimensions_preserve_a_small_requested_axis() {
        let (columns, rows) =
            bounded_grid_dimensions(i32::MAX, 2, 10_000.0, 2.0, MAX_HEATMAP_CELLS);

        assert_eq!(rows, 2);
        assert_eq!(columns, MAX_HEATMAP_CELLS / rows);
    }

    #[test]
    fn grid_dimensions_normalize_non_positive_input() {
        assert_eq!(
            bounded_grid_dimensions(-4, 0, 100.0, 100.0, MAX_HEATMAP_CELLS),
            (1, 1)
        );
    }

    #[test]
    fn cell_budget_adapts_to_source_count() {
        assert_eq!(heatmap_cell_budget(0), MAX_HEATMAP_CELLS);
        assert_eq!(heatmap_cell_budget(64), 1_024);
        assert_eq!(heatmap_cell_budget(usize::MAX), 1);
    }

    #[test]
    fn legend_steps_are_bounded_by_pixels_and_constant_budget() {
        assert_eq!(bounded_legend_steps(10, 100.0), 12);
        assert_eq!(bounded_legend_steps(usize::MAX, 10_000.0), 64);
        assert_eq!(bounded_legend_steps(usize::MAX, 3.0), 3);
    }

    #[test]
    fn heat_intensity_keeps_every_materialized_source() {
        let sources = [
            TemplatePaneWeightHeatmapSourceData {
                x: 0.0,
                y: 0.0,
                weight: 0.1,
                selected: false,
            },
            TemplatePaneWeightHeatmapSourceData {
                x: 0.5,
                y: 0.5,
                weight: 1.0,
                selected: true,
            },
        ];

        assert_eq!(heat_intensity(&sources, 0.5, 0.5), 1.0);
    }
}
