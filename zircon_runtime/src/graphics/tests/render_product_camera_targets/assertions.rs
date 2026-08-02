use crate::core::framework::render::CapturedFrame;
use crate::core::math::UVec2;

#[derive(Clone, Copy)]
pub(super) struct RenderViewportRegion {
    pub(super) origin: UVec2,
    pub(super) size: UVec2,
}

impl RenderViewportRegion {
    pub(super) fn new(origin: UVec2, size: UVec2) -> Self {
        Self { origin, size }
    }
}

pub(super) fn dominant_red_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[0] > 72 && channel_exceeds(pixel[0], pixel[1]))
        .count()
}

pub(super) fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[1] > 72 && channel_exceeds(pixel[1], pixel[0]))
        .count()
}

pub(super) fn dominant_blue_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[2] > 72 && channel_exceeds(pixel[2], pixel[0]))
        .count()
}

pub(super) fn dominant_red_pixels_in_region(
    frame: &CapturedFrame,
    region: RenderViewportRegion,
) -> usize {
    dominant_pixels_in_region(frame, region, is_dominant_red)
}

pub(super) fn dominant_green_pixels_in_region(
    frame: &CapturedFrame,
    region: RenderViewportRegion,
) -> usize {
    dominant_pixels_in_region(frame, region, is_dominant_green)
}

pub(super) fn dominant_blue_pixels_in_region(
    frame: &CapturedFrame,
    region: RenderViewportRegion,
) -> usize {
    dominant_pixels_in_region(frame, region, is_dominant_blue)
}

pub(super) fn dominant_red_pixels_in_rgba_region(
    rgba: &[u8],
    extent: UVec2,
    region: RenderViewportRegion,
) -> usize {
    dominant_pixels_in_rgba_region(rgba, extent, region, is_dominant_red)
}

pub(super) fn dominant_green_pixels_in_rgba_region(
    rgba: &[u8],
    extent: UVec2,
    region: RenderViewportRegion,
) -> usize {
    dominant_pixels_in_rgba_region(rgba, extent, region, is_dominant_green)
}

pub(super) fn dominant_blue_pixels_in_rgba_region(
    rgba: &[u8],
    extent: UVec2,
    region: RenderViewportRegion,
) -> usize {
    dominant_pixels_in_rgba_region(rgba, extent, region, is_dominant_blue)
}

pub(super) fn average_channel_in_region(
    frame: &CapturedFrame,
    region: RenderViewportRegion,
    channel: usize,
) -> f32 {
    let x_end = region
        .origin
        .x
        .saturating_add(region.size.x)
        .min(frame.width) as usize;
    let y_end = region
        .origin
        .y
        .saturating_add(region.size.y)
        .min(frame.height) as usize;
    let width = frame.width as usize;
    let mut total = 0.0;
    let mut count = 0.0;
    for y in region.origin.y as usize..y_end {
        for x in region.origin.x as usize..x_end {
            let index = (y * width + x) * 4 + channel;
            total += frame.rgba[index] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn dominant_pixels_in_region(
    frame: &CapturedFrame,
    region: RenderViewportRegion,
    predicate: fn(&[u8]) -> bool,
) -> usize {
    dominant_pixels_in_rgba_region(
        &frame.rgba,
        UVec2::new(frame.width, frame.height),
        region,
        predicate,
    )
}

fn dominant_pixels_in_rgba_region(
    rgba: &[u8],
    extent: UVec2,
    region: RenderViewportRegion,
    predicate: fn(&[u8]) -> bool,
) -> usize {
    let x_end = region.origin.x.saturating_add(region.size.x).min(extent.x) as usize;
    let y_end = region.origin.y.saturating_add(region.size.y).min(extent.y) as usize;
    let width = extent.x as usize;
    let mut count = 0;
    for y in region.origin.y as usize..y_end {
        for x in region.origin.x as usize..x_end {
            let index = (y * width + x) * 4;
            if predicate(&rgba[index..index + 4]) {
                count += 1;
            }
        }
    }
    count
}

pub(super) fn rgba_pixel_at(rgba: &[u8], width: u32, position: UVec2) -> [u8; 4] {
    let width = width as usize;
    let index = (position.y as usize * width + position.x as usize) * 4;
    [
        rgba[index],
        rgba[index + 1],
        rgba[index + 2],
        rgba[index + 3],
    ]
}

pub(super) fn is_dominant_red(pixel: &[u8]) -> bool {
    pixel[3] >= 240 && pixel[0] > 72 && channel_exceeds(pixel[0], pixel[1])
}

pub(super) fn is_dominant_green(pixel: &[u8]) -> bool {
    pixel[3] >= 240 && pixel[1] > 72 && channel_exceeds(pixel[1], pixel[0])
}

fn is_dominant_blue(pixel: &[u8]) -> bool {
    pixel[3] >= 240 && pixel[2] > 72 && channel_exceeds(pixel[2], pixel[0])
}

fn channel_exceeds(channel: u8, other: u8) -> bool {
    u16::from(channel) > u16::from(other) + 32
}
