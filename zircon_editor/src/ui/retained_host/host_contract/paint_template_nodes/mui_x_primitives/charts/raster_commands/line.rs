use super::super::raster::ChartRaster;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

const MUI_X_CHART_LINE_WIDTH: f32 = 2.4;

pub(super) fn draw_line_chart_raster(raster: &mut ChartRaster) {
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
