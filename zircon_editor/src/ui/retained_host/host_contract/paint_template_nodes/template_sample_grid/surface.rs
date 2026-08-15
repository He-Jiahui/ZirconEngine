use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::SampleGridGeometry;
use super::metrics::{SampleGridMetrics, GRID_DASH_GAP, GRID_DASH_LENGTH};
use super::palette::SampleGridPalette;

pub(super) fn push_sample_grid_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &SampleGridGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: SampleGridMetrics,
    palette: SampleGridPalette,
) {
    let grid = &node.sample_grid.generation;
    commands.push(HostPaintCommand::quad(
        geometry.outer.clone(),
        Some(clip.clone()),
        order,
        Some(palette.outer_surface),
        Some(palette.outer_border),
        metrics.border_width,
        metrics.outer_radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        geometry.plot.clone(),
        Some(clip.clone()),
        order + 1,
        Some(palette.plot_surface),
        Some(palette.plot_border),
        metrics.border_width,
        metrics.plot_radius,
        opacity,
    ));

    for tick in grid.x_ticks() {
        let x = geometry.x_for_value(tick.value(), grid.x_min(), grid.x_max());
        push_dashed_vertical(
            commands,
            x,
            &geometry.plot,
            clip,
            order + 2,
            opacity,
            metrics,
            palette,
        );
    }
    for tick in grid.y_ticks() {
        let y = geometry.y_for_value(tick.value(), grid.y_min(), grid.y_max());
        push_dashed_horizontal(
            commands,
            y,
            &geometry.plot,
            clip,
            order + 2,
            opacity,
            metrics,
            palette,
        );
    }

    if grid.x_min() <= 0.0 && grid.x_max() >= 0.0 {
        let x = geometry.x_for_value(0.0, grid.x_min(), grid.x_max());
        push_line(
            commands,
            FrameRect {
                x,
                y: geometry.plot.y,
                width: metrics.grid_line_width,
                height: geometry.plot.height,
            },
            clip,
            order + 3,
            palette.zero_axis,
            opacity,
        );
    }
    if grid.y_min() <= 0.0 && grid.y_max() >= 0.0 {
        let y = geometry.y_for_value(0.0, grid.y_min(), grid.y_max());
        push_line(
            commands,
            FrameRect {
                x: geometry.plot.x,
                y,
                width: geometry.plot.width,
                height: metrics.grid_line_width,
            },
            clip,
            order + 3,
            palette.zero_axis,
            opacity,
        );
    }
}

fn push_dashed_vertical(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: SampleGridMetrics,
    palette: SampleGridPalette,
) {
    let mut y = plot.y;
    while y < plot.y + plot.height {
        push_line(
            commands,
            FrameRect {
                x,
                y,
                width: metrics.grid_line_width,
                height: GRID_DASH_LENGTH.min(plot.y + plot.height - y),
            },
            clip,
            order,
            palette.grid_line,
            opacity,
        );
        y += GRID_DASH_LENGTH + GRID_DASH_GAP;
    }
}

fn push_dashed_horizontal(
    commands: &mut Vec<HostPaintCommand>,
    y: f32,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: SampleGridMetrics,
    palette: SampleGridPalette,
) {
    let mut x = plot.x;
    while x < plot.x + plot.width {
        push_line(
            commands,
            FrameRect {
                x,
                y,
                width: GRID_DASH_LENGTH.min(plot.x + plot.width - x),
                height: metrics.grid_line_width,
            },
            clip,
            order,
            palette.grid_line,
            opacity,
        );
        x += GRID_DASH_LENGTH + GRID_DASH_GAP;
    }
}

fn push_line(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
