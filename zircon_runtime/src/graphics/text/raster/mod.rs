//! Glyph rasterization policy and backend routing.

mod policy;
mod swash;

pub(crate) use policy::{
    raster_path_for, raster_path_for_request, GlyphRasterEffects, GlyphRasterPath,
    GlyphRasterPolicy, GlyphRasterPolicyRequest,
};
pub(crate) use swash::{
    color_glyph_raster_plan, select_color_bitmap_strike, ColorGlyphBitmapStrike,
    ColorGlyphBitmapStrikeFit, ColorGlyphBitmapStrikeSelection, ColorGlyphRasterPlan, GlyphBitmap,
    GlyphBitmapError, SwashBitmapStrike, SwashRasterError, SwashRasterImageContent,
    SwashRasterRequest, SwashRasterSource, SwashRasterizer,
};
