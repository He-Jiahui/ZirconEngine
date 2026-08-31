use crate::core::framework::render::SkyboxMode;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::{OffscreenTarget, RenderBackend};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::environment::RealtimeIblPendingSubmission;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistry;
use crate::graphics::scene::scene_renderer::mesh::MaterialPipelineFeatureSet;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::shadow::ShadowFramePlan;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasPreparedUpload;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::frame_lifecycle::{RenderGenerationIds, ensure_compiled_scene_graph_resources};

pub(super) struct PreparedCompiledSceneFrameFoundation {
    pub(super) encoder: wgpu::CommandEncoder,
    pub(super) frame_texture_uploads: WgpuTextureUploadBatch,
    pub(super) frame_buffer_uploads: WgpuBufferUploadBatch,
    pub(super) shadow_frame_plan: ShadowFramePlan,
    pub(super) shadow_atlas_prepared_upload: ShadowAtlasPreparedUpload,
    pub(super) realtime_ibl_submission: Option<RealtimeIblPendingSubmission>,
    pub(super) generation_ids: RenderGenerationIds,
    pub(super) material_pipeline_features: MaterialPipelineFeatureSet,
}

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_compiled_scene_frame_foundation(
        &mut self,
        backend: &RenderBackend,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        target: &OffscreenTarget,
        pipeline: &CompiledRenderPipeline,
        render_pass_executors: &RenderPassExecutorRegistry,
        runtime_features: SceneRuntimeFeatureFlags,
        frame_generation: u64,
    ) -> Result<PreparedCompiledSceneFrameFoundation, GraphicsError> {
        let device = &backend.device;
        self.mesh_pipelines
            .collect_terminal_pipeline_submissions(|ticket| backend.submission_status(ticket).ok());
        self.mesh_pipelines.begin_submission_usage_recording();
        self.mesh_pipelines
            .begin_forward_receiver_binding_profile_frame();
        self.mesh_pipelines.light_cookies.begin_profile_frame();
        ensure_compiled_scene_graph_resources(
            self.deferred_lighting_profile,
            self.post_process.has_full_resources(),
            self.scene_clear.is_some(),
        )?;
        render_pass_executors
            .validate_compiled_pipeline(pipeline)
            .map_err(GraphicsError::Asset)?;
        let realtime_ibl_prepared = matches!(
            frame.environment().skybox.mode,
            SkyboxMode::ProceduralGradient
        )
        .then_some(frame.environment().skybox.procedural)
        .filter(|sky| sky.intensity > 0.0)
        .map(|sky| self.realtime_ibl.prepare_frame(device, sky));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        crate::graphics::debug_markers::insert_marker(
            &mut encoder,
            crate::graphics::debug_markers::RENDERDOC_MARKER_FRAME_EXTRACT,
        );
        let mut frame_texture_uploads = WgpuTextureUploadBatch::new();
        let mut frame_buffer_uploads = self.write_scene_uniform(
            backend,
            &mut encoder,
            streamer,
            frame,
            realtime_ibl_prepared.as_ref(),
            runtime_features.reflection_probes_enabled,
            &mut frame_texture_uploads,
        )?;
        self.post_process.prepare_exposure_params_upload(
            target.size,
            frame.post_process().exposure,
            frame.extract.timing.raw_real_delta_seconds(),
            &mut frame_buffer_uploads,
        );
        let selected_irradiance_volume =
            super::irradiance_volume_selection::select_frame_irradiance_volume(streamer, frame);
        self.mesh_pipelines
            .irradiance_volume
            .prepare(selected_irradiance_volume, &mut frame_buffer_uploads)
            .map_err(GraphicsError::Asset)?;
        let static_caster_revision = streamer
            .with_ready_resource_revisions(|resource_revision| {
                crate::graphics::scene::scene_renderer::shadow::
                    static_shadow_caster_revision_from_meshes_with_resource_revisions(
                        &frame.extract.geometry.meshes,
                        |resource| resource_revision(resource),
                    )
            })
            .flatten();
        let shadow_frame_plan = crate::graphics::scene::scene_renderer::shadow::
            build_shadow_frame_plan_with_static_caster_revision(
                &mut self.shadow_atlas_allocator,
                frame,
                self.shadow_atlas_resources.config(),
                static_caster_revision,
            );
        let mut shadow_atlas_prepared_upload = self
            .shadow_atlas_resources
            .prepare_frame_upload(shadow_frame_plan.slots(), shadow_frame_plan.globals())
            .map_err(GraphicsError::Asset)?;
        shadow_atlas_prepared_upload.append_to(&mut frame_buffer_uploads);
        if let Some(shadow_map_renderer) = self.shadow_map_renderer.as_mut() {
            let mut shadow_slot_scene_uploads = shadow_map_renderer
                .prepare_slot_scene_uploads(device, shadow_frame_plan.atlas_passes())
                .map_err(GraphicsError::Asset)?;
            frame_buffer_uploads.append(&mut shadow_slot_scene_uploads);
        }
        let realtime_ibl_gpu_timing_enabled = self.realtime_ibl.gpu_timestamps_supported();
        let realtime_ibl_submission = realtime_ibl_prepared
            .as_ref()
            .map(|prepared| {
                self.realtime_ibl.record_prepared_frame(
                    device,
                    &mut encoder,
                    realtime_ibl_gpu_timing_enabled,
                    prepared,
                    &mut self.ibl_bake_pipeline_cache,
                )
            })
            .transpose()
            .map_err(GraphicsError::Asset)?
            .flatten();
        let generation_ids =
            RenderGenerationIds::new(frame_generation, self.mesh_command_generation);
        let material_pipeline_features = MaterialPipelineFeatureSet::from_executor_ids(
            pipeline
                .graph()
                .passes()
                .iter()
                .map(|pass| pass.executor_id.as_deref()),
        );
        Ok(PreparedCompiledSceneFrameFoundation {
            encoder,
            frame_texture_uploads,
            frame_buffer_uploads,
            shadow_frame_plan,
            shadow_atlas_prepared_upload,
            realtime_ibl_submission,
            generation_ids,
            material_pipeline_features,
        })
    }
}
