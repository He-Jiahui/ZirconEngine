use crate::core::framework::render::{
    AntiAliasMode, PostProcessGraphResourceNames, RenderCapabilitySummary,
    RenderPluginRendererOutputs,
};
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{insert_marker, RENDERDOC_MARKER_FRAME_EXTRACT};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderPassExecutorRegistry,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::{
    build_mesh_pass_command_buffers_cached, MeshDrawReplayStatsAccumulator,
    MeshPassIndirectDrawExecutions,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::sprite::prepare_sprite_queue_stats;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::CompiledRenderPipeline;
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::super::scene_renderer_core::{
    merge_plugin_renderer_outputs, SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};
use super::super::SceneRendererCompiledSceneOutputs;
use super::assign_execution_owned_indirect_args::assign_execution_owned_indirect_args;
use super::bind_compiled_scene_graph_resources::{
    bind_compiled_scene_graph_resources, CompiledSceneGraphResourceBindingFlags,
};
use super::build_compiled_scene_draws::build_compiled_scene_draws;
use super::execute_compiled_scene_graph_stages::CompiledSceneGraphStageContext;
use super::execute_graph_stage::RenderGraphStageExecution;
use super::prepare_overlay_buffers::prepare_overlay_buffers;
use super::submit_compiled_scene_frame::CompiledSceneFrameSubmissionContext;

const SPRITE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Opaque2d,
    RenderPassStage::AlphaMask2d,
    RenderPassStage::Transparent2d,
];

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_compiled_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        target: &mut OffscreenTarget,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        render_pass_executors: &RenderPassExecutorRegistry,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Result<SceneRendererCompiledSceneOutputs, GraphicsError> {
        render_pass_executors
            .validate_compiled_pipeline(pipeline)
            .map_err(GraphicsError::Asset)?;
        self.write_scene_uniform(queue, frame);
        let shadow_frame_plan =
            crate::graphics::scene::scene_renderer::shadow::build_shadow_frame_plan(
                &mut self.shadow_atlas_allocator,
                frame,
                self.shadow_atlas_resources.config(),
            );
        let _shadow_atlas_upload_report = self
            .shadow_atlas_resources
            .upload_frame(
                queue,
                shadow_frame_plan.slots(),
                shadow_frame_plan.globals(),
            )
            .map_err(GraphicsError::Asset)?;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        insert_marker(&mut encoder, RENDERDOC_MARKER_FRAME_EXTRACT);
        let frame_generation = self.mesh_command_generation;
        let mut compiled_scene_draws = build_compiled_scene_draws(
            &self.advanced_plugin_resources,
            device,
            queue,
            &mut encoder,
            &self.material_texture_bind_group_layout,
            &mut self.gpu_scene,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
            Some(shadow_frame_plan.light_slots()),
            &mut self.cached_mesh_draw_commands,
            &mut self.mesh_pipelines,
            frame_generation,
            frame.shader_quality(),
        );
        let _execution_args_buffer = assign_execution_owned_indirect_args(
            device,
            &mut encoder,
            compiled_scene_draws.draws_mut(),
            runtime_features.deferred_lighting_enabled,
        );
        let mut mesh_pass_command_buffers =
            compiled_scene_draws.prebuilt_mesh_pass_command_buffers();
        let residual_mesh_pass_command_buffers = build_mesh_pass_command_buffers_cached(
            compiled_scene_draws.draws(),
            &mut self.mesh_pipelines,
            &mut self.cached_mesh_draw_commands,
            frame_generation,
            frame.shader_quality(),
        );
        mesh_pass_command_buffers.extend(residual_mesh_pass_command_buffers);
        self.cached_mesh_draw_commands
            .retain_generation(frame_generation);
        self.mesh_command_generation = self.mesh_command_generation.wrapping_add(1);
        let mesh_pass_command_stats =
            mesh_pass_command_buffers.stats_with_indirect_batches(capabilities);
        let mut mesh_pass_indirect_draws =
            MeshPassIndirectDrawExecutions::build(device, capabilities, &mesh_pass_command_buffers);
        mesh_pass_indirect_draws.attach_visible_remap_scene_bind_groups(device, &self.gpu_scene);
        let prepared_mesh_queue_stats = compiled_scene_draws
            .prepared_mesh_queue_stats()
            .with_pending_command_cache_plan_stats(
                compiled_scene_draws.pending_command_cache_plan_stats(),
            )
            .with_pending_command_cache_extraction_stats(
                compiled_scene_draws.pending_command_cache_extraction_stats(),
            )
            .with_mesh_pass_command_buffer_stats(mesh_pass_command_stats);
        debug_assert_eq!(
            prepared_mesh_queue_stats.draw_count,
            compiled_scene_draws.draws().len()
                + prepared_mesh_queue_stats.pre_mesh_draw_static_command_cache_skipped_draw_count
        );
        // Draw counts are the extracted source census; command counts are pruned by visibility
        // and per-phase relevance before submission.
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
            prepared_mesh_queue_stats.transparent_command_count
                <= prepared_mesh_queue_stats.transparent_draw_count
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
        let mesh_draw_replay_stats = MeshDrawReplayStatsAccumulator::default();
        let gpu_scene_bind_group = self.gpu_scene.scene_bind_group().clone();
        let gpu_scene_bind_handle =
            crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshSceneDataBindHandle::new(
                &gpu_scene_bind_group,
            );
        let mesh_draw_lists =
            crate::graphics::scene::scene_renderer::graph_execution::RenderPassMeshCommandLists {
                replay_stats: &mesh_draw_replay_stats,
                gpu_scene_bind_group: Some(gpu_scene_bind_handle),
                depth_prepass_commands: mesh_pass_command_buffers.depth_prepass().commands(),
                shadow_commands: mesh_pass_command_buffers.shadow().commands(),
                opaque_commands: mesh_pass_command_buffers.opaque().commands(),
                alpha_mask_commands: mesh_pass_command_buffers.alpha_mask().commands(),
                transparent_commands: mesh_pass_command_buffers.transparent().commands(),
                velocity_commands: mesh_pass_command_buffers.velocity().commands(),
                taa_reactive_mask_commands: mesh_pass_command_buffers
                    .taa_reactive_mask()
                    .commands(),
                depth_prepass_indirect: mesh_pass_indirect_draws.depth_prepass(),
                shadow_indirect: mesh_pass_indirect_draws.shadow(),
                opaque_indirect: mesh_pass_indirect_draws.opaque(),
                alpha_mask_indirect: mesh_pass_indirect_draws.alpha_mask(),
                transparent_indirect: mesh_pass_indirect_draws.transparent(),
                velocity_indirect: mesh_pass_indirect_draws.velocity(),
                taa_reactive_mask_indirect: mesh_pass_indirect_draws.taa_reactive_mask(),
            };
        let prepared_overlays = prepare_overlay_buffers(self, device, queue, streamer, frame)?;
        let material_gbuffer_valid =
            pipeline_writes_resource(pipeline, PostProcessGraphResourceNames::GBUFFER_MATERIAL);
        let history_textures_present = history_textures.is_some();
        let taa_history_enabled = history_textures_present
            && frame.extract.view.anti_alias.mode == AntiAliasMode::Taa
            && pipeline_writes_resource(
                pipeline,
                PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
            );
        let screen_space_reflection_history_enabled = runtime_features.temporal_history_enabled
            && frame
                .extract
                .post_process
                .effect_stack
                .screen_space_reflection
                .is_enabled()
            && pipeline_writes_resource(
                pipeline,
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            );
        let hzb_history_enabled =
            pipeline_writes_resource(pipeline, PostProcessGraphResourceNames::HZB_FURTHEST);
        let exposure_history_enabled =
            pipeline_writes_resource(pipeline, PostProcessGraphResourceNames::EXPOSURE_CURRENT);

        let advanced_plugin_readbacks =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, streamer, frame)?;
        let mut graph_resources = RenderGraphExecutionResources::new();
        self.transient_resource_pool.begin_frame();
        let final_target_output = bind_compiled_scene_graph_resources(
            device,
            pipeline,
            streamer,
            frame,
            target,
            history_textures.as_deref(),
            CompiledSceneGraphResourceBindingFlags {
                taa_history_enabled,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                history_available,
                runtime_features,
            },
            &mut graph_resources,
            &mut self.transient_resource_pool,
            mesh_draw_lists,
            self.hzb_occlusion_culler.as_ref(),
            &self.shadow_atlas_resources,
            advanced_plugin_readbacks.external_buffer_bindings(),
        )?;
        let materialization_report = graph_resources
            .validate_materialized_graph_resources(&pipeline.graph)
            .map_err(GraphicsError::Asset)?;
        let mut graph_execution_record = RenderGraphExecutionRecord::default();
        graph_execution_record.set_materialization_report(materialization_report);
        graph_execution_record.set_resource_report(graph_resources.resource_report());
        graph_execution_record.set_resource_alias_report(graph_resources.resource_alias_report());
        let mut graph_plugin_outputs = RenderPluginRendererOutputs::default();
        let mut graph_execution = RenderGraphStageExecution::new(
            &mut graph_resources,
            &mut graph_execution_record,
            &mut graph_plugin_outputs,
        );
        self.execute_compiled_scene_graph_stages(CompiledSceneGraphStageContext {
            device,
            queue,
            encoder: &mut encoder,
            streamer,
            frame,
            target,
            pipeline,
            render_pass_executors,
            runtime_features,
            graph_execution: &mut graph_execution,
            mesh_draw_lists,
            history_textures,
            history_available,
            material_gbuffer_valid,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
            shadow_frame_plan: &shadow_frame_plan,
            prepared_overlays: &prepared_overlays,
        })?;
        drop(graph_execution);

        self.submit_compiled_scene_frame(CompiledSceneFrameSubmissionContext {
            device,
            queue,
            encoder,
            streamer,
            frame,
            graph_resources: &mut graph_resources,
            graph_execution_record: &mut graph_execution_record,
            mesh_pass_indirect_draws: &mesh_pass_indirect_draws,
        });

        let mut renderer_outputs = advanced_plugin_readbacks.into_outputs();
        merge_plugin_renderer_outputs(&mut renderer_outputs, graph_plugin_outputs);
        let prepared_mesh_queue_stats =
            prepared_mesh_queue_stats.with_mesh_draw_replay_stats(mesh_draw_replay_stats.stats());
        let prepared_mesh_queue_stats = prepared_mesh_queue_stats.with_gpu_scene_stats(
            self.gpu_scene.stats(),
            compiled_scene_draws.gpu_scene_upload_report(),
        );
        let prepared_mesh_queue_stats = prepared_mesh_queue_stats
            .with_virtual_geometry_execution_stats(
                compiled_scene_draws.virtual_geometry_execution_stats(),
            )
            .with_virtual_geometry_indirect_stats(
                compiled_scene_draws.virtual_geometry_indirect_stats(),
            );
        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::from_outputs(renderer_outputs),
            graph_execution_record,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
        );
        let _prev_transform_roll_report = self.gpu_scene.roll_prev_transforms_after_success();
        let _prev_skinned_palette_roll_report =
            self.gpu_scene.roll_prev_skinned_palettes_after_success();
        let _prev_skinned_source_roll_report =
            self.gpu_scene.roll_prev_skinned_gpu_sources_after_success();
        Ok(match final_target_output.graph_import_report {
            Some(report) => outputs.with_output_target_graph_import_report(report),
            None => outputs,
        })
    }
}

fn active_sprite_graph_stages(pipeline: &CompiledRenderPipeline) -> Vec<RenderPassStage> {
    SPRITE_GRAPH_STAGES
        .iter()
        .copied()
        .filter(|stage| pipeline_has_active_sprite_stage(pipeline, *stage))
        .collect()
}

fn pipeline_has_active_sprite_stage(
    pipeline: &CompiledRenderPipeline,
    stage: RenderPassStage,
) -> bool {
    pipeline
        .pass_stages
        .iter()
        .filter(|stage_entry| stage_entry.stage == stage)
        .any(|stage_entry| {
            pipeline.graph.passes().iter().any(|pass| {
                pass.name == stage_entry.pass_name
                    && !pass.culled
                    && pass
                        .executor_id
                        .as_deref()
                        .is_some_and(|executor_id| executor_id.starts_with("sprite."))
            })
        })
}

fn pipeline_writes_resource(pipeline: &CompiledRenderPipeline, resource_name: &str) -> bool {
    pipeline
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .flat_map(|pass| pass.resources.iter())
        .any(|resource| {
            resource.name == resource_name
                && resource.access == RenderGraphResourceAccessKind::Write
        })
}

#[cfg(test)]
mod tests {
    use super::{active_sprite_graph_stages, SPRITE_GRAPH_STAGES};
    use crate::core::framework::render::RenderPipelineHandle;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
    use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};

    #[test]
    fn compiled_scene_sprite_stage_list_owns_core2d_product_stages() {
        assert!(SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Opaque2d));
        assert!(SPRITE_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask2d));
        assert!(SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
        assert!(!SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Deferred));
        assert!(!SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Lighting));
        assert!(!SPRITE_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask3d));
    }

    #[test]
    fn active_sprite_graph_stages_follow_unculled_sprite_passes() {
        let pipeline = compiled_pipeline_with_passes([
            (RenderPassStage::Opaque2d, "sprite-opaque", "sprite.opaque"),
            (
                RenderPassStage::Transparent2d,
                "sprite-transparent",
                "sprite.transparent",
            ),
            (RenderPassStage::Ui, "runtime-ui", "ui.screen-space"),
        ]);

        assert_eq!(
            active_sprite_graph_stages(&pipeline),
            vec![RenderPassStage::Opaque2d, RenderPassStage::Transparent2d]
        );
    }

    fn compiled_pipeline_with_passes<const N: usize>(
        passes: [(RenderPassStage, &str, &str); N],
    ) -> CompiledRenderPipeline {
        let mut graph = RenderGraphBuilder::new("sprite-stage-test");
        let mut pass_stages = Vec::new();
        for (stage, pass_name, executor_id) in passes {
            let pass =
                graph.add_pass_with_executor(pass_name, QueueLane::Graphics, Some(executor_id));
            // This fixture tests stage filtering only, so synthetic passes are rooted directly.
            graph
                .set_pass_flags(
                    pass,
                    PassFlags {
                        has_side_effects: true,
                        ..PassFlags::default()
                    },
                )
                .expect("sprite stage test root");
            pass_stages.push(CompiledRenderPipelinePassStage::new(pass_name, stage));
        }

        CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(99),
            name: "sprite-stage-test".to_string(),
            renderer_name: "sprite-stage-test".to_string(),
            stages: SPRITE_GRAPH_STAGES.to_vec(),
            pass_stages,
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph: graph.compile().expect("sprite stage test graph"),
        }
    }
}
