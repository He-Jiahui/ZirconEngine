use super::super::raster::ChartRaster;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(super) fn draw_pie_chart_raster(raster: &mut ChartRaster, node: &TemplatePaneNodeData) {
    let center = raster.center();
    let radius = raster.width.min(raster.height) as f32 * 0.43;
    let hole_radius = if node.selected || node.checked {
        radius * 0.34
    } else {
        0.0
    };
    raster.draw_pie(center, radius, hole_radius);
}
