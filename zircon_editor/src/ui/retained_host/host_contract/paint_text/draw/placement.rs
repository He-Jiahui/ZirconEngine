use super::super::super::paint_theme::HostTextSmoothing;

mod metrics;

pub(super) use metrics::RETAINED_TEXT_SUBPIXEL_BINS;
use metrics::{
    finite_text_origin, quantized_left_offset_px, screen_pixel_x, screen_subpixel_bin,
    subpixel_offset_for_bin,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RetainedGlyphPlacement {
    pub(super) pixel_x: i32,
    pub(super) subpixel_offset: f32,
}

impl RetainedGlyphPlacement {
    pub(super) fn from_screen_x(screen_x: f32) -> Self {
        let pixel_x = screen_pixel_x(screen_x);
        let bin = screen_subpixel_bin(screen_x);
        Self {
            pixel_x,
            subpixel_offset: subpixel_offset_for_bin(bin),
        }
    }
}

pub(super) fn retained_glyph_placement_for_smoothing(
    screen_x: f32,
    smoothing: HostTextSmoothing,
) -> RetainedGlyphPlacement {
    match smoothing {
        HostTextSmoothing::Grayscale | HostTextSmoothing::Subpixel => {
            RetainedGlyphPlacement::from_screen_x(screen_x)
        }
    }
}

pub(super) fn retained_glyph_placements_share_bin_for_smoothing(
    a: f32,
    b: f32,
    smoothing: HostTextSmoothing,
) -> bool {
    retained_glyph_placement_for_smoothing(a, smoothing)
        == retained_glyph_placement_for_smoothing(b, smoothing)
}

pub(super) fn retained_text_origin_for_smoothing(value: f32, smoothing: HostTextSmoothing) -> f32 {
    match smoothing {
        HostTextSmoothing::Grayscale => retained_text_origin_device_px(value),
        HostTextSmoothing::Subpixel => retained_text_origin_subpixel_px(value),
    }
}

pub(super) fn retained_text_origin_device_px(value: f32) -> f32 {
    finite_text_origin(value).round()
}

fn retained_text_origin_subpixel_px(value: f32) -> f32 {
    finite_text_origin(value)
}

pub(super) fn retained_glyph_left_offset_px(offset: f32) -> f32 {
    quantized_left_offset_px(offset)
}

#[cfg(test)]
mod tests;
