use std::f32::consts::{PI, TAU};

use super::super::raster::ChartRaster;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(super) fn draw_gauge_raster(raster: &mut ChartRaster, value: f32) {
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

pub(super) fn chart_value(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent > 0.0 {
        node.value_percent
    } else if node.value_number > 1.0 {
        node.value_number / 100.0
    } else {
        node.value_number
    }
}
