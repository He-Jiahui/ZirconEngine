use crate::core::math::UVec2;
use crate::text::atlas::render_gpu_plan::{GlyphAtlasGpuDrawCommand, GlyphAtlasGpuPipelineKey};
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};

use super::resources::GlyphAtlasBitmapAtlasResources;

pub(super) struct GlyphAtlasBitmapRendererAtlasResource {
    pub(super) atlas_format: GlyphAtlasFormat,
    pub(super) atlas: GlyphAtlasBitmapAtlasResources,
}

pub(super) struct GlyphAtlasBitmapRendererDrawPass {
    pub(super) instance_buffer: Option<wgpu::Buffer>,
    pub(super) instance_buffer_capacity_bytes: u64,
    pub(super) instance_buffer_payload_hash: Option<[u8; 32]>,
    pub(super) draw_commands: Vec<GlyphAtlasGpuDrawCommand>,
}

pub(super) struct GlyphAtlasBitmapPipelineResource {
    pub(super) key: GlyphAtlasGpuPipelineKey,
    pub(super) pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRendererPrepareReport {
    pub(super) atlas_size: UVec2,
    pub(super) atlas_layer_count: u32,
    pub(super) atlas_storage_format: GlyphAtlasStorageFormat,
    pub(super) storage_pass_count: usize,
    /// Cross-UI prepare reporting projects one glyph-atlas instance per visible glyph.
    pub(in crate::graphics::scene::scene_renderer::ui) storage_pass_visible_glyph_count: usize,
    pub(super) mixed_atlas_storage_format: bool,
    /// Unique atlas resources held by the canonical frame plan.
    pub(super) storage_resource_count: usize,
    /// Format transitions required to replay draw commands in painter order.
    pub(super) ordered_draw_segment_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) atlas_resized: bool,
    pub(super) vertex_count: usize,
    pub(super) vertex_buffer_byte_len: usize,
    pub(super) instance_buffer_capacity_byte_len: usize,
    pub(super) instance_buffer_reallocation_count: usize,
    /// Cross-UI prepare reporting exposes the final painter-order draw command count.
    pub(in crate::graphics::scene::scene_renderer::ui) draw_command_count: usize,
    pub(super) pipeline_count: usize,
    pub(super) requires_background_composite: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_plan_build_count: usize,
    pub(super) upload_plan_skip_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_request_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_requeued_count: usize,
    pub(super) upload_missing_page_requeue_count: usize,
    pub(super) upload_page_generation_mismatch_requeue_count: usize,
    pub(super) upload_face_invalidated_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_byte_len: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_ready_to_write_texture: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_failure_count: usize,
    pub(super) invalidated_storage_pass_count: usize,
}

impl Default for GlyphAtlasBitmapRendererPrepareReport {
    fn default() -> Self {
        Self {
            atlas_size: UVec2::new(1, 1),
            atlas_layer_count: 1,
            atlas_storage_format: GlyphAtlasStorageFormat::R8Unorm,
            storage_pass_count: 0,
            storage_pass_visible_glyph_count: 0,
            mixed_atlas_storage_format: false,
            storage_resource_count: 0,
            ordered_draw_segment_count: 0,
            atlas_resized: false,
            vertex_count: 0,
            vertex_buffer_byte_len: 0,
            instance_buffer_capacity_byte_len: 0,
            instance_buffer_reallocation_count: 0,
            draw_command_count: 0,
            pipeline_count: 0,
            requires_background_composite: false,
            upload_plan_build_count: 0,
            upload_plan_skip_count: 0,
            upload_request_count: 0,
            upload_requeued_count: 0,
            upload_missing_page_requeue_count: 0,
            upload_page_generation_mismatch_requeue_count: 0,
            upload_face_invalidated_count: 0,
            upload_byte_len: 0,
            upload_ready_to_write_texture: false,
            upload_failure_count: 0,
            invalidated_storage_pass_count: 0,
        }
    }
}

impl GlyphAtlasBitmapRendererAtlasResource {
    pub(super) fn new(
        atlas_format: GlyphAtlasFormat,
        atlas: GlyphAtlasBitmapAtlasResources,
    ) -> Self {
        Self {
            atlas_format,
            atlas,
        }
    }
}

impl GlyphAtlasBitmapRendererDrawPass {
    pub(super) fn new() -> Self {
        Self {
            instance_buffer: None,
            instance_buffer_capacity_bytes: 0,
            instance_buffer_payload_hash: None,
            draw_commands: Vec::new(),
        }
    }
}
