use crate::core::math::UVec2;

const GLYPH_ATLAS_GPU_NDC_MIN: f32 = -1.0;
const GLYPH_ATLAS_GPU_NDC_MAX: f32 = 1.0;
const GLYPH_ATLAS_GPU_NDC_SPAN: f32 = GLYPH_ATLAS_GPU_NDC_MAX - GLYPH_ATLAS_GPU_NDC_MIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuPixelCoordinateConvention {
    /// Glyph quads describe pixel edges; sampling centers are handled by the rasterizer/backend.
    PixelEdges,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasGpuViewportTransform {
    pub(crate) viewport_size: UVec2,
    pub(crate) pixel_coordinate_convention: GlyphAtlasGpuPixelCoordinateConvention,
}

impl Default for GlyphAtlasGpuViewportTransform {
    fn default() -> Self {
        Self::new(UVec2::new(1, 1))
    }
}

impl GlyphAtlasGpuViewportTransform {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size,
            pixel_coordinate_convention: GlyphAtlasGpuPixelCoordinateConvention::PixelEdges,
        }
    }

    pub(crate) fn position_ndc(&self, position_px: [f32; 2]) -> [f32; 2] {
        [
            self.pixel_to_ndc_x(position_px[0]),
            self.pixel_to_ndc_y(position_px[1]),
        ]
    }

    fn pixel_to_ndc_x(&self, x: f32) -> f32 {
        (x / self.viewport_width() as f32) * GLYPH_ATLAS_GPU_NDC_SPAN + GLYPH_ATLAS_GPU_NDC_MIN
    }

    fn pixel_to_ndc_y(&self, y: f32) -> f32 {
        GLYPH_ATLAS_GPU_NDC_MAX - (y / self.viewport_height() as f32) * GLYPH_ATLAS_GPU_NDC_SPAN
    }

    fn viewport_width(&self) -> u32 {
        self.viewport_size.x.max(1)
    }

    fn viewport_height(&self) -> u32 {
        self.viewport_size.y.max(1)
    }
}

pub(crate) fn glyph_atlas_gpu_viewport_transform(
    viewport_size: UVec2,
) -> GlyphAtlasGpuViewportTransform {
    GlyphAtlasGpuViewportTransform::new(viewport_size)
}
