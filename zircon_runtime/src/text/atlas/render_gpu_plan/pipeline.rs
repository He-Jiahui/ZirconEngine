use super::bind_group::{GlyphAtlasGpuBindGroupLayout, glyph_atlas_gpu_bind_group_layout};
use super::draw_command::GlyphAtlasGpuPrimitiveTopology;
use super::instance::{GlyphAtlasGpuInstanceBufferLayout, glyph_atlas_gpu_instance_buffer_layout};
use crate::text::atlas::render_contract::{GlyphAtlasRenderContract, GlyphAtlasShaderEntryPoints};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuPipelineKey {
    pub(crate) render_contract: GlyphAtlasRenderContract,
    pub(crate) primitive_topology: GlyphAtlasGpuPrimitiveTopology,
}

/// Pipeline-facing state that must match the atlas sampling shader and draw-command topology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuPipelineContract {
    pub(crate) key: GlyphAtlasGpuPipelineKey,
    pub(crate) shader_entry_points: GlyphAtlasShaderEntryPoints,
    pub(crate) instance_layout: GlyphAtlasGpuInstanceBufferLayout,
    pub(crate) bind_group_layout: GlyphAtlasGpuBindGroupLayout,
}

pub(crate) fn glyph_atlas_gpu_pipeline_contract(
    key: GlyphAtlasGpuPipelineKey,
) -> GlyphAtlasGpuPipelineContract {
    GlyphAtlasGpuPipelineContract {
        key,
        shader_entry_points: key.render_contract.shader_entry_points(),
        instance_layout: glyph_atlas_gpu_instance_buffer_layout(),
        bind_group_layout: glyph_atlas_gpu_bind_group_layout(),
    }
}
