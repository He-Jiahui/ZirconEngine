use std::f32::consts::{PI, TAU};

use super::math::{clamp_pixel_range, normalized_angle};
use super::model::ChartRaster;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

type PieSliceColors = [[u8; 4]; 3];

impl ChartRaster {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_pie(
        &mut self,
        center: (f32, f32),
        radius: f32,
        hole_radius: f32,
    ) {
        let radius_sq = radius * radius;
        let hole_sq = hole_radius * hole_radius;
        let [accent_slice, success_slice, warning_slice] =
            pie_slice_colors_from_host(current_host_palette());
        for y in clamp_pixel_range(center.1 - radius, center.1 + radius, self.height) {
            for x in clamp_pixel_range(center.0 - radius, center.0 + radius, self.width) {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let distance_sq = dx * dx + dy * dy;
                if distance_sq > radius_sq || distance_sq < hole_sq {
                    continue;
                }
                let progress = (normalized_angle(dy.atan2(dx)) + PI * 0.5).rem_euclid(TAU) / TAU;
                let color = if progress < 0.42 {
                    accent_slice
                } else if progress < 0.76 {
                    success_slice
                } else {
                    warning_slice
                };
                self.set_pixel(x, y, color);
            }
        }
    }
}

fn pie_slice_colors_from_host(palette: HostMaterialPalette) -> PieSliceColors {
    [palette.accent, palette.success, palette.warning]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_pie_slice_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];
        palette.success = [20, 21, 22, 255];
        palette.warning = [30, 31, 32, 255];

        assert_eq!(
            pie_slice_colors_from_host(palette),
            [[10, 11, 12, 255], [20, 21, 22, 255], [30, 31, 32, 255]]
        );
    }
}
