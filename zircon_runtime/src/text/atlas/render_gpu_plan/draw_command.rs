use super::pipeline::GlyphAtlasGpuPipelineKey;
use crate::text::atlas::render_batch::GlyphAtlasDrawBatchKey;
use crate::text::atlas::render_contract::GlyphAtlasRenderContract;

pub(crate) const GLYPH_ATLAS_GPU_VERTICES_PER_INSTANCE: u32 = 6;
const GLYPH_ATLAS_GPU_TRIANGLES_PER_INSTANCE: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuBatch {
    pub(crate) key: GlyphAtlasDrawBatchKey,
    pub(crate) instance_start: u32,
    pub(crate) instance_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuPrimitiveTopology {
    TriangleList,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuDrawCommand {
    pub(crate) key: GlyphAtlasDrawBatchKey,
    pub(crate) pipeline_key: GlyphAtlasGpuPipelineKey,
    pub(crate) render_contract: GlyphAtlasRenderContract,
    pub(crate) primitive_topology: GlyphAtlasGpuPrimitiveTopology,
    pub(crate) instance_start: u32,
    pub(crate) instance_count: u32,
    pub(crate) atlas_layer: u32,
}

impl GlyphAtlasGpuDrawCommand {
    pub(crate) fn triangle_count(&self) -> u32 {
        self.instance_count * GLYPH_ATLAS_GPU_TRIANGLES_PER_INSTANCE
    }

    pub(crate) fn quad_count(&self) -> u32 {
        self.instance_count
    }

    pub(crate) fn is_quad_aligned(&self) -> bool {
        true
    }
}

pub(crate) fn glyph_atlas_gpu_draw_command(batch: GlyphAtlasGpuBatch) -> GlyphAtlasGpuDrawCommand {
    let primitive_topology = GlyphAtlasGpuPrimitiveTopology::TriangleList;
    let pipeline_key = GlyphAtlasGpuPipelineKey {
        render_contract: batch.key.render_contract,
        primitive_topology,
    };
    GlyphAtlasGpuDrawCommand {
        key: batch.key,
        pipeline_key,
        render_contract: batch.key.render_contract,
        primitive_topology,
        instance_start: batch.instance_start,
        instance_count: batch.instance_count,
        atlas_layer: batch.key.page_key.page_index,
    }
}

pub(crate) fn glyph_atlas_gpu_batch_contract(
    batch: GlyphAtlasGpuBatch,
) -> GlyphAtlasRenderContract {
    batch.key.render_contract
}
