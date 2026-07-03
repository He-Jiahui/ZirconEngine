use super::bitmap::GlyphBitmap;
use super::error::SwashRasterError;
use super::request::SwashRasterRequest;
use crate::core::math::{UVec2, Vec2};
use ::swash::scale::{
    image::{Content as SwashImageContent, Image as SwashImage},
    Render, ScaleContext,
};
use ::swash::FontRef;
use std::slice;

pub(crate) struct SwashRasterizer {
    context: ScaleContext,
}

impl SwashRasterizer {
    pub(crate) fn new() -> Self {
        Self {
            context: ScaleContext::new(),
        }
    }

    pub(crate) fn rasterize(
        &mut self,
        font_data: &[u8],
        request: SwashRasterRequest,
    ) -> Result<GlyphBitmap, SwashRasterError> {
        request.validate()?;
        let font = FontRef::from_index(font_data, request.face_index).ok_or(
            SwashRasterError::InvalidFontFace {
                face_index: request.face_index,
            },
        )?;
        let mut scaler = self
            .context
            .builder(font)
            .size(request.px_size)
            .hint(request.hint)
            .build();
        let source = request.source.to_swash_source();
        let mut render = Render::new(slice::from_ref(&source));
        render.format(request.source.render_format());
        let image = render.render(&mut scaler, request.glyph_id).ok_or(
            SwashRasterError::MissingGlyphImage {
                glyph_id: request.glyph_id,
                source: request.source,
            },
        )?;

        glyph_bitmap_from_swash_image(image, request.px_size)
    }

    pub(crate) fn rasterize_alpha_outline(
        &mut self,
        font_data: &[u8],
        face_index: usize,
        glyph_id: u16,
        px_size: f32,
        hint: bool,
    ) -> Result<GlyphBitmap, SwashRasterError> {
        self.rasterize(
            font_data,
            SwashRasterRequest::alpha_outline(face_index, glyph_id, px_size, hint),
        )
    }

    pub(crate) fn rasterize_subpixel_outline(
        &mut self,
        font_data: &[u8],
        face_index: usize,
        glyph_id: u16,
        px_size: f32,
        hint: bool,
    ) -> Result<GlyphBitmap, SwashRasterError> {
        self.rasterize(
            font_data,
            SwashRasterRequest::subpixel_outline(face_index, glyph_id, px_size, hint),
        )
    }
}

impl Default for SwashRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SwashRasterImageContent {
    Mask,
    SubpixelMask,
    Color,
}

impl From<SwashImageContent> for SwashRasterImageContent {
    fn from(content: SwashImageContent) -> Self {
        match content {
            SwashImageContent::Mask => Self::Mask,
            SwashImageContent::SubpixelMask => Self::SubpixelMask,
            SwashImageContent::Color => Self::Color,
        }
    }
}

pub(super) fn glyph_bitmap_from_swash_image(
    image: SwashImage,
    px_size: f32,
) -> Result<GlyphBitmap, SwashRasterError> {
    let content = SwashRasterImageContent::from(image.content);
    let size = UVec2::new(image.placement.width, image.placement.height);
    let bearing = Vec2::new(image.placement.left as f32, image.placement.top as f32);

    match content {
        SwashRasterImageContent::Mask => {
            GlyphBitmap::alpha_mask(size, bearing, px_size, image.data)
        }
        SwashRasterImageContent::Color => GlyphBitmap::color(size, bearing, px_size, image.data),
        SwashRasterImageContent::SubpixelMask => {
            GlyphBitmap::subpixel_mask(size, bearing, px_size, image.data)
        }
    }
    .map_err(SwashRasterError::InvalidGlyphBitmap)
}
