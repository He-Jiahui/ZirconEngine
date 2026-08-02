use std::f32::consts::{PI, TAU};

use super::super::raster::ChartRaster;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

type GaugeColors = [[u8; 4]; 2];

pub(super) fn draw_gauge_raster(raster: &mut ChartRaster, value: f32) {
    let [track_color, value_color] = gauge_colors_from_host(current_host_palette());
    let center = (raster.width as f32 * 0.5, raster.height as f32 - 3.0);
    let radius = (raster.height as f32 - 7.0).max(4.0);
    let thickness = (raster.height as f32 * 0.12).clamp(2.0, 4.0);
    let start = PI;
    let end = TAU;
    raster.draw_arc(center, radius, thickness, start, end, track_color);
    raster.draw_arc(
        center,
        radius,
        thickness,
        start,
        start + (end - start) * value.clamp(0.0, 1.0),
        value_color,
    );
    raster.draw_disc(center, thickness * 1.35, track_color);
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

fn gauge_colors_from_host(palette: HostMaterialPalette) -> GaugeColors {
    [palette.surface_hover, palette.accent]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_gauge_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_hover = [10, 11, 12, 255];
        palette.accent = [20, 21, 22, 255];

        assert_eq!(
            gauge_colors_from_host(palette),
            [[10, 11, 12, 255], [20, 21, 22, 255]]
        );
    }
}
