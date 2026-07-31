use super::bitmap::GlyphBitmap;
use super::error::SwashRasterError;
use super::request::SwashRasterRequest;
use crate::core::math::{UVec2, Vec2};
use swash::FontRef;
use swash::scale::{
    Render, ScaleContext,
    image::{Content as SwashImageContent, Image as SwashImage},
};
use swash::zeno::Vector as SwashVector;
use swash::{Setting as SwashSetting, Tag as SwashTag};

pub(crate) struct SwashRasterizer {
    context: ScaleContext,
    #[cfg(test)]
    scaler_build_count: usize,
}

impl SwashRasterizer {
    pub(crate) fn new() -> Self {
        Self {
            context: ScaleContext::new(),
            #[cfg(test)]
            scaler_build_count: 0,
        }
    }

    pub(crate) fn rasterize(
        &mut self,
        font_data: &[u8],
        request: SwashRasterRequest,
    ) -> Result<GlyphBitmap, SwashRasterError> {
        let mut result = Err(SwashRasterError::MissingGlyphImage {
            glyph_id: request.glyph_id,
            source: request.primary_source(),
        });
        self.rasterize_batch(
            font_data,
            std::slice::from_ref(&request),
            |_, next_result| {
                result = next_result;
            },
        );
        result
    }

    pub(crate) fn rasterize_batch<F>(
        &mut self,
        font_data: &[u8],
        requests: &[SwashRasterRequest],
        mut consume: F,
    ) where
        F: FnMut(usize, Result<GlyphBitmap, SwashRasterError>),
    {
        let Some(first_request) = requests.first() else {
            return;
        };
        if requests
            .iter()
            .skip(1)
            .any(|request| !first_request.shares_scaler_configuration_with(request))
        {
            for (index, request) in requests.iter().enumerate() {
                consume(index, self.rasterize(font_data, request.clone()));
            }
            return;
        }

        let mut valid_indices = Vec::with_capacity(requests.len());
        for (index, request) in requests.iter().enumerate() {
            match request.validate() {
                Ok(()) => valid_indices.push(index),
                Err(error) => consume(index, Err(error)),
            }
        }
        let Some(&first_valid_index) = valid_indices.first() else {
            return;
        };
        let first_valid_request = &requests[first_valid_index];
        let Some(font) = FontRef::from_index(font_data, first_valid_request.face_index) else {
            for index in valid_indices {
                consume(
                    index,
                    Err(SwashRasterError::InvalidFontFace {
                        face_index: requests[index].face_index,
                    }),
                );
            }
            return;
        };
        let mut scaler_builder = match first_valid_request.font_identity {
            Some(font_identity) => self.context.builder_with_id(font, font_identity),
            None => self.context.builder(font),
        }
        .size(first_valid_request.px_size)
        .hint(first_valid_request.hint);
        if !first_valid_request.variations.0.is_empty() {
            scaler_builder =
                scaler_builder.variations(first_valid_request.variations.0.iter().map(
                    |(tag, value)| SwashSetting {
                        tag: SwashTag::from_be_bytes(tag.to_be_bytes()),
                        value: *value,
                    },
                ));
        }
        let mut scaler = scaler_builder.build();
        #[cfg(test)]
        {
            self.scaler_build_count = self.scaler_build_count.saturating_add(1);
        }

        for index in valid_indices {
            let request = &requests[index];
            let sources = request.swash_sources();
            let mut render = Render::new(&sources[..request.source_count()]);
            render
                .format(request.render_format)
                .offset(SwashVector::new(request.offset.x, request.offset.y))
                .transform(request.fake_italic_transform());
            let result = render
                .render(&mut scaler, request.glyph_id)
                .ok_or(SwashRasterError::MissingGlyphImage {
                    glyph_id: request.glyph_id,
                    source: request.primary_source(),
                })
                .and_then(|image| glyph_bitmap_from_swash_image(image, request.px_size));
            consume(index, result);
        }
    }

    #[cfg(test)]
    pub(crate) fn scaler_build_count_for_test(&self) -> usize {
        self.scaler_build_count
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
