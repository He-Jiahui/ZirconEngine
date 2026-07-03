use crate::core::math::UVec2;

use super::render_batch::GlyphAtlasDrawBatchPlan;

mod bind_group;
mod draw_command;
mod pipeline;
mod vertex;
mod viewport;

pub(crate) use bind_group::{
    glyph_atlas_gpu_bind_group_layout, GlyphAtlasGpuBindGroupLayout, GlyphAtlasGpuSamplerBinding,
    GlyphAtlasGpuSamplerBindingType, GlyphAtlasGpuTextureBinding, GlyphAtlasGpuTextureSampleType,
    GlyphAtlasGpuTextureViewDimension,
};
pub(crate) use draw_command::{
    glyph_atlas_gpu_batch_contract, glyph_atlas_gpu_draw_command, GlyphAtlasGpuBatch,
    GlyphAtlasGpuDrawCommand, GlyphAtlasGpuPrimitiveTopology,
};
pub(crate) use pipeline::{
    glyph_atlas_gpu_pipeline_contract, GlyphAtlasGpuPipelineContract, GlyphAtlasGpuPipelineKey,
};
pub(crate) use vertex::{
    glyph_atlas_gpu_vertex_buffer_layout, GlyphAtlasGpuVertex, GlyphAtlasGpuVertexAttribute,
    GlyphAtlasGpuVertexAttributeFormat, GlyphAtlasGpuVertexAttributeSemantic,
    GlyphAtlasGpuVertexBufferLayout,
};
pub(crate) use viewport::{
    glyph_atlas_gpu_viewport_transform, GlyphAtlasGpuPixelCoordinateConvention,
    GlyphAtlasGpuViewportTransform,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphAtlasGpuDrawPlan {
    pub(crate) vertex_layout: GlyphAtlasGpuVertexBufferLayout,
    pub(crate) bind_group_layout: GlyphAtlasGpuBindGroupLayout,
    pub(crate) viewport_transform: GlyphAtlasGpuViewportTransform,
    pub(crate) pipeline_contracts: Vec<GlyphAtlasGpuPipelineContract>,
    pub(crate) batches: Vec<GlyphAtlasGpuBatch>,
    pub(crate) draw_commands: Vec<GlyphAtlasGpuDrawCommand>,
    pub(crate) vertices: Vec<GlyphAtlasGpuVertex>,
    pub(crate) visible_glyph_count: usize,
    pub(crate) skipped_glyph_count: usize,
    pub(crate) requires_background_composite: bool,
}

impl Default for GlyphAtlasGpuDrawPlan {
    fn default() -> Self {
        Self {
            vertex_layout: glyph_atlas_gpu_vertex_buffer_layout(),
            bind_group_layout: glyph_atlas_gpu_bind_group_layout(),
            viewport_transform: GlyphAtlasGpuViewportTransform::default(),
            pipeline_contracts: Vec::new(),
            batches: Vec::new(),
            draw_commands: Vec::new(),
            vertices: Vec::new(),
            visible_glyph_count: 0,
            skipped_glyph_count: 0,
            requires_background_composite: false,
        }
    }
}

impl GlyphAtlasGpuDrawPlan {
    pub(crate) fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

pub(crate) fn glyph_atlas_gpu_draw_plan(
    draw_plan: &GlyphAtlasDrawBatchPlan,
    viewport_size: UVec2,
) -> GlyphAtlasGpuDrawPlan {
    let viewport_transform = glyph_atlas_gpu_viewport_transform(viewport_size);
    let mut plan = GlyphAtlasGpuDrawPlan {
        viewport_transform,
        visible_glyph_count: draw_plan.visible_glyph_count,
        skipped_glyph_count: draw_plan.skipped_glyph_count,
        requires_background_composite: draw_plan.requires_background_composite,
        ..GlyphAtlasGpuDrawPlan::default()
    };

    for batch in &draw_plan.batches {
        let vertex_start = plan.vertices.len() as u32;
        for quad in &batch.quads {
            plan.vertices
                .extend(quad.vertices.iter().map(|vertex| GlyphAtlasGpuVertex {
                    position_ndc: viewport_transform.position_ndc(vertex.position_px),
                    uv: vertex.uv,
                    foreground_color: vertex.foreground_color,
                    background_color: vertex.background_color,
                    page_index: vertex.page_index,
                }));
        }
        let vertex_count = plan.vertices.len() as u32 - vertex_start;
        if vertex_count > 0 {
            let gpu_batch = GlyphAtlasGpuBatch {
                key: batch.key,
                vertex_start,
                vertex_count,
            };
            let draw_command = glyph_atlas_gpu_draw_command(gpu_batch);
            let pipeline_contract = glyph_atlas_gpu_pipeline_contract(draw_command.pipeline_key);
            if !plan
                .pipeline_contracts
                .iter()
                .any(|contract| contract.key == pipeline_contract.key)
            {
                plan.pipeline_contracts.push(pipeline_contract);
            }
            plan.draw_commands.push(draw_command);
            plan.batches.push(gpu_batch);
        }
    }

    plan
}
#[cfg(test)]
mod tests;
