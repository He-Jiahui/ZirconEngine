#[cfg(test)]
use crate::core::framework::render::RenderSceneVelocityReadbackReport;
use crate::core::framework::render::{
    AntiAliasMode, PostProcessGraphResourceNames, RenderCameraTargetGraphImportReport,
    RenderCapabilitySummary, RenderPluginRendererOutputs,
};
#[cfg(test)]
use crate::graphics::backend::read_texture_rgba;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    insert_marker, pop_group, push_group, RENDERDOC_MARKER_FRAME_EXTRACT,
    RENDERDOC_MARKER_HISTORY_COPY, RENDERDOC_MARKER_POST_PROCESS, RENDERDOC_MARKER_PREPASS,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::{OutputTargetTextureResource, ResourceStreamer};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderGraphImportedFinalTarget,
    RenderPassExecutorRegistry, RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::{
    build_mesh_pass_command_buffers_cached, prepare_mesh_queue, MeshDrawReplayStatsAccumulator,
    MeshIndirectArgsReadback, MeshPassIndirectDrawExecutions,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::sprite::prepare_sprite_queue_stats;
use crate::graphics::types::{
    GraphicsError, ViewportRenderFrame, ViewportTextureGraphImportStatus,
};
use crate::graphics::visibility::{
    HzbOcclusionCullReadbackStats, HzbOcclusionIndirectArgsReadbackSummary,
};
use crate::render_graph::RenderGraphResourceAccessKind;
#[cfg(test)]
use crate::rhi::TextureFormat;
use crate::CompiledRenderPipeline;
use std::sync::Arc;

use super::super::super::scene_renderer_core::{
    merge_plugin_renderer_outputs, SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};
use super::super::SceneRendererCompiledSceneOutputs;
use super::assign_execution_owned_indirect_args::assign_execution_owned_indirect_args;
use super::build_compiled_scene_draws::build_compiled_scene_draws;
use super::execute_graph_stage::{
    execute_graph_stage, import_frame_targets, RenderGraphStageExecution,
};
use super::prepare_overlay_buffers::prepare_overlay_buffers;

const EARLY_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::DepthPrepass,
    RenderPassStage::Shadow,
    RenderPassStage::AmbientOcclusion,
];

const LATE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Ui,
    RenderPassStage::Overlay,
    RenderPassStage::Debug,
];

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
        );
        let _execution_args_buffer = assign_execution_owned_indirect_args(
            device,
            &mut encoder,
            compiled_scene_draws.draws_mut(),
            runtime_features.deferred_lighting_enabled,
        );
        let prepared_mesh_queue = prepare_mesh_queue(compiled_scene_draws.draws());
        let mesh_pass_command_buffers = build_mesh_pass_command_buffers_cached(
            compiled_scene_draws.draws(),
            &mut self.mesh_pipelines,
            &mut self.cached_mesh_draw_commands,
            frame_generation,
        );
        self.cached_mesh_draw_commands
            .retain_generation(frame_generation);
        self.mesh_command_generation = self.mesh_command_generation.wrapping_add(1);
        let mesh_pass_command_stats =
            mesh_pass_command_buffers.stats_with_indirect_batches(capabilities);
        let mut mesh_pass_indirect_draws =
            MeshPassIndirectDrawExecutions::build(device, capabilities, &mesh_pass_command_buffers);
        mesh_pass_indirect_draws.attach_visible_remap_scene_bind_groups(device, &self.gpu_scene);
        let prepared_mesh_queue_stats = prepared_mesh_queue
            .stats()
            .with_mesh_pass_command_buffer_stats(mesh_pass_command_stats);
        debug_assert_eq!(
            prepared_mesh_queue_stats.draw_count,
            compiled_scene_draws.draws().len()
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
        let direct_imported_final_target = direct_imported_final_target(streamer, frame);
        let imported_final_target =
            direct_imported_final_target
                .as_ref()
                .map(|resource| RenderGraphImportedFinalTarget {
                    view: resource.view(),
                });
        import_frame_targets(
            &mut graph_resources,
            target,
            imported_final_target,
            Some(&self.shadow_atlas_resources),
        );
        let direct_import_report = direct_imported_final_target
            .as_ref()
            .map(|resource| RenderCameraTargetGraphImportReport::direct_imported(resource.size()));
        if let Some(history_textures) = history_textures.as_deref() {
            if taa_history_enabled {
                graph_resources.import_texture_view(
                    PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
                    history_textures.taa_scene_color_previous_view(),
                );
                graph_resources.import_texture_view(
                    PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
                    history_textures.taa_scene_color_current_view(),
                );
            }
            if history_available && screen_space_reflection_history_enabled {
                graph_resources.import_texture_view(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
                    history_textures
                        .screen_space_reflection
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
            }
            if history_available && hzb_history_enabled {
                graph_resources.import_texture_view(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
                    history_textures.hzb_furthest_view.clone(),
                );
            }
            if exposure_history_enabled {
                graph_resources.insert_buffer(
                    PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
                    history_textures.exposure_previous_buffer(),
                );
                graph_resources.insert_buffer(
                    PostProcessGraphResourceNames::EXPOSURE_CURRENT,
                    history_textures.exposure_current_buffer(),
                );
            }
        }
        graph_resources
            .materialize_transient_resources_with_pool(
                device,
                &pipeline.graph,
                &mut self.transient_resource_pool,
            )
            .map_err(GraphicsError::Asset)?;
        let mut graph_execution_record = RenderGraphExecutionRecord::default();
        graph_execution_record.set_resource_report(graph_resources.resource_report());
        let mut graph_plugin_outputs = RenderPluginRendererOutputs::default();
        let mut graph_execution = RenderGraphStageExecution::new(
            &mut graph_resources,
            &mut graph_execution_record,
            &mut graph_plugin_outputs,
        );
        let early_post_process_stack = RenderPassPostProcessStackContext::new(
            &self.post_process,
            &*target,
            streamer,
            runtime_features,
            history_textures.as_deref(),
            history_available,
        )
        .with_material_gbuffer_valid(material_gbuffer_valid);
        for stage in EARLY_GRAPH_STAGES {
            let is_depth_prepass = *stage == RenderPassStage::DepthPrepass;
            let is_shadow = *stage == RenderPassStage::Shadow;
            if is_depth_prepass {
                push_group(&mut encoder, RENDERDOC_MARKER_PREPASS);
            }
            let overlay_renderer = if is_depth_prepass {
                Some(&mut self.overlay_renderer)
            } else {
                None
            };
            let depth_prepass_streamer = is_depth_prepass.then_some(streamer);
            let depth_prepass_mesh_pipelines = if is_depth_prepass {
                Some(&mut self.mesh_pipelines)
            } else {
                None
            };
            let stage_result = execute_graph_stage(
                pipeline,
                render_pass_executors,
                *stage,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                Some(early_post_process_stack),
                overlay_renderer,
                None,
                is_depth_prepass.then_some(&self.normal_prepass),
                None,
                None,
                None,
                depth_prepass_streamer,
                depth_prepass_mesh_pipelines,
                (is_depth_prepass || is_shadow).then_some(mesh_draw_lists),
                self.hzb_occlusion_culler.as_ref(),
                is_shadow.then_some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                is_shadow.then_some(&shadow_frame_plan),
                &mut graph_execution,
            );
            if is_depth_prepass {
                pop_group(&mut encoder);
            }
            stage_result?;
        }
        if !runtime_features.deferred_lighting_enabled {
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Lighting,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                Some(early_post_process_stack),
                None, // overlay renderer
                None, // prepared overlays
                None, // prepass
                None, // deferred resources
                None, // particle renderer
                None, // sprite renderer
                None, // resource streamer
                None, // mesh pipeline cache
                None, // mesh draw lists
                None, // HZB occlusion culler
                None, // shadow map renderer
                Some(&self.shadow_atlas_resources),
                None,
                &mut graph_execution,
            )?;
        }
        self.render_scene_passes(
            device,
            queue,
            &mut encoder,
            streamer,
            frame,
            target,
            runtime_features,
            pipeline,
            render_pass_executors,
            &mut graph_execution,
            mesh_draw_lists,
            history_textures.as_deref(),
            history_available,
        )?;
        let mut runtime_frame = frame.clone();
        if !history_available {
            runtime_frame.extract.post_process.stack = runtime_frame
                .extract
                .post_process
                .stack
                .without_history_resources();
            runtime_frame.extract.post_process.graph =
                runtime_frame.extract.post_process.stack.validated_graph();
        }
        insert_marker(&mut encoder, RENDERDOC_MARKER_POST_PROCESS);
        let post_process_stack = RenderPassPostProcessStackContext::new(
            &self.post_process,
            &*target,
            streamer,
            runtime_features,
            history_textures.as_deref(),
            history_available,
        )
        .with_material_gbuffer_valid(material_gbuffer_valid);
        execute_graph_stage(
            pipeline,
            render_pass_executors,
            RenderPassStage::PostProcess,
            device,
            queue,
            &mut encoder,
            &runtime_frame,
            &self.scene_bind_group,
            &mut self.screen_space_ui_renderer,
            Some(post_process_stack),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(streamer),
            Some(&mut self.mesh_pipelines),
            Some(mesh_draw_lists),
            self.hzb_occlusion_culler.as_ref(),
            None,
            Some(&self.shadow_atlas_resources),
            None,
            &mut graph_execution,
        )?;
        graph_execution.record_post_process_graph(&runtime_frame.extract.post_process.graph);
        let history_copy_required = history_textures_present
            && (taa_history_enabled
                || runtime_features.hybrid_global_illumination_enabled
                || runtime_features.ssao_enabled
                || screen_space_reflection_history_enabled
                || hzb_history_enabled
                || exposure_history_enabled);
        if history_copy_required {
            insert_marker(&mut encoder, RENDERDOC_MARKER_HISTORY_COPY);
        }
        let history_copy_report = self.copy_history_textures(
            &mut encoder,
            target,
            &*graph_execution.resources,
            history_textures,
            runtime_features,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
        );
        graph_execution
            .record
            .set_history_copy_report(history_copy_report);
        for stage in LATE_GRAPH_STAGES {
            let overlay_stage = matches!(*stage, RenderPassStage::Overlay | RenderPassStage::Debug);
            let overlay_renderer = if overlay_stage {
                Some(&mut self.overlay_renderer)
            } else {
                None
            };
            let prepared_overlay_buffers = overlay_stage.then_some(&prepared_overlays);
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                *stage,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                None,
                overlay_renderer,
                prepared_overlay_buffers,
                None, // prepass
                None, // deferred resources
                None, // particle renderer
                None, // sprite renderer
                None, // resource streamer
                None, // mesh pipeline cache
                None, // mesh draw lists
                None, // HZB occlusion culler
                None, // shadow map renderer
                Some(&self.shadow_atlas_resources),
                None,
                &mut graph_execution,
            )?;
        }
        drop(graph_execution);
        let hzb_occlusion_indirect_args_readbacks = encode_hzb_occlusion_indirect_args_readbacks(
            device,
            &mut encoder,
            &mesh_pass_indirect_draws,
            &graph_execution_record,
        );
        queue.submit([encoder.finish()]);
        #[cfg(test)]
        attach_scene_velocity_readback_stats(
            device,
            queue,
            &graph_resources,
            &mut graph_execution_record,
        );
        if let Some(hzb_occlusion_culler) = self.hzb_occlusion_culler.as_ref() {
            attach_hzb_occlusion_readback_stats(
                hzb_occlusion_culler,
                device,
                hzb_occlusion_indirect_args_readbacks,
                &mut graph_execution_record,
            );
        }
        graph_resources.release_transient_backings_into_pool(&mut self.transient_resource_pool);
        self.transient_resource_pool.end_frame();
        graph_execution_record.set_resource_report(
            graph_execution_record
                .resource_report()
                .with_transient_pool_report(self.transient_resource_pool.last_frame_report()),
        );

        let mut renderer_outputs = advanced_plugin_readbacks.into_outputs();
        merge_plugin_renderer_outputs(&mut renderer_outputs, graph_plugin_outputs);
        let prepared_mesh_queue_stats =
            prepared_mesh_queue_stats.with_mesh_draw_replay_stats(mesh_draw_replay_stats.stats());
        let prepared_mesh_queue_stats = prepared_mesh_queue_stats.with_gpu_scene_stats(
            self.gpu_scene.stats(),
            compiled_scene_draws.gpu_scene_upload_report(),
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
        Ok(match direct_import_report {
            Some(report) => outputs.with_output_target_graph_import_report(report),
            None => outputs,
        })
    }
}

#[cfg(test)]
fn attach_scene_velocity_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    graph_resources: &RenderGraphExecutionResources,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let resource_name = PostProcessGraphResourceNames::SCENE_VELOCITY;
    let Some(texture) = graph_resources.owned_texture(resource_name) else {
        return;
    };
    let Some(desc) = graph_resources.owned_texture_desc(resource_name) else {
        return;
    };
    if desc.format != TextureFormat::Rg16Float || desc.sample_count != 1 || desc.depth != 1 {
        return;
    }
    let size = crate::core::math::UVec2::new(desc.width, desc.height);
    if size.x == 0 || size.y == 0 {
        return;
    }
    let Ok(bytes) = read_texture_rgba(device, queue, texture, size) else {
        return;
    };
    graph_execution_record.set_scene_velocity_readback_report(
        RenderSceneVelocityReadbackReport::from_raw_rg16_float_bytes(size, &bytes),
    );
}

fn attach_hzb_occlusion_readback_stats(
    culler: &HzbOcclusionCuller,
    device: &wgpu::Device,
    indirect_args_readbacks: Vec<MeshIndirectArgsReadback>,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let Some(report) = graph_execution_record.hzb_occlusion_cull_report() else {
        return;
    };
    let mut report = if report.dispatched_phase_count == 0 {
        report
            .with_readback_stats(HzbOcclusionCullReadbackStats::default())
            .with_indirect_args_readback(HzbOcclusionIndirectArgsReadbackSummary::default())
    } else {
        if let Some(readback_stats) = culler.collect_last_readback_stats(device) {
            report.with_readback_stats(readback_stats)
        } else {
            report
        }
    };
    if report.dispatched_phase_count > 0 {
        if let Some(summary) =
            collect_hzb_occlusion_indirect_args_readback_summary(device, indirect_args_readbacks)
        {
            report = report.with_indirect_args_readback(summary);
        }
    }
    graph_execution_record.set_hzb_occlusion_cull_report(report);
}

fn encode_hzb_occlusion_indirect_args_readbacks(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    indirect_draws: &MeshPassIndirectDrawExecutions,
    graph_execution_record: &RenderGraphExecutionRecord,
) -> Vec<MeshIndirectArgsReadback> {
    let Some(report) = graph_execution_record.hzb_occlusion_cull_report() else {
        return Vec::new();
    };
    if report.dispatched_phase_count == 0 {
        return Vec::new();
    }

    indirect_draws.copy_hzb_occlusion_args_to_readbacks(
        device,
        encoder,
        "zircon-hzb-occlusion-indirect-args-readback",
    )
}

fn collect_hzb_occlusion_indirect_args_readback_summary(
    device: &wgpu::Device,
    readbacks: Vec<MeshIndirectArgsReadback>,
) -> Option<HzbOcclusionIndirectArgsReadbackSummary> {
    let mut summary = HzbOcclusionIndirectArgsReadbackSummary::default();
    for readback in readbacks {
        let snapshot = readback.collect(device)?;
        summary.add_assign(HzbOcclusionIndirectArgsReadbackSummary::new(
            snapshot.args_count(),
            snapshot.compacted_draw_count(),
            snapshot.zero_instance_arg_count(),
            snapshot.remaining_instance_count(),
        ));
    }
    Some(summary)
}

fn direct_imported_final_target(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> Option<Arc<OutputTargetTextureResource>> {
    let texture = frame.output_target().texture_handle()?;
    let prepared = streamer.output_target_texture_resource(&texture.id())?;
    let plan = frame
        .output_target()
        .graph_import_plan(Some(prepared.descriptor().format.as_str()));
    (plan.status() == ViewportTextureGraphImportStatus::ReadyForDirectImport).then_some(prepared)
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
    use super::{
        active_sprite_graph_stages, EARLY_GRAPH_STAGES, LATE_GRAPH_STAGES, SPRITE_GRAPH_STAGES,
    };
    use crate::core::framework::render::RenderPipelineHandle;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
    use crate::render_graph::{QueueLane, RenderGraphBuilder};

    #[test]
    fn compiled_scene_graph_stage_lists_cover_core2d_product_stages() {
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Opaque2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Deferred));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Lighting));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask3d));
        assert_eq!(
            LATE_GRAPH_STAGES,
            &[
                RenderPassStage::Ui,
                RenderPassStage::Overlay,
                RenderPassStage::Debug,
            ]
        );
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
            graph.add_pass_with_executor(pass_name, QueueLane::Graphics, Some(executor_id));
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
