use crate::core::math::{UVec2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColorGlyphBitmapStrike {
    pub(crate) ppem: u16,
    pub(crate) bitmap_size: UVec2,
    pub(crate) bearing: Vec2,
    pub(crate) advance_px: f32,
}

impl ColorGlyphBitmapStrike {
    pub(crate) fn new(ppem: u16, bitmap_size: UVec2, bearing: Vec2, advance_px: f32) -> Self {
        Self {
            ppem,
            bitmap_size,
            bearing,
            advance_px,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColorGlyphBitmapStrikeFit {
    Exact,
    Downsample,
    UpscaleFallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColorGlyphBitmapStrikeSelection {
    pub(crate) strike: ColorGlyphBitmapStrike,
    pub(crate) target_px: f32,
    pub(crate) scale: f32,
    pub(crate) fit: ColorGlyphBitmapStrikeFit,
}

impl ColorGlyphBitmapStrikeSelection {
    pub(crate) fn scaled_size(self) -> UVec2 {
        UVec2::new(
            scale_dimension(self.strike.bitmap_size.x, self.scale),
            scale_dimension(self.strike.bitmap_size.y, self.scale),
        )
    }

    pub(crate) fn scaled_bearing(self) -> Vec2 {
        Vec2::new(
            self.strike.bearing.x * self.scale,
            self.strike.bearing.y * self.scale,
        )
    }

    pub(crate) fn scaled_advance_px(self) -> f32 {
        self.strike.advance_px * self.scale
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ColorGlyphRasterPlan {
    ColrCpalVector,
    BitmapStrike(ColorGlyphBitmapStrikeSelection),
    Missing,
}

pub(crate) fn color_glyph_raster_plan(
    has_colr_cpal: bool,
    target_px: f32,
    bitmap_strikes: &[ColorGlyphBitmapStrike],
) -> ColorGlyphRasterPlan {
    if has_colr_cpal {
        return ColorGlyphRasterPlan::ColrCpalVector;
    }

    select_color_bitmap_strike(target_px, bitmap_strikes)
        .map(ColorGlyphRasterPlan::BitmapStrike)
        .unwrap_or(ColorGlyphRasterPlan::Missing)
}

pub(crate) fn select_color_bitmap_strike(
    target_px: f32,
    strikes: &[ColorGlyphBitmapStrike],
) -> Option<ColorGlyphBitmapStrikeSelection> {
    let target_px = finite_positive_or_default(target_px, 1.0);
    let mut nearest_larger_or_equal = None;
    let mut largest_smaller = None;

    for strike in strikes.iter().copied().filter(valid_bitmap_strike) {
        if strike.ppem as f32 >= target_px {
            if nearest_larger_or_equal
                .map(|candidate: ColorGlyphBitmapStrike| strike.ppem < candidate.ppem)
                .unwrap_or(true)
            {
                nearest_larger_or_equal = Some(strike);
            }
        } else if largest_smaller
            .map(|candidate: ColorGlyphBitmapStrike| strike.ppem > candidate.ppem)
            .unwrap_or(true)
        {
            largest_smaller = Some(strike);
        }
    }

    let strike = nearest_larger_or_equal.or(largest_smaller)?;
    let scale = target_px / strike.ppem as f32;
    Some(ColorGlyphBitmapStrikeSelection {
        strike,
        target_px,
        scale,
        fit: strike_fit(strike.ppem, target_px),
    })
}

fn valid_bitmap_strike(strike: &ColorGlyphBitmapStrike) -> bool {
    strike.ppem > 0
        && strike.bitmap_size.x > 0
        && strike.bitmap_size.y > 0
        && strike.bearing.x.is_finite()
        && strike.bearing.y.is_finite()
        && strike.advance_px.is_finite()
        && strike.advance_px >= 0.0
}

fn strike_fit(ppem: u16, target_px: f32) -> ColorGlyphBitmapStrikeFit {
    let ppem = ppem as f32;
    if (ppem - target_px).abs() <= f32::EPSILON {
        ColorGlyphBitmapStrikeFit::Exact
    } else if ppem > target_px {
        ColorGlyphBitmapStrikeFit::Downsample
    } else {
        ColorGlyphBitmapStrikeFit::UpscaleFallback
    }
}

fn scale_dimension(value: u32, scale: f32) -> u32 {
    ((value as f32 * scale).round()).max(1.0) as u32
}

fn finite_positive_or_default(value: f32, default_value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        default_value
    }
}
