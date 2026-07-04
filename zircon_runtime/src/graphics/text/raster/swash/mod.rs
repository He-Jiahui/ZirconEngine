mod atlas_source;
mod bitmap;
mod color_strike;
mod error;
mod rasterizer;
mod request;

pub(crate) use atlas_source::glyph_atlas_bitmap_source_from_glyph_bitmap;
pub(crate) use bitmap::{GlyphBitmap, GlyphBitmapContent, GlyphBitmapError};
pub(crate) use color_strike::{
    color_glyph_raster_plan, select_color_bitmap_strike, ColorGlyphBitmapStrike,
    ColorGlyphBitmapStrikeFit, ColorGlyphBitmapStrikeSelection, ColorGlyphRasterPlan,
};
pub(crate) use error::SwashRasterError;
pub(crate) use rasterizer::{SwashRasterImageContent, SwashRasterizer};
pub(crate) use request::{SwashBitmapStrike, SwashRasterRequest, SwashRasterSource};

#[cfg(test)]
mod tests;
