const GLYPH_ATLAS_GPU_VERTEX_F32_BYTES: u64 = std::mem::size_of::<f32>() as u64;
const GLYPH_ATLAS_GPU_VERTEX_U32_BYTES: u64 = std::mem::size_of::<u32>() as u64;
const GLYPH_ATLAS_GPU_VERTEX_POSITION_COMPONENTS: u64 = 2;
const GLYPH_ATLAS_GPU_VERTEX_UV_COMPONENTS: u64 = 2;
const GLYPH_ATLAS_GPU_VERTEX_COLOR_COMPONENTS: u64 = 4;
const GLYPH_ATLAS_GPU_VERTEX_POSITION_OFFSET_BYTES: u64 = 0;
const GLYPH_ATLAS_GPU_VERTEX_UV_OFFSET_BYTES: u64 = GLYPH_ATLAS_GPU_VERTEX_POSITION_OFFSET_BYTES
    + GLYPH_ATLAS_GPU_VERTEX_POSITION_COMPONENTS * GLYPH_ATLAS_GPU_VERTEX_F32_BYTES;
const GLYPH_ATLAS_GPU_VERTEX_FOREGROUND_OFFSET_BYTES: u64 = GLYPH_ATLAS_GPU_VERTEX_UV_OFFSET_BYTES
    + GLYPH_ATLAS_GPU_VERTEX_UV_COMPONENTS * GLYPH_ATLAS_GPU_VERTEX_F32_BYTES;
const GLYPH_ATLAS_GPU_VERTEX_BACKGROUND_OFFSET_BYTES: u64 =
    GLYPH_ATLAS_GPU_VERTEX_FOREGROUND_OFFSET_BYTES
        + GLYPH_ATLAS_GPU_VERTEX_COLOR_COMPONENTS * GLYPH_ATLAS_GPU_VERTEX_F32_BYTES;
const GLYPH_ATLAS_GPU_VERTEX_PAGE_INDEX_OFFSET_BYTES: u64 =
    GLYPH_ATLAS_GPU_VERTEX_BACKGROUND_OFFSET_BYTES
        + GLYPH_ATLAS_GPU_VERTEX_COLOR_COMPONENTS * GLYPH_ATLAS_GPU_VERTEX_F32_BYTES;
const GLYPH_ATLAS_GPU_VERTEX_STRIDE_BYTES: u64 =
    GLYPH_ATLAS_GPU_VERTEX_PAGE_INDEX_OFFSET_BYTES + GLYPH_ATLAS_GPU_VERTEX_U32_BYTES;
const GLYPH_ATLAS_GPU_VERTEX_ATTRIBUTE_COUNT: usize = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasGpuVertex {
    pub(crate) position_ndc: [f32; 2],
    pub(crate) uv: [f32; 2],
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
    pub(crate) page_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuVertexAttributeSemantic {
    PositionNdc,
    Uv,
    ForegroundColor,
    BackgroundColor,
    PageIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuVertexAttributeFormat {
    Float32x2,
    Float32x4,
    Uint32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuVertexAttribute {
    pub(crate) semantic: GlyphAtlasGpuVertexAttributeSemantic,
    pub(crate) shader_location: u32,
    pub(crate) format: GlyphAtlasGpuVertexAttributeFormat,
    pub(crate) offset_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuVertexBufferLayout {
    pub(crate) stride_bytes: u64,
    pub(crate) attributes: [GlyphAtlasGpuVertexAttribute; GLYPH_ATLAS_GPU_VERTEX_ATTRIBUTE_COUNT],
}

pub(crate) fn glyph_atlas_gpu_vertex_buffer_layout() -> GlyphAtlasGpuVertexBufferLayout {
    GlyphAtlasGpuVertexBufferLayout {
        stride_bytes: GLYPH_ATLAS_GPU_VERTEX_STRIDE_BYTES,
        attributes: [
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::PositionNdc,
                shader_location: 0,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x2,
                offset_bytes: GLYPH_ATLAS_GPU_VERTEX_POSITION_OFFSET_BYTES,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::Uv,
                shader_location: 1,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x2,
                offset_bytes: GLYPH_ATLAS_GPU_VERTEX_UV_OFFSET_BYTES,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::ForegroundColor,
                shader_location: 2,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x4,
                offset_bytes: GLYPH_ATLAS_GPU_VERTEX_FOREGROUND_OFFSET_BYTES,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::BackgroundColor,
                shader_location: 3,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x4,
                offset_bytes: GLYPH_ATLAS_GPU_VERTEX_BACKGROUND_OFFSET_BYTES,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::PageIndex,
                shader_location: 4,
                format: GlyphAtlasGpuVertexAttributeFormat::Uint32,
                offset_bytes: GLYPH_ATLAS_GPU_VERTEX_PAGE_INDEX_OFFSET_BYTES,
            },
        ],
    }
}
