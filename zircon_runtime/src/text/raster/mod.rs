//! Glyph rasterization policy and backend routing.

mod policy;
mod swash;

pub(crate) use policy::{
    distance_field_mode_for_request, raster_path_for, raster_path_for_request, GlyphRasterEffects,
    GlyphRasterPath, GlyphRasterPolicy, GlyphRasterPolicyRequest,
};
pub(crate) use swash::{
    color_glyph_raster_plan, glyph_atlas_bitmap_source_from_glyph_bitmap,
    select_color_bitmap_strike, ColorGlyphBitmapStrike, ColorGlyphBitmapStrikeFit,
    ColorGlyphBitmapStrikeSelection, ColorGlyphRasterPlan, GlyphBitmap, GlyphBitmapError,
    SwashBitmapStrike, SwashRasterError, SwashRasterImageContent, SwashRasterRequest,
    SwashRasterSource, SwashRasterizer,
};
