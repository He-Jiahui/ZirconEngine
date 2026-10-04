//! 把左上原点的像素边缘矩形交给顶点着色器转换为裁剪空间。
//! 不额外添加半像素偏移；光栅位置已经由上游按灰度或子像素策略对齐。

use crate::core::math::UVec2;

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

    pub(crate) fn uniform_bytes(&self) -> [f32; 4] {
        [
            self.viewport_width() as f32,
            self.viewport_height() as f32,
            0.0,
            0.0,
        ]
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
