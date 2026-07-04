use crate::graphics::text::atlas::render_gpu_plan::{
    GlyphAtlasGpuVertexAttributeFormat, GlyphAtlasGpuVertexBufferLayout,
};

const GLYPH_ATLAS_WGPU_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 8,
        shader_location: 1,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 16,
        shader_location: 2,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 32,
        shader_location: 3,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset: 48,
        shader_location: 4,
    },
];

pub(super) fn glyph_atlas_wgpu_vertex_buffer_layout(
    layout: GlyphAtlasGpuVertexBufferLayout,
) -> wgpu::VertexBufferLayout<'static> {
    for (attribute, wgpu_attribute) in layout
        .attributes
        .iter()
        .zip(GLYPH_ATLAS_WGPU_VERTEX_ATTRIBUTES.iter())
    {
        debug_assert_eq!(
            wgpu_attribute.format,
            glyph_atlas_wgpu_vertex_format(attribute.format)
        );
        debug_assert_eq!(wgpu_attribute.offset, attribute.offset_bytes);
        debug_assert_eq!(wgpu_attribute.shader_location, attribute.shader_location);
    }

    wgpu::VertexBufferLayout {
        array_stride: layout.stride_bytes,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &GLYPH_ATLAS_WGPU_VERTEX_ATTRIBUTES,
    }
}

fn glyph_atlas_wgpu_vertex_format(
    format: GlyphAtlasGpuVertexAttributeFormat,
) -> wgpu::VertexFormat {
    match format {
        GlyphAtlasGpuVertexAttributeFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        GlyphAtlasGpuVertexAttributeFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
        GlyphAtlasGpuVertexAttributeFormat::Uint32 => wgpu::VertexFormat::Uint32,
    }
}
