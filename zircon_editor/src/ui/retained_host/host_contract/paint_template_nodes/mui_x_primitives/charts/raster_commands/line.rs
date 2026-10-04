use super::super::raster::ChartRaster;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

const MUI_X_CHART_LINE_WIDTH: f32 = 2.4;
const MUI_X_CHART_SECONDARY_LINE_WIDTH_RATIO: f32 = 0.72;
const MUI_X_CHART_POINT_RADIUS: f32 = 2.2;
const MUI_X_CHART_POINT_RATIO_UNITS: f32 = 100.0;
type LineChartColors = [[u8; 4]; 2];

#[derive(Clone, Copy)]
struct LineChartPointSpec {
    x_units: u8,
    y_units: u8,
}

impl LineChartPointSpec {
    const fn new(x_units: u8, y_units: u8) -> Self {
        Self { x_units, y_units }
    }
}

const PRIMARY_LINE_POINTS: [LineChartPointSpec; 5] = [
    LineChartPointSpec::new(8, 78),
    LineChartPointSpec::new(30, 38),
    LineChartPointSpec::new(52, 52),
    LineChartPointSpec::new(75, 24),
    LineChartPointSpec::new(92, 44),
];

const SECONDARY_LINE_POINTS: [LineChartPointSpec; 4] = [
    LineChartPointSpec::new(10, 56),
    LineChartPointSpec::new(34, 62),
    LineChartPointSpec::new(56, 42),
    LineChartPointSpec::new(80, 50),
];

/// 折线和采样点共用位图坐标系；宿主色板变化由上层缓存身份触发重绘。
pub(super) fn draw_line_chart_raster(raster: &mut ChartRaster) {
    let [primary_line, secondary_line] = line_chart_colors_from_host(current_host_palette());
    let points = line_chart_points(PRIMARY_LINE_POINTS);
    let secondary_points = line_chart_points(SECONDARY_LINE_POINTS);
    raster.draw_polyline(&points, MUI_X_CHART_LINE_WIDTH, primary_line);
    raster.draw_polyline(
        &secondary_points,
        MUI_X_CHART_LINE_WIDTH * MUI_X_CHART_SECONDARY_LINE_WIDTH_RATIO,
        secondary_line,
    );
    raster.draw_points(&points, MUI_X_CHART_POINT_RADIUS, primary_line);
}

fn line_chart_colors_from_host(palette: HostMaterialPalette) -> LineChartColors {
    [palette.accent, palette.success]
}

fn line_chart_points<const N: usize>(points: [LineChartPointSpec; N]) -> [(f32, f32); N] {
    points.map(line_chart_point)
}

fn line_chart_point(point: LineChartPointSpec) -> (f32, f32) {
    (
        f32::from(point.x_units) / MUI_X_CHART_POINT_RATIO_UNITS,
        f32::from(point.y_units) / MUI_X_CHART_POINT_RATIO_UNITS,
    )
}

#[cfg(test)]
#[path = "tests/line.rs"]
mod tests;
