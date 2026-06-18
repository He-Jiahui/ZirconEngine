use std::f32::consts::{PI, TAU};

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::raster::ChartRaster;
use super::ChartKind;

const MUI_X_CHART_MAX_RASTER_EXTENT: f32 = 192.0;
const MUI_X_CHART_LINE_WIDTH: f32 = 2.4;
const MUI_X_SPARKLINE_WIDTH: f32 = 2.0;

pub(super) fn push_chart_raster(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChartKind,
) {
    let Some((width, height)) = chart_raster_dimensions(plot) else {
        return;
    };
    let mut raster = ChartRaster::transparent(width, height);
    match kind {
        ChartKind::Line => draw_line_chart_raster(&mut raster),
        ChartKind::Pie => draw_pie_chart_raster(&mut raster, node),
        ChartKind::Sparkline => draw_sparkline_raster(&mut raster),
        ChartKind::Gauge => draw_gauge_raster(&mut raster, chart_value(node)),
        ChartKind::Aggregate | ChartKind::Bar => return,
    }
    commands.push(HostPaintCommand::image_pixels(
        plot.clone(),
        Some(clip.clone()),
        order,
        format!("mui-x-chart:{}:{}x{}", chart_kind_name(kind), width, height),
        width,
        height,
        raster.rgba,
        None,
        opacity,
    ));
}

fn chart_raster_dimensions(plot: &FrameRect) -> Option<(u32, u32)> {
    if plot.width <= 0.0 || plot.height <= 0.0 {
        return None;
    }
    Some((
        plot.width.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
        plot.height.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
    ))
}

fn chart_kind_name(kind: ChartKind) -> &'static str {
    match kind {
        ChartKind::Aggregate => "aggregate",
        ChartKind::Line => "line",
        ChartKind::Bar => "bar",
        ChartKind::Pie => "pie",
        ChartKind::Sparkline => "sparkline",
        ChartKind::Gauge => "gauge",
    }
}

fn draw_line_chart_raster(raster: &mut ChartRaster) {
    let points = [
        (0.08, 0.78),
        (0.30, 0.38),
        (0.52, 0.52),
        (0.75, 0.24),
        (0.92, 0.44),
    ];
    raster.draw_polyline(&points, MUI_X_CHART_LINE_WIDTH, PALETTE.accent);
    raster.draw_polyline(
        &[(0.10, 0.56), (0.34, 0.62), (0.56, 0.42), (0.80, 0.50)],
        MUI_X_CHART_LINE_WIDTH * 0.72,
        PALETTE.success,
    );
    raster.draw_points(&points, 2.2, PALETTE.accent);
}

fn draw_sparkline_raster(raster: &mut ChartRaster) {
    let points = [
        (0.06, 0.72),
        (0.24, 0.38),
        (0.44, 0.58),
        (0.65, 0.31),
        (0.86, 0.46),
    ];
    raster.draw_polyline(&points, MUI_X_SPARKLINE_WIDTH, PALETTE.accent);
    raster.draw_points(&points, 1.9, PALETTE.accent);
}

fn draw_pie_chart_raster(raster: &mut ChartRaster, node: &TemplatePaneNodeData) {
    let center = raster.center();
    let radius = raster.width.min(raster.height) as f32 * 0.43;
    let hole_radius = if node.selected || node.checked {
        radius * 0.34
    } else {
        0.0
    };
    raster.draw_pie(center, radius, hole_radius);
}

fn draw_gauge_raster(raster: &mut ChartRaster, value: f32) {
    let center = (raster.width as f32 * 0.5, raster.height as f32 - 3.0);
    let radius = (raster.height as f32 - 7.0).max(4.0);
    let thickness = (raster.height as f32 * 0.12).clamp(2.0, 4.0);
    let start = PI;
    let end = TAU;
    raster.draw_arc(center, radius, thickness, start, end, PALETTE.surface_hover);
    raster.draw_arc(
        center,
        radius,
        thickness,
        start,
        start + (end - start) * value.clamp(0.0, 1.0),
        PALETTE.accent,
    );
    raster.draw_disc(center, thickness * 1.35, PALETTE.surface_hover);
}

fn chart_value(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent > 0.0 {
        node.value_percent
    } else if node.value_number > 1.0 {
        node.value_number / 100.0
    } else {
        node.value_number
    }
}
