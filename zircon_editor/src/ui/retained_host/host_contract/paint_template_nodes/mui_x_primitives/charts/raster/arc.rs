use super::math::{clamp_pixel_range, normalized_angle};
use super::model::ChartRaster;

impl ChartRaster {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_arc(
        &mut self,
        center: (f32, f32),
        radius: f32,
        thickness: f32,
        start_angle: f32,
        end_angle: f32,
        color: [u8; 4],
    ) {
        let inner = (radius - thickness * 0.5).max(0.0);
        let outer = radius + thickness * 0.5;
        for y in clamp_pixel_range(center.1 - outer, center.1 + outer, self.height) {
            for x in clamp_pixel_range(center.0 - outer, center.0 + outer, self.width) {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = normalized_angle(dy.atan2(dx));
                if distance >= inner
                    && distance <= outer
                    && angle >= start_angle
                    && angle <= end_angle
                {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }
}
