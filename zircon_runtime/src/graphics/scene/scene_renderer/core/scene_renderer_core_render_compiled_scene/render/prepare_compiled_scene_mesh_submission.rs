use crate::core::TaskPool;
use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::GpuScenePreparedUpload;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassMeshCommandLists;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshIndirectWorkspacePreparedUpload, MeshPassIndirectDrawExecutions, MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::{
    MaterialPipelineFeatureSet, MeshDrawReplayStatsAccumulator, MeshPassCommandBuffers,
    MeshPassIndirectDrawPlans, PreparedMeshQueueStats, build_mesh_pass_command_buffers_cached,
    build_mesh_pass_command_buffers_cached_parallel,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::shadow::ShadowFramePlan;
use crate::graphics::scene::scene_renderer::sprite::{
    PreparedSpriteQueueStats, prepare_sprite_queue_stats,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::assign_execution_owned_indirect_args::assign_execution_owned_indirect_args;
use super::build_compiled_scene_draws::{CompiledSceneDraws, build_compiled_scene_draws};
use super::frame_lifecycle::RenderGenerationIds;
use super::sprite_stage_selection::active_sprite_graph_stages;

pub(super) struct PreparedCompiledSceneMeshSubmission {
    pub(super) compiled_scene_draws: CompiledSceneDraws,
    pub(super) gpu_scene_prepared_upload: GpuScenePreparedUpload,
    pub(super) mesh_pass_command_buffers: MeshPassCommandBuffers,
    pub(super) mesh_pass_indirect_draws: MeshPassIndirectDrawExecutions,
    pub(super) mesh_indirect_prepared_upload: MeshIndirectWorkspacePreparedUpload,
    pub(super) prepared_mesh_queue_stats: PreparedMeshQueueStats,
    pub(super) prepared_sprite_queue_stats: PreparedSpriteQueueStats,
    pub(super) mesh_draw_replay_stats: MeshDrawReplayStatsAccumulator,
}

pub(super) fn project_compiled_scene_mesh_draw_lists<'a>(
    replay_stats: &'a MeshDrawReplayStatsAccumulator,
    gpu_scene_bind_group: &'a wgpu::BindGroup,
    command_buffers: &'a MeshPassCommandBuffers,
    indirect_draws: &'a MeshPassIndirectDrawExecutions,
    transmission_step_count: usize,
) -> RenderPassMeshCommandLists<'a> {
    RenderPassMeshCommandLists {
        replay_stats,
        gpu_scene_bind_group: Some(MeshSceneDataBindHandle::new(gpu_scene_bind_group)),
        depth_prepass_commands: command_buffers.depth_prepass().commands(),
        shadow_commands: command_buffers.shadow().commands(),
        opaque_commands: command_buffers.opaque().commands(),
        alpha_mask_commands: command_buffers.alpha_mask().commands(),
        advanced_pbr_opaque_commands: command_buffers.advanced_pbr_opaque().commands(),
        transmission_commands: command_buffers.transmission().commands(),
        transmission_step_count,
        transparent_commands: command_buffers.transparent().commands(),
        half_resolution_transparent_commands: command_buffers
            .half_resolution_transparent()
            .commands(),
        velocity_commands: command_buffers.velocity().commands(),
        taa_reactive_mask_commands: command_buffers.taa_reactive_mask().commands(),
        depth_prepass_indirect: indirect_draws.depth_prepass(),
        shadow_indirect: indirect_draws.shadow(),
        opaque_indirect: indirect_draws.opaque(),
        alpha_mask_indirect: indirect_draws.alpha_mask(),
        advanced_pbr_opaque_indirect: indirect_draws.advanced_pbr_opaque(),
        transparent_indirect: indirect_draws.transparent(),
        half_resolution_transparent_indirect: indirect_draws.half_resolution_transparent(),
        velocity_indirect: indirect_draws.velocity(),
        taa_reactive_mask_indirect: indirect_draws.taa_reactive_mask(),
    }
}

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_compiled_scene_mesh_submission(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &mut ResourceStreamer,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        runtime_features: SceneRuntimeFeatureFlags,
        material_pipeline_features: MaterialPipelineFeatureSet,
        shadow_frame_plan: &ShadowFramePlan,
        generation_ids: RenderGenerationIds,
        compute_task_pool: Option<&TaskPool>,
        frame_buffer_uploads: &mut WgpuBufferUploadBatch,
    ) -> Result<PreparedCompiledSceneMeshSubmission, GraphicsError> {
        let device = &backend.device;
        let mut compiled_scene_draws = build_compiled_scene_draws(
            &self.advanced_plugin_resources,
            backend,
            encoder,
            &self.material_texture_bind_group_layout,
            &mut self.gpu_scene,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
            material_pipeline_features,
            Some(shadow_frame_plan.light_slots()),
            &mut self.cached_mesh_draw_commands,
            &mut self.mesh_pipelines,
            generation_ids.mesh_commands,
            frame.shader_quality(),
        )?;
        let mut gpu_scene_prepared_upload =
            compiled_scene_draws.take_gpu_scene_prepared_upload()?;
        gpu_scene_prepared_upload.append_to(&self.gpu_scene, frame_buffer_uploads);
        let material_pipeline_requirements =
            compiled_scene_draws.take_material_pipeline_requirements();
        crate::graphics::scene::scene_renderer::mesh::coordinate_material_pipeline_publications(
            device,
            streamer,
            &mut self.mesh_pipelines,
            material_pipeline_requirements,
            frame
                .camera_stack_output_policy()
                .starts_viewport_submission(),
            frame
                .camera_stack_output_policy()
                .owns_viewport_submission(),
        );
        let _execution_args_buffer = assign_execution_owned_indirect_args(
            device,
            encoder,
            compiled_scene_draws.draws_mut(),
            runtime_features.deferred_lighting_enabled,
        );
        let mut mesh_pass_command_buffers =
            compiled_scene_draws.prebuilt_mesh_pass_command_buffers();
        let residual_mesh_pass_command_buffers = if let Some(task_pool) = compute_task_pool {
            build_mesh_pass_command_buffers_cached_parallel(
                compiled_scene_draws.draws(),
                &mut self.mesh_pipelines,
                &mut self.cached_mesh_draw_commands,
                generation_ids.mesh_commands,
                frame.shader_quality(),
                task_pool,
            )
        } else {
            build_mesh_pass_command_buffers_cached(
                compiled_scene_draws.draws(),
                &mut self.mesh_pipelines,
                &mut self.cached_mesh_draw_commands,
                generation_ids.mesh_commands,
                frame.shader_quality(),
            )
        };
        mesh_pass_command_buffers.extend(residual_mesh_pass_command_buffers);
        self.cached_mesh_draw_commands
            .retain_generation(generation_ids.mesh_commands);
        self.mesh_command_generation = self.mesh_command_generation.wrapping_add(1);
        let half_resolution_mesh_pass_available = pipeline.graph().passes().iter().any(|pass| {
            pass.executor_id.as_deref() == Some(HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID)
        });
        if !half_resolution_mesh_pass_available {
            // Preserve material-marked transparent meshes on profile, MSAA, and plugin fallbacks.
            mesh_pass_command_buffers.merge_half_resolution_transparent_into_transparent();
        }
        let mesh_pass_indirect_plans =
            MeshPassIndirectDrawPlans::build(&mesh_pass_command_buffers, capabilities);
        let mesh_pass_command_stats =
            mesh_pass_command_buffers.stats_with_indirect_plan(mesh_pass_indirect_plans.stats());
        let (
            mut mesh_pass_indirect_draws,
            mesh_indirect_workspace_stats,
            mut mesh_indirect_prepared_upload,
        ) = self.mesh_indirect_draw_workspace.prepare(
            device,
            capabilities,
            mesh_pass_indirect_plans,
        );
        mesh_indirect_prepared_upload.append_to(frame_buffer_uploads);
        mesh_pass_indirect_draws.attach_visible_remap_scene_bind_groups(device, &self.gpu_scene);
        let prepared_mesh_queue_stats = compiled_scene_draws
            .prepared_mesh_queue_stats()
            .with_pending_command_cache_plan_stats(
                compiled_scene_draws.pending_command_cache_plan_stats(),
            )
            .with_pending_command_cache_extraction_stats(
                compiled_scene_draws.pending_command_cache_extraction_stats(),
            )
            .with_mesh_pass_command_buffer_stats(mesh_pass_command_stats)
            .with_indirect_workspace_stats(mesh_indirect_workspace_stats);
        debug_assert_eq!(
            prepared_mesh_queue_stats.draw_count,
            compiled_scene_draws.draws().len()
                + prepared_mesh_queue_stats.pre_mesh_draw_static_command_cache_skipped_draw_count
        );
        // Draw counts are the extracted source census; command counts are pruned by visibility.
        debug_assert!(
            prepared_mesh_queue_stats.depth_prepass_command_count
                <= prepared_mesh_queue_stats.early_z_draw_count
        );
        debug_assert!(
            prepared_mesh_queue_stats.shadow_command_count
                <= prepared_mesh_queue_stats.shadow_caster_draw_count
        );
        debug_assert!(
            prepared_mesh_queue_stats.opaque_command_count
                <= prepared_mesh_queue_stats.opaque_draw_count
        );
        debug_assert!(
            prepared_mesh_queue_stats.alpha_mask_command_count
                <= prepared_mesh_queue_stats.alpha_mask_draw_count
        );
        debug_assert!(
            prepared_mesh_queue_stats
                .transparent_command_count
                .saturating_add(prepared_mesh_queue_stats.transmission_command_count)
                <= prepared_mesh_queue_stats.transparent_draw_count
        );
        debug_assert!(
            prepared_mesh_queue_stats.advanced_pbr_opaque_command_count
                <= prepared_mesh_queue_stats.opaque_draw_count
        );
        debug_assert_eq!(
            prepared_mesh_queue_stats.velocity_command_count,
            mesh_pass_command_buffers.velocity().commands().len()
        );
        debug_assert_eq!(
            prepared_mesh_queue_stats.taa_reactive_mask_command_count,
            mesh_pass_command_buffers
                .taa_reactive_mask()
                .commands()
                .len()
        );
        let prepared_sprite_queue_stats = runtime_features
            .sprite_rendering_enabled
            .then(|| prepare_sprite_queue_stats(frame, active_sprite_graph_stages(pipeline)))
            .unwrap_or_default();

        Ok(PreparedCompiledSceneMeshSubmission {
            compiled_scene_draws,
            gpu_scene_prepared_upload,
            mesh_pass_command_buffers,
            mesh_pass_indirect_draws,
            mesh_indirect_prepared_upload,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
            mesh_draw_replay_stats: MeshDrawReplayStatsAccumulator::default(),
        })
    }
}
