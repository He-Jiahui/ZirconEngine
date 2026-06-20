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
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                if dx * dx + dy * dy <= radius_sq {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }
}
