use crate::core::math::UVec2;

use super::render_batch::GlyphAtlasDrawBatchPlan;

mod bind_group;
mod draw_command;
mod instance;
mod pipeline;
mod viewport;

pub(crate) use bind_group::{
    GlyphAtlasGpuBindGroupLayout, GlyphAtlasGpuSamplerBinding, GlyphAtlasGpuSamplerBindingType,
    GlyphAtlasGpuTextureBinding, GlyphAtlasGpuTextureSampleType, GlyphAtlasGpuTextureViewDimension,
    glyph_atlas_gpu_bind_group_layout,
};
pub(crate) use draw_command::{
    GLYPH_ATLAS_GPU_VERTICES_PER_INSTANCE, GlyphAtlasGpuBatch, GlyphAtlasGpuDrawCommand,
    GlyphAtlasGpuPrimitiveTopology, glyph_atlas_gpu_batch_contract, glyph_atlas_gpu_draw_command,
};
pub(crate) use instance::{
    GlyphAtlasGpuInstance, GlyphAtlasGpuInstanceAttribute, GlyphAtlasGpuInstanceAttributeFormat,
    GlyphAtlasGpuInstanceAttributeSemantic, GlyphAtlasGpuInstanceBufferLayout,
    glyph_atlas_gpu_instance_buffer_layout,
};
pub(crate) use pipeline::{
    GlyphAtlasGpuPipelineContract, GlyphAtlasGpuPipelineKey, glyph_atlas_gpu_pipeline_contract,
};
pub(crate) use viewport::{
    GlyphAtlasGpuPixelCoordinateConvention, GlyphAtlasGpuViewportTransform,
    glyph_atlas_gpu_viewport_transform,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphAtlasGpuDrawPlan {
    pub(crate) instance_layout: GlyphAtlasGpuInstanceBufferLayout,
    pub(crate) bind_group_layout: GlyphAtlasGpuBindGroupLayout,
    pub(crate) viewport_transform: GlyphAtlasGpuViewportTransform,
    pub(crate) pipeline_contracts: Vec<GlyphAtlasGpuPipelineContract>,
    pub(crate) batches: Vec<GlyphAtlasGpuBatch>,
    pub(crate) draw_commands: Vec<GlyphAtlasGpuDrawCommand>,
    pub(crate) instances: Vec<GlyphAtlasGpuInstance>,
    pub(crate) visible_glyph_count: usize,
    pub(crate) skipped_glyph_count: usize,
    pub(crate) requires_background_composite: bool,
}

impl Default for GlyphAtlasGpuDrawPlan {
    fn default() -> Self {
        Self {
            instance_layout: glyph_atlas_gpu_instance_buffer_layout(),
            bind_group_layout: glyph_atlas_gpu_bind_group_layout(),
            viewport_transform: GlyphAtlasGpuViewportTransform::default(),
            pipeline_contracts: Vec::new(),
            batches: Vec::new(),
            draw_commands: Vec::new(),
            instances: Vec::new(),
            visible_glyph_count: 0,
            skipped_glyph_count: 0,
            requires_background_composite: false,
        }
    }
}

impl GlyphAtlasGpuDrawPlan {
    pub(crate) fn vertex_count(&self) -> usize {
        self.instances.len() * GLYPH_ATLAS_GPU_VERTICES_PER_INSTANCE as usize
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
    plan.instances.reserve(draw_plan.instance_count);
    plan.batches.reserve(draw_plan.batches.len());
    plan.draw_commands.reserve(draw_plan.batches.len());

    for batch in &draw_plan.batches {
        let instance_start = plan.instances.len() as u32;
        plan.instances.extend(
            batch
                .instances
                .iter()
                .map(|instance| GlyphAtlasGpuInstance {
                    screen_rect_px: [
                        instance.screen_rect.x,
                        instance.screen_rect.y,
                        instance.screen_rect.width,
                        instance.screen_rect.height,
                    ],
                    uv_rect: [
                        instance.uv_rect.x0,
                        instance.uv_rect.y0,
                        instance.uv_rect.x1,
                        instance.uv_rect.y1,
                    ],
                    foreground_color: instance.foreground_color,
                    background_color: instance.background_color,
                    page_index: instance.page_key.page_index,
                }),
        );
        let instance_count = plan.instances.len() as u32 - instance_start;
        if instance_count > 0 {
            let gpu_batch = GlyphAtlasGpuBatch {
                key: batch.key,
                instance_start,
                instance_count,
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
