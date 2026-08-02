use crate::core::framework::render::{
    select_irradiance_volume_for_view, AntiAliasMode, PostProcessGraphResourceNames,
    RenderCapabilitySummary, RenderPluginRendererOutputs, SkyboxMode,
};
use crate::core::TaskPool;
use crate::graphics::backend::{GpuPassTimer, OffscreenTarget};
use crate::graphics::debug_markers::{insert_marker, RENDERDOC_MARKER_FRAME_EXTRACT};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_render_with_pipeline::AsyncViewportCaptureRequest;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderGraphExecutionRecord, RenderGraphExecutionResources,
    RenderPassExecutorRegistry,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::{
    build_mesh_pass_command_buffers_cached, build_mesh_pass_command_buffers_cached_parallel,
    MeshDrawReplayStatsAccumulator, MeshPassIndirectDrawExecutions,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::sprite::prepare_sprite_queue_stats;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::CompiledRenderPipeline;

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
use super::pipeline_resource_usage::pipeline_writes_resource;
use super::prepare_overlay_buffers::prepare_overlay_buffers;
use super::sprite_stage_selection::active_sprite_graph_stages;
use super::submit_compiled_scene_frame::CompiledSceneFrameSubmissionContext;

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
        frame_generation: u64,
        mut gpu_pass_timer: Option<&mut GpuPassTimer>,
        compute_task_pool: Option<&TaskPool>,
        parallel_record_min_passes_per_bucket: Option<usize>,
        hzb_indirect_args_readback_enabled: bool,
        viewport_capture: Option<AsyncViewportCaptureRequest>,
    ) -> Result<SceneRendererCompiledSceneOutputs, GraphicsError> {
        ensure_compiled_scene_graph_resources(
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
        self.write_scene_uniform(
            device,
            queue,
            streamer,
            frame,
            realtime_ibl_prepared.as_ref(),
            runtime_features.reflection_probes_enabled,
        )?;
        let camera_layers = frame.extract.view.selected_camera_layers();
        let irradiance_volumes = &frame.extract.lighting.advanced_lighting.irradiance_volumes;
        let irradiance_sample_positions = collect_irradiance_sample_positions(
            !irradiance_volumes.is_empty(),
            frame
                .extract
                .geometry
                .meshes
                .iter()
                .filter(|mesh| mesh.common.layer_mask.intersects(camera_layers))
                .map(|mesh| mesh.transform.translation),
        );
        let selected_irradiance_volume = irradiance_sample_positions
            .as_deref()
            .and_then(|positions| {
                select_irradiance_volume_for_view(irradiance_volumes, camera_layers, positions)
            })
            .cloned()
            .and_then(|volume| {
                streamer
                    .irradiance_volume_texture(volume.voxels)
                    .map(|texture| (volume, texture))
            });
        self.mesh_pipelines
            .irradiance_volume
            .prepare(queue, selected_irradiance_volume);
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
        let gpu_timing_enabled = gpu_pass_timer.is_some();
        let mut realtime_ibl_submission = realtime_ibl_prepared
            .as_ref()
            .map(|prepared| {
                self.realtime_ibl.record_prepared_frame(
                    device,
                    &mut encoder,
                    gpu_timing_enabled,
                    prepared,
                    &mut self.ibl_bake_pipeline_cache,
                )
            })
            .transpose()
            .map_err(GraphicsError::Asset)?
            .flatten();
        let generation_ids =
            RenderGenerationIds::new(frame_generation, self.mesh_command_generation);
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
            generation_ids.mesh_commands,
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
                advanced_pbr_opaque_commands: mesh_pass_command_buffers
                    .advanced_pbr_opaque()
                    .commands(),
                transmission_commands: mesh_pass_command_buffers.transmission().commands(),
                transmission_step_count: frame
                    .extract
                    .lighting
                    .advanced_lighting
                    .transmission_draw_step_count(),
                transparent_commands: mesh_pass_command_buffers.transparent().commands(),
                velocity_commands: mesh_pass_command_buffers.velocity().commands(),
                taa_reactive_mask_commands: mesh_pass_command_buffers
                    .taa_reactive_mask()
                    .commands(),
                depth_prepass_indirect: mesh_pass_indirect_draws.depth_prepass(),
                shadow_indirect: mesh_pass_indirect_draws.shadow(),
                opaque_indirect: mesh_pass_indirect_draws.opaque(),
                alpha_mask_indirect: mesh_pass_indirect_draws.alpha_mask(),
                advanced_pbr_opaque_indirect: mesh_pass_indirect_draws.advanced_pbr_opaque(),
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
        let volumetric_history_enabled = pipeline_writes_resource(
            pipeline,
            PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        ) && crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_history_quality(
            &frame.extract,
            frame.shader_quality(),
        )
        .map_err(GraphicsError::Asset)?
        .is_some();

        let mut advanced_plugin_readbacks =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, streamer, frame)?;
        let mut graph_resources = RenderGraphExecutionResources::new();
        self.transient_resource_pool.begin_frame();
        let environment_source_cubemap_view = frame
            .environment()
            .skybox
            .source_cubemap_environment()
            .map(|_| self.scene_environment_cubemap.source_view());
        let final_target_output = bind_compiled_scene_graph_resources(
            device,
            pipeline,
            streamer,
            frame,
            target,
            self.gpu_scene.light_buffer(),
            history_textures.as_deref(),
            CompiledSceneGraphResourceBindingFlags {
                taa_history_enabled,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                volumetric_history_enabled,
                history_available,
                runtime_features,
            },
            &mut graph_resources,
            &mut self.transient_resource_pool,
            mesh_draw_lists,
            self.hzb_occlusion_culler.as_ref(),
            &self.shadow_atlas_resources,
            advanced_plugin_readbacks.external_buffer_bindings(),
            environment_source_cubemap_view,
        )?;
        let materialization_report = graph_resources
            .validate_materialized_graph_resources(pipeline.graph())
            .map_err(GraphicsError::Asset)?;
        let mut graph_execution_record = RenderGraphExecutionRecord::default();
        graph_execution_record.set_materialization_report(materialization_report);
        graph_execution_record.set_resource_report(graph_resources.resource_report());
        graph_execution_record.set_resource_alias_report(graph_resources.resource_alias_report());
        let mut graph_plugin_outputs = RenderPluginRendererOutputs::default();
        self.readback_frame_index = self.readback_frame_index.wrapping_add(1);
        let readback_frame_index = self.readback_frame_index;
        let readback_ready = self
            .readback_queue
            .prepare_frame(device, readback_frame_index)
            .is_ok();
        if readback_ready {
            if let Err(error) =
                advanced_plugin_readbacks.register_gpu_readbacks(&mut self.readback_queue)
            {
                self.readback_queue.abort_frame(readback_frame_index);
                return Err(GraphicsError::BufferMap(error));
            }
        } else {
            advanced_plugin_readbacks
                .fail_gpu_readbacks("shared GPU readback queue was unavailable for this frame");
        }
        if readback_ready {
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.begin_frame(generation_ids.timer_frame());
            }
        }
        let mut graph_execution = RenderGraphStageExecution::new(
            &mut graph_resources,
            &mut graph_execution_record,
            &mut graph_plugin_outputs,
            gpu_pass_timer.as_deref_mut(),
        );
        let mut command_encoders = FrameCommandEncoderSet::from_serial_encoder(encoder);
        let parallel_recording = compute_task_pool.zip(parallel_record_min_passes_per_bucket);
        let graph_execution_result =
            self.execute_compiled_scene_graph_stages(CompiledSceneGraphStageContext {
                device,
                queue,
                command_encoders: &mut command_encoders,
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
                volumetric_history_enabled,
                shadow_frame_plan: &shadow_frame_plan,
                prepared_overlays: &prepared_overlays,
                parallel_recording,
            });
        drop(graph_execution);
        if let Err(error) = graph_execution_result {
            if readback_ready {
                self.readback_queue.abort_frame(readback_frame_index);
            }
            if let Some(submission) = realtime_ibl_submission.take() {
                self.realtime_ibl.complete_submission(submission, false);
            }
            return Err(error);
        }

        let hzb_readback_requested = graph_execution_record
            .hzb_occlusion_cull_report()
            .is_some_and(|report| report.dispatched_phase_count > 0);
        if readback_ready {
            if let Some(submission) = realtime_ibl_submission.as_ref() {
                self.realtime_ibl.request_gpu_timestamp_readback(
                    submission,
                    queue.get_timestamp_period(),
                    &mut self.readback_queue,
                );
            }
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.resolve_and_request(
                    command_encoders.serial_encoder(device),
                    &mut self.readback_queue,
                );
            }
            if hzb_readback_requested {
                if let Some(culler) = self.hzb_occlusion_culler.as_ref() {
                    if let Err(error) = culler.request_frame_readbacks(
                        &mut self.readback_queue,
                        &mesh_pass_indirect_draws,
                        readback_frame_index,
                        hzb_indirect_args_readback_enabled,
                    ) {
                        self.readback_queue.abort_frame(readback_frame_index);
                        if let Some(submission) = realtime_ibl_submission.take() {
                            self.realtime_ibl.complete_submission(submission, false);
                        }
                        return Err(GraphicsError::BufferMap(error.to_string()));
                    }
                }
            }
        } else if hzb_readback_requested {
            if let Some(culler) = self.hzb_occlusion_culler.as_ref() {
                culler.record_skipped_readback();
            }
        }
        if readback_ready {
            if let Some(viewport_capture) = viewport_capture {
                // The final offscreen color is available before the queue submission and shares
                // the renderer-owned bounded readback ring with all other GPU consumers.
                let (callback, admission) = viewport_capture.into_parts();
                if self
                    .readback_queue
                    .request_texture_rgba(
                        "viewport-final-color",
                        &target.final_color,
                        target.size.x,
                        target.size.y,
                        callback,
                    )
                    .is_ok()
                {
                    admission.store(std::sync::atomic::Ordering::Release);
                }
            }
            if let Err(error) = self.readback_queue.encode_copies(
                command_encoders.serial_encoder(device),
                readback_frame_index,
            ) {
                self.readback_queue.abort_frame(readback_frame_index);
                if let Some(submission) = realtime_ibl_submission.take() {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(GraphicsError::BufferMap(error.to_string()));
            }
        }

        self.submit_compiled_scene_frame(CompiledSceneFrameSubmissionContext {
            device,
            queue,
            command_buffers: command_encoders.finish(),
            streamer,
            frame,
            graph_resources: &mut graph_resources,
            graph_execution_record: &mut graph_execution_record,
            environment_ibl_bake_request: pipeline.environment_ibl_bake_request,
            realtime_ibl_submission,
            readback_frame_index: readback_ready.then_some(readback_frame_index),
        })?;

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
        let _prev_morph_weights_roll_report =
            self.gpu_scene.roll_prev_morph_weights_after_success();
        Ok(match final_target_output.graph_import_report {
            Some(report) => outputs.with_output_target_graph_import_report(report),
            None => outputs,
        })
    }
}

fn ensure_compiled_scene_graph_resources(
    has_full_post_process_resources: bool,
    has_scene_clear_resources: bool,
) -> Result<(), GraphicsError> {
    if !has_full_post_process_resources {
        return Err(GraphicsError::Asset(
            "environment-only PBR startup profile cannot execute a compiled scene graph".to_owned(),
        ));
    }
    if !has_scene_clear_resources {
        return Err(GraphicsError::Asset(
            "compiled scene graph requires scene-clear resources".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod resource_mode_tests {
    use super::ensure_compiled_scene_graph_resources;
    use crate::graphics::types::GraphicsError;

    #[test]
    fn output_transfer_only_resources_are_rejected_before_compiled_graph_execution() {
        assert!(ensure_compiled_scene_graph_resources(true, true).is_ok());
        let error = ensure_compiled_scene_graph_resources(false, true)
            .expect_err("expected output-transfer-only rejection");
        assert!(matches!(error, GraphicsError::Asset(_)));
        assert!(error
            .to_string()
            .contains("cannot execute a compiled scene graph"));
        let error = ensure_compiled_scene_graph_resources(true, false)
            .expect_err("expected missing scene-clear rejection");
        assert!(matches!(error, GraphicsError::Asset(_)));
        assert!(error.to_string().contains("scene-clear resources"));
    }
}

fn collect_irradiance_sample_positions<T>(
    has_irradiance_volumes: bool,
    positions: impl Iterator<Item = T>,
) -> Option<Vec<T>> {
    has_irradiance_volumes.then(|| positions.collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderGenerationIds {
    frame: u64,
    mesh_commands: u64,
}

impl RenderGenerationIds {
    fn new(frame: u64, mesh_commands: u64) -> Self {
        Self {
            frame,
            mesh_commands,
        }
    }

    fn timer_frame(self) -> u64 {
        self.frame
    }
}

#[cfg(test)]
mod performance_tests {
    use std::cell::Cell;

    use super::{collect_irradiance_sample_positions, RenderGenerationIds};

    #[test]
    fn gpu_timer_frame_generation_stays_independent_from_mesh_command_cache_generation() {
        let generation_ids = RenderGenerationIds::new(41, 7);

        assert_eq!(generation_ids.timer_frame(), 41);
        assert_eq!(generation_ids.mesh_commands, 7);
    }

    #[test]
    fn frame_without_irradiance_volumes_does_not_scan_mesh_positions() {
        let visited = Cell::new(0);
        let positions = (0..4).inspect(|_| visited.set(visited.get() + 1));

        let collected = collect_irradiance_sample_positions(false, positions);

        assert!(collected.is_none());
        assert_eq!(visited.get(), 0);
    }

    #[test]
    fn frame_with_irradiance_volumes_collects_mesh_positions_once() {
        let visited = Cell::new(0);
        let positions = (0..4).inspect(|_| visited.set(visited.get() + 1));

        let collected = collect_irradiance_sample_positions(true, positions);

        assert_eq!(collected, Some(vec![0, 1, 2, 3]));
        assert_eq!(visited.get(), 4);
    }
}
