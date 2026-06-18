use std::f32::consts::{PI, TAU};

use super::super::super::super::paint_theme::PALETTE;

pub(super) struct ChartRaster {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
}

impl ChartRaster {
    pub(super) fn transparent(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rgba: vec![0; width as usize * height as usize * 4],
        }
    }

    pub(super) fn center(&self) -> (f32, f32) {
        (self.width as f32 * 0.5, self.height as f32 * 0.5)
    }

    pub(super) fn draw_polyline(&mut self, points: &[(f32, f32)], width: f32, color: [u8; 4]) {
        for pair in points.windows(2) {
            let start = self.normalized_point(pair[0]);
            let end = self.normalized_point(pair[1]);
            self.draw_line(start, end, width, color);
        }
    }

    pub(super) fn draw_points(&mut self, points: &[(f32, f32)], radius: f32, color: [u8; 4]) {
        for point in points {
            self.draw_disc(self.normalized_point(*point), radius, color);
        }
    }

    fn normalized_point(&self, point: (f32, f32)) -> (f32, f32) {
        (
            point.0.clamp(0.0, 1.0) * (self.width.saturating_sub(1)) as f32,
            point.1.clamp(0.0, 1.0) * (self.height.saturating_sub(1)) as f32,
        )
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

    pub(super) fn draw_disc(&mut self, center: (f32, f32), radius: f32, color: [u8; 4]) {
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

    pub(super) fn draw_arc(
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

    pub(super) fn draw_pie(&mut self, center: (f32, f32), radius: f32, hole_radius: f32) {
        let radius_sq = radius * radius;
        let hole_sq = hole_radius * hole_radius;
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
                    PALETTE.accent
                } else if progress < 0.76 {
                    PALETTE.success
                } else {
                    PALETTE.warning
                };
                self.set_pixel(x, y, color);
            }
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let offset = ((y as usize * self.width as usize) + x as usize) * 4;
        self.rgba[offset..offset + 4].copy_from_slice(&color);
    }
}

fn clamp_pixel_range(min: f32, max: f32, extent: u32) -> std::ops::Range<u32> {
    let start = min.floor().max(0.0).min(extent as f32) as u32;
    let end = max.ceil().max(0.0).min(extent as f32) as u32;
    start..end
}

fn distance_to_segment(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let segment = (end.0 - start.0, end.1 - start.1);
    let length_sq = segment.0 * segment.0 + segment.1 * segment.1;
    if length_sq <= f32::EPSILON {
        let dx = point.0 - start.0;
        let dy = point.1 - start.1;
        return (dx * dx + dy * dy).sqrt();
    }
    let t = (((point.0 - start.0) * segment.0 + (point.1 - start.1) * segment.1) / length_sq)
        .clamp(0.0, 1.0);
    let projection = (start.0 + segment.0 * t, start.1 + segment.1 * t);
    let dx = point.0 - projection.0;
    let dy = point.1 - projection.1;
    (dx * dx + dy * dy).sqrt()
}

fn normalized_angle(angle: f32) -> f32 {
    angle.rem_euclid(TAU)
}
