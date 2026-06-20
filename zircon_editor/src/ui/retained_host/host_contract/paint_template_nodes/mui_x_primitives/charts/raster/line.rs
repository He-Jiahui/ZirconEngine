use super::math::{clamp_pixel_range, distance_to_segment};
use super::model::ChartRaster;

impl ChartRaster {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_polyline(
        &mut self,
        points: &[(f32, f32)],
        width: f32,
        color: [u8; 4],
    ) {
        for pair in points.windows(2) {
            let start = self.normalized_point(pair[0]);
            let end = self.normalized_point(pair[1]);
            self.draw_line(start, end, width, color);
        }
    }

    fn draw_line(&mut self, start: (f32, f32), end: (f32, f32), width: f32, color: [u8; 4]) {
        let radius = (width * 0.5).max(0.5);
        let min_x = start.0.min(end.0) - radius;
        let max_x = start.0.max(end.0) + radius;
        let min_y = start.1.min(end.1) - radius;
        let max_y = start.1.max(end.1) + radius;
        for y in clamp_pixel_range(min_y, max_y, self.height) {
            for x in clamp_pixel_range(min_x, max_x, self.width) {
                let point = (x as f32 + 0.5, y as f32 + 0.5);
                if distance_to_segment(point, start, end) <= radius {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }
}
