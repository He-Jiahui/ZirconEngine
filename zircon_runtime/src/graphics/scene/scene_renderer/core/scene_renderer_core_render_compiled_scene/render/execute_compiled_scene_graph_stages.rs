use crate::core::TaskPool;
use crate::core::framework::render::RenderPipelinePhase;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::{OffscreenTarget, ViewportSurface};
use crate::graphics::debug_markers::{
    RENDERDOC_MARKER_HISTORY_COPY, RENDERDOC_MARKER_POST_PROCESS, RENDERDOC_MARKER_PREPASS,
    insert_marker, pop_group, push_group,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderPassExecutorRegistry, RenderPassMeshCommandLists,
    RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryAvailability, SceneHistoryFrameTransaction,
};
use crate::graphics::scene::scene_renderer::overlay::PreparedOverlayBuffers;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::shadow::ShadowFramePlan;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::RenderGraphPassFrameServices;
use super::execute_graph_stage::{RenderGraphStageExecution, execute_graph_stage};

pub(super) const EARLY_GRAPH_STAGES: &[RenderPassStage] =
    &[RenderPassStage::DepthPrepass, RenderPassStage::Shadow];

pub(super) const FORWARD_PRE_SCENE_GRAPH_STAGES: &[RenderPassStage] =
    &[RenderPassStage::AmbientOcclusion, RenderPassStage::Lighting];

pub(super) const LATE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Ui,
    RenderPassStage::Overlay,
    RenderPassStage::Debug,
];

pub(super) struct CompiledSceneGraphStageContext<'a, 'graph, 'mesh> {
    pub(super) device: &'a wgpu::Device,
    pub(super) command_encoders: &'a mut FrameCommandEncoderSet,
    pub(super) streamer: &'a ResourceStreamer,
    pub(super) frame: &'a ViewportRenderFrame,
    pub(super) surface_frame: Option<(
        &'a ViewportSurface,
        &'a zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget,
    )>,
    pub(super) target: &'a mut OffscreenTarget,
    pub(super) pipeline: &'a CompiledRenderPipeline,
    pub(super) render_pass_executors: &'a RenderPassExecutorRegistry,
    pub(super) runtime_features: SceneRuntimeFeatureFlags,
    pub(super) graph_execution: &'a mut RenderGraphStageExecution<'graph>,
    pub(super) mesh_draw_lists: RenderPassMeshCommandLists<'mesh>,
    pub(super) history_textures: Option<&'a SceneFrameHistoryTextures>,
    pub(super) history_frame_transaction: &'a mut SceneHistoryFrameTransaction,
    pub(super) history_availability: SceneHistoryAvailability,
    pub(super) material_gbuffer_valid: bool,
    pub(super) taa_history_enabled: bool,
    pub(super) screen_space_reflection_history_enabled: bool,
    pub(super) hzb_history_enabled: bool,
    pub(super) exposure_history_enabled: bool,
    pub(super) volumetric_history_enabled: bool,
    pub(super) shadow_frame_plan: &'a ShadowFramePlan,
    pub(super) prepared_overlays: &'a PreparedOverlayBuffers,
    pub(super) parallel_recording: Option<(&'a TaskPool, usize)>,
}

impl SceneRendererCore {
    pub(super) fn execute_compiled_scene_graph_stages(
        &mut self,
        ctx: CompiledSceneGraphStageContext<'_, '_, '_>,
    ) -> Result<(), GraphicsError> {
        let CompiledSceneGraphStageContext {
            device,
            command_encoders,
            streamer,
            frame,
            surface_frame,
            target,
            pipeline,
            render_pass_executors,
            runtime_features,
            graph_execution,
            mesh_draw_lists,
            history_textures,
            history_frame_transaction,
            history_availability,
            material_gbuffer_valid,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
            volumetric_history_enabled,
            shadow_frame_plan,
            prepared_overlays,
            parallel_recording,
        } = ctx;

        let scene_clear = self.scene_clear.as_ref().ok_or_else(|| {
            GraphicsError::Asset("compiled scene graph requires scene-clear resources".to_owned())
        })?;
        let mut scene_clear_uploads = scene_clear.record_frame_clear(
            command_encoders.serial_encoder(device),
            &target.scene_color_view,
            &target.depth_view,
            frame,
        );
        graph_execution.append_buffer_uploads(&mut scene_clear_uploads);
        for stage in EARLY_GRAPH_STAGES {
            let is_depth_prepass = *stage == RenderPassStage::DepthPrepass;
            let is_shadow = *stage == RenderPassStage::Shadow;
            let uses_mesh_pipeline_context = is_depth_prepass || is_shadow;
            if is_depth_prepass {
                push_group(
                    command_encoders.serial_encoder(device),
                    RENDERDOC_MARKER_PREPASS,
                );
            }
            let stage_streamer = uses_mesh_pipeline_context.then_some(streamer);
            let stage_mesh_pipelines = if uses_mesh_pipeline_context {
                Some(&mut self.mesh_pipelines)
            } else {
                None
            };
            let early_post_process_stack = RenderPassPostProcessStackContext::new(
                &self.post_process,
                streamer,
                runtime_features,
                history_textures,
                history_availability,
            )
            .with_material_gbuffer_valid(material_gbuffer_valid);
            let stage_result = execute_graph_stage(
                pipeline,
                render_pass_executors,
                *stage,
                RenderGraphPassFrameServices {
                    device,
                    command_encoders,
                    frame,
                    scene_bind_group_layout: &self.scene_bind_group_layout,
                    target_format: self.scene_color_format,
                    depth_format: self.depth_format,
                    scene_bind_group: &self.scene_bind_group,
                    surface_frame: None,
                    screen_space_ui_renderer: None,
                    post_process_stack: Some(early_post_process_stack),
                    overlay_renderer: None,
                    prepared_overlays: None,
                    deferred: None,
                    particle_renderer: None,
                    sprite_renderer: None,
                    streamer: stage_streamer,
                    ibl_bake_pipeline_cache: Some(&mut self.ibl_bake_pipeline_cache),
                    mesh_pipelines: stage_mesh_pipelines,
                    mesh_draw_lists: uses_mesh_pipeline_context.then_some(mesh_draw_lists),
                    hzb_occlusion_culler: self.hzb_occlusion_culler.as_ref(),
                    shadow_map_renderer: if is_shadow {
                        self.shadow_map_renderer.as_ref()
                    } else {
                        None
                    },
                    shadow_atlas_resources: Some(&self.shadow_atlas_resources),
                    shadow_frame_plan: is_shadow.then_some(shadow_frame_plan),
                    parallel_recording,
                },
                graph_execution,
            );
            if is_depth_prepass {
                pop_group(command_encoders.serial_encoder(device));
            }
            stage_result?;
        }
        if !runtime_features.deferred_lighting_enabled {
            for stage in FORWARD_PRE_SCENE_GRAPH_STAGES {
                let early_post_process_stack = RenderPassPostProcessStackContext::new(
                    &self.post_process,
                    streamer,
                    runtime_features,
                    history_textures,
                    history_availability,
                )
                .with_material_gbuffer_valid(material_gbuffer_valid);
                execute_graph_stage(
                    pipeline,
                    render_pass_executors,
                    *stage,
                    RenderGraphPassFrameServices {
                        device,
                        command_encoders,
                        frame,
                        scene_bind_group_layout: &self.scene_bind_group_layout,
                        target_format: self.scene_color_format,
                        depth_format: self.depth_format,
                        scene_bind_group: &self.scene_bind_group,
                        surface_frame: None,
                        screen_space_ui_renderer: None,
                        post_process_stack: Some(early_post_process_stack),
                        overlay_renderer: None,
                        prepared_overlays: None,
                        deferred: None,
                        particle_renderer: None,
                        sprite_renderer: None,
                        streamer: None,
                        ibl_bake_pipeline_cache: Some(&mut self.ibl_bake_pipeline_cache),
                        mesh_pipelines: None,
                        mesh_draw_lists: None,
                        hzb_occlusion_culler: None,
                        shadow_map_renderer: None,
                        shadow_atlas_resources: Some(&self.shadow_atlas_resources),
                        shadow_frame_plan: None,
                        parallel_recording,
                    },
                    graph_execution,
                )?;
            }
        }
        self.render_scene_passes(
            device,
            command_encoders,
            streamer,
            frame,
            runtime_features,
            pipeline,
            render_pass_executors,
            graph_execution,
            mesh_draw_lists,
            history_textures,
            history_availability,
            parallel_recording,
        )?;
        insert_marker(
            command_encoders.serial_encoder(device),
            RENDERDOC_MARKER_POST_PROCESS,
        );
        let post_process_stack = RenderPassPostProcessStackContext::new(
            &self.post_process,
            streamer,
            runtime_features,
            history_textures,
            history_availability,
        )
        .with_material_gbuffer_valid(material_gbuffer_valid);
        execute_graph_stage(
            pipeline,
            render_pass_executors,
            RenderPassStage::PostProcess,
            RenderGraphPassFrameServices {
                device,
                command_encoders,
                frame,
                scene_bind_group_layout: &self.scene_bind_group_layout,
                target_format: self.scene_color_format,
                depth_format: self.depth_format,
                scene_bind_group: &self.scene_bind_group,
                surface_frame: None,
                screen_space_ui_renderer: None,
                post_process_stack: Some(post_process_stack),
                overlay_renderer: None,
                prepared_overlays: None,
                deferred: None,
                particle_renderer: None,
                sprite_renderer: None,
                streamer: Some(streamer),
                ibl_bake_pipeline_cache: Some(&mut self.ibl_bake_pipeline_cache),
                mesh_pipelines: Some(&mut self.mesh_pipelines),
                mesh_draw_lists: Some(mesh_draw_lists),
                hzb_occlusion_culler: self.hzb_occlusion_culler.as_ref(),
                shadow_map_renderer: None,
                shadow_atlas_resources: Some(&self.shadow_atlas_resources),
                shadow_frame_plan: None,
                parallel_recording,
            },
            graph_execution,
        )?;
        graph_execution.record_post_process_graph(&frame.post_process().graph);
        let history_copy_required = history_textures.is_some()
            && (taa_history_enabled
                || runtime_features.hybrid_global_illumination_enabled
                || runtime_features.ssao_enabled
                || screen_space_reflection_history_enabled
                || hzb_history_enabled
                || exposure_history_enabled
                || volumetric_history_enabled);
        if history_copy_required {
            insert_marker(
                command_encoders.serial_encoder(device),
                RENDERDOC_MARKER_HISTORY_COPY,
            );
        }
        let scene_linear_region = frame
            .render_region_for_phase(RenderPipelinePhase::SceneLinear)
            .unwrap_or_else(|| frame.render_region());
        let (history_copy_report, history_write_intent) = self
            .copy_history_textures(
                command_encoders.serial_encoder(device),
                target,
                scene_linear_region,
                &*graph_execution.resources,
                pipeline.history_epilogue_plan(),
                graph_execution.history_writes(),
                history_textures,
                runtime_features,
                taa_history_enabled,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                volumetric_history_enabled,
            )
            .map_err(GraphicsError::Asset)?;
        history_frame_transaction.absorb_writes(history_write_intent);
        graph_execution
            .record
            .set_history_copy_report(history_copy_report);
        for stage in active_late_graph_stages(pipeline) {
            let overlay_stage = matches!(stage, RenderPassStage::Overlay | RenderPassStage::Debug);
            let overlay_renderer = if overlay_stage {
                Some(&mut self.overlay_renderer)
            } else {
                None
            };
            let prepared_overlay_buffers = overlay_stage.then_some(prepared_overlays);
            let screen_space_ui_renderer = if stage == RenderPassStage::Ui {
                self.screen_space_ui_renderer.as_mut()
            } else {
                None
            };
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                stage,
                RenderGraphPassFrameServices {
                    device,
                    command_encoders,
                    frame,
                    scene_bind_group_layout: &self.scene_bind_group_layout,
                    target_format: self.final_color_format,
                    depth_format: self.depth_format,
                    scene_bind_group: &self.scene_bind_group,
                    surface_frame: None,
                    screen_space_ui_renderer,
                    post_process_stack: None,
                    overlay_renderer,
                    prepared_overlays: prepared_overlay_buffers,
                    deferred: None,
                    particle_renderer: None,
                    sprite_renderer: None,
                    streamer: Some(streamer),
                    ibl_bake_pipeline_cache: Some(&mut self.ibl_bake_pipeline_cache),
                    mesh_pipelines: None,
                    mesh_draw_lists: None,
                    hzb_occlusion_culler: None,
                    shadow_map_renderer: None,
                    shadow_atlas_resources: Some(&self.shadow_atlas_resources),
                    shadow_frame_plan: None,
                    parallel_recording,
                },
                graph_execution,
            )?;
        }
        if surface_frame.is_some() || frame.output_target().texture_handle().is_some() {
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Present,
                RenderGraphPassFrameServices {
                    device,
                    command_encoders,
                    frame,
                    scene_bind_group_layout: &self.scene_bind_group_layout,
                    target_format: self.final_color_format,
                    depth_format: self.depth_format,
                    scene_bind_group: &self.scene_bind_group,
                    surface_frame,
                    screen_space_ui_renderer: None,
                    post_process_stack: None,
                    overlay_renderer: None,
                    prepared_overlays: None,
                    deferred: None,
                    particle_renderer: None,
                    sprite_renderer: None,
                    streamer: Some(streamer),
                    ibl_bake_pipeline_cache: None,
                    mesh_pipelines: None,
                    mesh_draw_lists: None,
                    hzb_occlusion_culler: None,
                    shadow_map_renderer: None,
                    shadow_atlas_resources: None,
                    shadow_frame_plan: None,
                    parallel_recording: None,
                },
                graph_execution,
            )?;
        }
        graph_execution.validate_graph_execution(pipeline)?;
        Ok(())
    }
}

pub(super) fn active_late_graph_stages(
    pipeline: &CompiledRenderPipeline,
) -> impl Iterator<Item = RenderPassStage> + '_ {
    pipeline
        .execution_stages_in_graph_order()
        .filter(|stage| LATE_GRAPH_STAGES.contains(stage))
}
