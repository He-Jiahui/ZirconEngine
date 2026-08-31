use super::math::clamp_pixel_range;
use super::model::ChartRaster;

impl ChartRaster {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_points(
        &mut self,
        points: &[(f32, f32)],
        radius: f32,
        color: [u8; 4],
    ) {
        for point in points {
            self.draw_disc(self.normalized_point(*point), radius, color);
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_disc(
        &mut self,
        center: (f32, f32),
        radius: f32,
        color: [u8; 4],
    ) {
        let radius_sq = radius * radius;
        for y in clamp_pixel_range(center.1 - radius, center.1 + radius, self.height) {
            for x in clamp_pixel_range(center.0 - radius, center.0 + radius, self.width) {
                self.sample_pixel(x, y, |px, py| {
                    let dx = px - center.0;
                    let dy = py - center.1;
                    (dx * dx + dy * dy <= radius_sq).then_some(color)
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChartRaster;

    #[test]
    fn disc_edges_resolve_to_fractional_alpha_without_losing_the_opaque_center() {
        let mut raster = ChartRaster::transparent(8, 8);

        raster.draw_disc((4.0, 4.0), 3.5, [80, 160, 240, 255]);

        let alpha = raster
            .rgba
            .chunks_exact(4)
            .map(|pixel| pixel[3])
            .collect::<Vec<_>>();
        assert!(alpha.contains(&0));
        assert!(alpha.contains(&255));
        assert!(alpha.iter().any(|value| *value > 0 && *value < 255));
    }
}
