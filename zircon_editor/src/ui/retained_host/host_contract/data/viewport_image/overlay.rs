use std::sync::Arc;

use zircon_runtime_interface::math::{UVec2, Vec2, Vec4};

use crate::scene::viewport::HandleScreenLine;
use crate::ui::retained_host::host_contract::paint_color::blend_srgb_pixel_linear;

const MAX_LINE_WIDTH_PX: f32 = 8.0;
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Clone)]
pub(crate) struct HostViewportOverlayImageData {
    pub(crate) resource_key: String,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Arc<[u8]>,
}

impl HostViewportOverlayImageData {
    pub(crate) fn from_screen_lines(
        resource_scope: &str,
        source_size: UVec2,
        lines: &[HandleScreenLine],
    ) -> Option<Self> {
        if resource_scope.is_empty() || source_size.x == 0 || source_size.y == 0 {
            return None;
        }
        let clipped = lines
            .iter()
            .copied()
            .filter_map(|line| RasterLine::clipped(line, source_size))
            .collect::<Vec<_>>();
        let bounds = RasterBounds::from_lines(&clipped, source_size)?;
        let byte_len = bounds
            .width()
            .checked_mul(bounds.height())?
            .checked_mul(4)? as usize;
        let mut rgba = vec![0; byte_len];
        for line in clipped {
            raster_line(&mut rgba, bounds, line);
        }
        if rgba.chunks_exact(4).all(|pixel| pixel[3] == 0) {
            return None;
        }
        let content_hash = fnv1a(&rgba);
        Some(Self {
            resource_key: format!(
                "viewport-overlay:{}:{}:{}:{}:{content_hash:016x}:{resource_scope}",
                bounds.left,
                bounds.top,
                bounds.width(),
                bounds.height()
            ),
            x: bounds.left,
            y: bounds.top,
            width: bounds.width(),
            height: bounds.height(),
            rgba: rgba.into(),
        })
    }

    pub(crate) fn is_valid_for(&self, base_width: u32, base_height: u32) -> bool {
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self
                .width
                .checked_mul(self.height)
                .and_then(|pixels| pixels.checked_mul(4))
                .is_some_and(|bytes| bytes as usize == self.rgba.len())
            && self
                .x
                .checked_add(self.width)
                .is_some_and(|right| right <= base_width)
            && self
                .y
                .checked_add(self.height)
                .is_some_and(|bottom| bottom <= base_height)
    }
}

#[derive(Clone, Copy)]
struct RasterLine {
    start: Vec2,
    end: Vec2,
    color: [u8; 4],
    width: f32,
}

impl RasterLine {
    fn clipped(line: HandleScreenLine, source_size: UVec2) -> Option<Self> {
        if !line.is_finite() || line.color().w <= 0.0 {
            return None;
        }
        let (start, end) = clip_line_to_viewport(line.start(), line.end(), source_size)?;
        let color = line.color().clamp(Vec4::ZERO, Vec4::ONE);
        Some(Self {
            start,
            end,
            color: [
                unit_color_channel(color.x),
                unit_color_channel(color.y),
                unit_color_channel(color.z),
                unit_color_channel(color.w),
            ],
            width: line.width().clamp(1.0, MAX_LINE_WIDTH_PX),
        })
    }
}

#[derive(Clone, Copy)]
struct RasterBounds {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

impl RasterBounds {
    fn from_lines(lines: &[RasterLine], source_size: UVec2) -> Option<Self> {
        let first = *lines.first()?;
        let mut min = first.start.min(first.end);
        let mut max = first.start.max(first.end);
        let mut padding = first.width * 0.5 + 1.0;
        for line in &lines[1..] {
            min = min.min(line.start.min(line.end));
            max = max.max(line.start.max(line.end));
            padding = padding.max(line.width * 0.5 + 1.0);
        }
        let max_x = source_size.x.saturating_sub(1) as f32;
        let max_y = source_size.y.saturating_sub(1) as f32;
        Some(Self {
            left: (min.x - padding).floor().clamp(0.0, max_x) as u32,
            top: (min.y - padding).floor().clamp(0.0, max_y) as u32,
            right: (max.x + padding).ceil().clamp(0.0, max_x) as u32,
            bottom: (max.y + padding).ceil().clamp(0.0, max_y) as u32,
        })
    }

    const fn width(self) -> u32 {
        self.right - self.left + 1
    }

    const fn height(self) -> u32 {
        self.bottom - self.top + 1
    }
}

fn raster_line(rgba: &mut [u8], bounds: RasterBounds, line: RasterLine) {
    let half_width = line.width * 0.5;
    let padding = half_width + 1.0;
    let min = line.start.min(line.end) - Vec2::splat(padding);
    let max = line.start.max(line.end) + Vec2::splat(padding);
    let left = min.x.floor().max(bounds.left as f32) as u32;
    let top = min.y.floor().max(bounds.top as f32) as u32;
    let right = max.x.ceil().min(bounds.right as f32) as u32;
    let bottom = max.y.ceil().min(bounds.bottom as f32) as u32;

    for y in top..=bottom {
        for x in left..=right {
            let sample = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let distance = distance_to_segment(sample, line.start, line.end);
            let coverage = (half_width + 0.5 - distance).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }
            let local_x = x - bounds.left;
            let local_y = y - bounds.top;
            let offset = ((local_y as usize * bounds.width() as usize) + local_x as usize) * 4;
            blend_source_over(&mut rgba[offset..offset + 4], line.color, coverage);
        }
    }
}

fn clip_line_to_viewport(start: Vec2, end: Vec2, size: UVec2) -> Option<(Vec2, Vec2)> {
    let delta = end - start;
    let max = Vec2::new(
        size.x.saturating_sub(1) as f32,
        size.y.saturating_sub(1) as f32,
    );
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for (p, q) in [
        (-delta.x, start.x),
        (delta.x, max.x - start.x),
        (-delta.y, start.y),
        (delta.y, max.y - start.y),
    ] {
        if p.abs() <= f32::EPSILON {
            if q < 0.0 {
                return None;
            }
            continue;
        }
        let ratio = q / p;
        if p < 0.0 {
            enter = enter.max(ratio);
        } else {
            exit = exit.min(ratio);
        }
        if enter > exit {
            return None;
        }
    }
    Some((start + delta * enter, start + delta * exit))
}

fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_squared();
    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }
    let t = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}

fn blend_source_over(pixel: &mut [u8], color: [u8; 4], coverage: f32) {
    blend_srgb_pixel_linear(pixel, color, coverage);
}

fn unit_color_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offscreen_line_is_clipped_before_overlay_allocation() {
        let overlay = HostViewportOverlayImageData::from_screen_lines(
            "clip-test",
            UVec2::new(320, 180),
            &[HandleScreenLine::new(
                Vec2::new(-10_000.0, 90.0),
                Vec2::new(20.0, 90.0),
                Vec4::ONE,
                2.0,
                None,
            )],
        )
        .expect("a partially visible line should rasterize");

        assert_eq!(overlay.x, 0);
        assert!(overlay.width < 32);
        assert!(overlay.height < 16);
    }

    #[test]
    fn disjoint_line_does_not_allocate_an_overlay() {
        assert!(HostViewportOverlayImageData::from_screen_lines(
            "clip-test",
            UVec2::new(320, 180),
            &[HandleScreenLine::new(
                Vec2::new(-20.0, -20.0),
                Vec2::new(-10.0, -10.0),
                Vec4::ONE,
                2.0,
                None,
            )],
        )
        .is_none());
    }

    #[test]
    fn source_over_preserves_transparency_and_blends_coverage_in_linear_light() {
        let mut transparent = [0, 0, 0, 0];
        blend_source_over(&mut transparent, [255, 255, 255, 128], 1.0);
        assert_eq!(transparent, [255, 255, 255, 128]);

        let mut opaque_black = [0, 0, 0, 255];
        blend_source_over(&mut opaque_black, [255, 255, 255, 255], 0.5);
        assert_eq!(opaque_black, [188, 188, 188, 255]);
    }
}
