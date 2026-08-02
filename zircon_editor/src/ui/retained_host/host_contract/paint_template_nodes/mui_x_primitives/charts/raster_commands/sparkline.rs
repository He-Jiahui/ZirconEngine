use super::super::raster::ChartRaster;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

const MUI_X_SPARKLINE_WIDTH: f32 = 2.0;

pub(super) fn draw_sparkline_raster(raster: &mut ChartRaster) {
    let mark_color = sparkline_color_from_host(current_host_palette());
    let points = [
        (0.06, 0.72),
        (0.24, 0.38),
        (0.44, 0.58),
        (0.65, 0.31),
        (0.86, 0.46),
    ];
    raster.draw_polyline(&points, MUI_X_SPARKLINE_WIDTH, mark_color);
    raster.draw_points(&points, 1.9, mark_color);
}

fn sparkline_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.accent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_sparkline_color_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];

        assert_eq!(sparkline_color_from_host(palette), [10, 11, 12, 255]);
    }
}
