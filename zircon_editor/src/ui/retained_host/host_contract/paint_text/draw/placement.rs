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

pub(super) fn retained_glyph_placements_share_bin(a: f32, b: f32) -> bool {
    RetainedGlyphPlacement::from_screen_x(a) == RetainedGlyphPlacement::from_screen_x(b)
}
