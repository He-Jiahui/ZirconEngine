use super::super::super::paint_theme::HostTextSmoothing;

pub(super) const RETAINED_TEXT_SUBPIXEL_BINS: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RetainedGlyphPlacement {
    pub(super) pixel_x: i32,
    pub(super) subpixel_offset: f32,
}

impl RetainedGlyphPlacement {
    pub(super) fn from_screen_x(screen_x: f32) -> Self {
        let screen_x = if screen_x.is_finite() { screen_x } else { 0.0 };
        let mut pixel_x = screen_x.floor() as i32;
        let fraction = screen_x.rem_euclid(1.0);
        let mut bin = (fraction * RETAINED_TEXT_SUBPIXEL_BINS as f32).round() as u8;
        if bin >= RETAINED_TEXT_SUBPIXEL_BINS {
            pixel_x += 1;
            bin = 0;
        }
        Self {
            pixel_x,
            subpixel_offset: bin as f32 / RETAINED_TEXT_SUBPIXEL_BINS as f32,
        }
    }
}

pub(super) fn retained_glyph_placement_for_smoothing(
    screen_x: f32,
    smoothing: HostTextSmoothing,
) -> RetainedGlyphPlacement {
    match smoothing {
        // Grayscale controls the coverage format, not per-glyph x quantization.
        // The line origin is snapped separately; glyph origins keep an alpha
        // phase so small editor labels do not gain uneven integer-spacing jitter.
        HostTextSmoothing::Grayscale => RetainedGlyphPlacement::from_screen_x(screen_x),
        HostTextSmoothing::Subpixel => RetainedGlyphPlacement::from_screen_x(screen_x),
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

pub(super) fn retained_text_origin_device_px(value: f32) -> f32 {
    if value.is_finite() {
        value.round()
    } else {
        0.0
    }
}

pub(super) fn retained_text_origin_for_smoothing(value: f32, smoothing: HostTextSmoothing) -> f32 {
    match smoothing {
        HostTextSmoothing::Grayscale => retained_text_origin_device_px(value),
        HostTextSmoothing::Subpixel => retained_text_origin_subpixel_px(value),
    }
}

fn retained_text_origin_subpixel_px(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

pub(super) fn retained_glyph_left_offset_px(offset: f32) -> f32 {
    if !offset.is_finite() {
        return 0.0;
    }

    let bins = RETAINED_TEXT_SUBPIXEL_BINS as f32;
    (offset * bins).round() / bins
}

#[cfg(test)]
mod tests;
