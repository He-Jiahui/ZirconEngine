use bytemuck::{Pod, Zeroable};

const GLYPH_ATLAS_GPU_INSTANCE_F32_BYTES: u64 = std::mem::size_of::<f32>() as u64;
const GLYPH_ATLAS_GPU_INSTANCE_U32_BYTES: u64 = std::mem::size_of::<u32>() as u64;
const GLYPH_ATLAS_GPU_INSTANCE_RECT_COMPONENTS: u64 = 4;
const GLYPH_ATLAS_GPU_INSTANCE_COLOR_COMPONENTS: u64 = 4;
const GLYPH_ATLAS_GPU_INSTANCE_SCREEN_RECT_OFFSET_BYTES: u64 = 0;
const GLYPH_ATLAS_GPU_INSTANCE_UV_RECT_OFFSET_BYTES: u64 =
    GLYPH_ATLAS_GPU_INSTANCE_SCREEN_RECT_OFFSET_BYTES
        + GLYPH_ATLAS_GPU_INSTANCE_RECT_COMPONENTS * GLYPH_ATLAS_GPU_INSTANCE_F32_BYTES;
const GLYPH_ATLAS_GPU_INSTANCE_FOREGROUND_OFFSET_BYTES: u64 =
    GLYPH_ATLAS_GPU_INSTANCE_UV_RECT_OFFSET_BYTES
        + GLYPH_ATLAS_GPU_INSTANCE_RECT_COMPONENTS * GLYPH_ATLAS_GPU_INSTANCE_F32_BYTES;
const GLYPH_ATLAS_GPU_INSTANCE_BACKGROUND_OFFSET_BYTES: u64 =
    GLYPH_ATLAS_GPU_INSTANCE_FOREGROUND_OFFSET_BYTES
        + GLYPH_ATLAS_GPU_INSTANCE_COLOR_COMPONENTS * GLYPH_ATLAS_GPU_INSTANCE_F32_BYTES;
const GLYPH_ATLAS_GPU_INSTANCE_PAGE_INDEX_OFFSET_BYTES: u64 =
    GLYPH_ATLAS_GPU_INSTANCE_BACKGROUND_OFFSET_BYTES
        + GLYPH_ATLAS_GPU_INSTANCE_COLOR_COMPONENTS * GLYPH_ATLAS_GPU_INSTANCE_F32_BYTES;
const GLYPH_ATLAS_GPU_INSTANCE_STRIDE_BYTES: u64 =
    GLYPH_ATLAS_GPU_INSTANCE_PAGE_INDEX_OFFSET_BYTES + GLYPH_ATLAS_GPU_INSTANCE_U32_BYTES;
const GLYPH_ATLAS_GPU_INSTANCE_ATTRIBUTE_COUNT: usize = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct GlyphAtlasGpuInstance {
    pub(crate) screen_rect_px: [f32; 4],
    pub(crate) uv_rect: [f32; 4],
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
    pub(crate) page_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuInstanceAttributeSemantic {
    ScreenRectPx,
    UvRect,
    ForegroundColor,
    BackgroundColor,
    PageIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuInstanceAttributeFormat {
    Float32x4,
    Uint32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuInstanceAttribute {
    pub(crate) semantic: GlyphAtlasGpuInstanceAttributeSemantic,
    pub(crate) shader_location: u32,
    pub(crate) format: GlyphAtlasGpuInstanceAttributeFormat,
    pub(crate) offset_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuInstanceBufferLayout {
    pub(crate) stride_bytes: u64,
    pub(crate) attributes:
        [GlyphAtlasGpuInstanceAttribute; GLYPH_ATLAS_GPU_INSTANCE_ATTRIBUTE_COUNT],
}

pub(crate) fn glyph_atlas_gpu_instance_buffer_layout() -> GlyphAtlasGpuInstanceBufferLayout {
    GlyphAtlasGpuInstanceBufferLayout {
        stride_bytes: GLYPH_ATLAS_GPU_INSTANCE_STRIDE_BYTES,
        attributes: [
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::ScreenRectPx,
                shader_location: 0,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: GLYPH_ATLAS_GPU_INSTANCE_SCREEN_RECT_OFFSET_BYTES,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::UvRect,
                shader_location: 1,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: GLYPH_ATLAS_GPU_INSTANCE_UV_RECT_OFFSET_BYTES,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::ForegroundColor,
                shader_location: 2,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: GLYPH_ATLAS_GPU_INSTANCE_FOREGROUND_OFFSET_BYTES,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::BackgroundColor,
                shader_location: 3,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: GLYPH_ATLAS_GPU_INSTANCE_BACKGROUND_OFFSET_BYTES,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::PageIndex,
                shader_location: 4,
                format: GlyphAtlasGpuInstanceAttributeFormat::Uint32,
                offset_bytes: GLYPH_ATLAS_GPU_INSTANCE_PAGE_INDEX_OFFSET_BYTES,
            },
        ],
    }
}
