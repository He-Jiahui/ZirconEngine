use crate::core::math::UVec2;
use crate::text::atlas::render_gpu_plan::{GlyphAtlasGpuDrawCommand, GlyphAtlasGpuPipelineKey};
use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapRenderSubmissionPlan,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat, GlyphAtlasStorageFormat,
};

use super::resources::GlyphAtlasBitmapAtlasResources;

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRendererStorageSubmission<
    'a,
> {
    pub(super) submission: &'a GlyphAtlasBitmapRenderSubmissionPlan,
    pub(super) source_bytes: Vec<GlyphAtlasBitmapUploadSourceBytes<'a>>,
    pub(super) atlas_layer_count: u32,
    pub(super) atlas_format: GlyphAtlasFormat,
    pub(super) face_validity: GlyphAtlasBitmapFaceValidity,
}

pub(super) struct GlyphAtlasBitmapRendererAtlasResource {
    pub(super) atlas_format: GlyphAtlasFormat,
    pub(super) atlas: GlyphAtlasBitmapAtlasResources,
}

pub(super) struct GlyphAtlasBitmapRendererDrawPass {
    pub(super) atlas_format: GlyphAtlasFormat,
    pub(super) instance_buffer: Option<wgpu::Buffer>,
    pub(super) instance_buffer_capacity_bytes: u64,
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
    pub(super) storage_pass_visible_glyph_count: usize,
    pub(super) mixed_atlas_storage_format: bool,
    pub(super) atlas_resized: bool,
    pub(super) vertex_count: usize,
    pub(super) vertex_buffer_byte_len: usize,
    pub(super) instance_buffer_capacity_byte_len: usize,
    pub(super) instance_buffer_reallocation_count: usize,
    pub(super) draw_command_count: usize,
    pub(super) pipeline_count: usize,
    pub(super) requires_background_composite: bool,
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
            atlas_resized: false,
            vertex_count: 0,
            vertex_buffer_byte_len: 0,
            instance_buffer_capacity_byte_len: 0,
            instance_buffer_reallocation_count: 0,
            draw_command_count: 0,
            pipeline_count: 0,
            requires_background_composite: false,
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

impl<'a> GlyphAtlasBitmapRendererStorageSubmission<'a> {
    pub(in crate::graphics::scene::scene_renderer::ui) fn new(
        submission: &'a GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: Vec<GlyphAtlasBitmapUploadSourceBytes<'a>>,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
    ) -> Self {
        Self::new_with_face_validity(
            submission,
            source_bytes,
            atlas_layer_count,
            atlas_format,
            GlyphAtlasBitmapFaceValidity::Valid,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn new_with_face_validity(
        submission: &'a GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: Vec<GlyphAtlasBitmapUploadSourceBytes<'a>>,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
        face_validity: GlyphAtlasBitmapFaceValidity,
    ) -> Self {
        Self {
            submission,
            source_bytes,
            atlas_layer_count,
            atlas_format,
            face_validity,
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
    pub(super) fn new(atlas_format: GlyphAtlasFormat) -> Self {
        Self {
            atlas_format,
            instance_buffer: None,
            instance_buffer_capacity_bytes: 0,
            draw_commands: Vec::new(),
        }
    }
}
