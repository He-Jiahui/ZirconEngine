use std::borrow::Cow;

use crate::core::TaskPool;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    insert_marker, pop_group, push_group, RENDERDOC_MARKER_HISTORY_COPY,
    RENDERDOC_MARKER_POST_PROCESS, RENDERDOC_MARKER_PREPASS,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderPassExecutorRegistry, RenderPassMeshCommandLists,
    RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::overlay::PreparedOverlayBuffers;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::shadow::ShadowFramePlan;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::CompiledRenderPipeline;

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::execute_graph_stage::{execute_graph_stage, RenderGraphStageExecution};

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

pub(super) struct CompiledSceneGraphStageContext<'a, 'graph, 'mesh> {
    pub(super) device: &'a wgpu::Device,
    pub(super) queue: &'a wgpu::Queue,
    pub(super) command_encoders: &'a mut FrameCommandEncoderSet,
    pub(super) streamer: &'a ResourceStreamer,
    pub(super) frame: &'a ViewportRenderFrame,
    pub(super) target: &'a mut OffscreenTarget,
    pub(super) pipeline: &'a CompiledRenderPipeline,
    pub(super) render_pass_executors: &'a RenderPassExecutorRegistry,
    pub(super) runtime_features: SceneRuntimeFeatureFlags,
    pub(super) graph_execution: &'a mut RenderGraphStageExecution<'graph>,
    pub(super) mesh_draw_lists: RenderPassMeshCommandLists<'mesh>,
    pub(super) history_textures: Option<&'a mut SceneFrameHistoryTextures>,
    pub(super) history_available: bool,
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
            queue,
            command_encoders,
            streamer,
            frame,
            target,
            pipeline,
            render_pass_executors,
            runtime_features,
            graph_execution,
            mesh_draw_lists,
            history_textures,
            history_available,
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
        scene_clear.record_frame_clear(
            queue,
            command_encoders.serial_encoder(device),
            &target.scene_color_view,
            &target.depth_view,
            frame,
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
            let stage_result = execute_graph_stage(
                pipeline,
                render_pass_executors,
                *stage,
                device,
                queue,
                command_encoders,
                frame,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                &self.scene_bind_group,
                None,
                Some(early_post_process_stack),
                None,
                None,
                None,
                None,
                None,
                stage_streamer,
                stage_mesh_pipelines,
                Some(&mut self.ibl_bake_pipeline_cache),
                uses_mesh_pipeline_context.then_some(mesh_draw_lists),
                self.hzb_occlusion_culler.as_ref(),
                is_shadow.then_some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                is_shadow.then_some(shadow_frame_plan),
                parallel_recording,
                graph_execution,
            );
            if is_depth_prepass {
                pop_group(command_encoders.serial_encoder(device));
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
                command_encoders,
                frame,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                &self.scene_bind_group,
                None,
                Some(early_post_process_stack),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(&mut self.ibl_bake_pipeline_cache),
                None,
                None,
                None,
                Some(&self.shadow_atlas_resources),
                None,
                parallel_recording,
                graph_execution,
            )?;
        }
        self.render_scene_passes(
            device,
            queue,
            command_encoders,
            streamer,
            frame,
            target,
            runtime_features,
            pipeline,
            render_pass_executors,
            graph_execution,
            mesh_draw_lists,
            history_textures.as_deref(),
            history_available,
            parallel_recording,
        )?;
        let runtime_frame = if history_available {
            Cow::Borrowed(frame)
        } else {
            let mut historyless_frame = frame.clone();
            let historyless_stack = historyless_frame
                .extract
                .post_process
                .stack
                .without_history_resources();
            let historyless_graph = historyless_stack.validated_graph();
            let extract = historyless_frame.extract_mut();
            extract.post_process.stack = historyless_stack;
            extract.post_process.graph = historyless_graph;
            Cow::Owned(historyless_frame)
        };
        insert_marker(
            command_encoders.serial_encoder(device),
            RENDERDOC_MARKER_POST_PROCESS,
        );
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
            command_encoders,
            runtime_frame.as_ref(),
            &self.scene_bind_group_layout,
            self.scene_color_format,
            self.depth_format,
            &self.scene_bind_group,
            None,
            Some(post_process_stack),
            None,
            None,
            None,
            None,
            None,
            Some(streamer),
            Some(&mut self.mesh_pipelines),
            Some(&mut self.ibl_bake_pipeline_cache),
            Some(mesh_draw_lists),
            self.hzb_occlusion_culler.as_ref(),
            None,
            Some(&self.shadow_atlas_resources),
            None,
            parallel_recording,
            graph_execution,
        )?;
        graph_execution.record_post_process_graph(&runtime_frame.extract.post_process.graph);
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
        let history_copy_report = self.copy_history_textures(
            command_encoders.serial_encoder(device),
            target,
            runtime_frame.render_region(),
            &*graph_execution.resources,
            history_textures,
            runtime_features,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
            volumetric_history_enabled,
        );
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
                device,
                queue,
                command_encoders,
                frame,
                &self.scene_bind_group_layout,
                self.final_color_format,
                self.depth_format,
                &self.scene_bind_group,
                screen_space_ui_renderer,
                None,
                overlay_renderer,
                prepared_overlay_buffers,
                None,
                None,
                None,
                Some(streamer),
                None,
                Some(&mut self.ibl_bake_pipeline_cache),
                None,
                None,
                None,
                Some(&self.shadow_atlas_resources),
                None,
                parallel_recording,
                graph_execution,
            )?;
        }
        Ok(())
    }
}

fn active_late_graph_stages(
    pipeline: &CompiledRenderPipeline,
) -> impl Iterator<Item = RenderPassStage> + '_ {
    pipeline
        .stages
        .iter()
        .copied()
        .filter(|stage| LATE_GRAPH_STAGES.contains(stage))
}

#[cfg(test)]
mod tests {
    use super::{active_late_graph_stages, EARLY_GRAPH_STAGES, LATE_GRAPH_STAGES};
    use crate::core::framework::render::RenderPipelineHandle;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::CompiledRenderPipeline;
    use crate::render_graph::RenderGraphBuilder;

    #[test]
    fn compiled_scene_graph_stage_lists_keep_early_and_late_boundaries() {
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Opaque2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Deferred));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Lighting));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask3d));
        assert!(LATE_GRAPH_STAGES.contains(&RenderPassStage::Ui));
        assert!(LATE_GRAPH_STAGES.contains(&RenderPassStage::Overlay));
        assert!(LATE_GRAPH_STAGES.contains(&RenderPassStage::Debug));
    }

    #[test]
    fn compiled_scene_stage_execution_borrows_stable_frame_and_late_stage_iterator() {
        let source = include_str!("execute_compiled_scene_graph_stages.rs");
        let borrowed_frame = ["Cow::Borrowed(", "frame)"].concat();
        let iterator_return = ["-> impl ", "Iterator<Item = RenderPassStage>"].concat();

        assert!(source.contains(&borrowed_frame));
        assert!(source.contains(&iterator_return));
    }

    #[test]
    fn active_late_graph_stages_follow_compiled_pipeline_order() {
        let default_3d = compiled_pipeline_with_stages(vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
            RenderPassStage::Ui,
        ]);
        assert_eq!(
            active_late_graph_stages(&default_3d).collect::<Vec<_>>(),
            vec![
                RenderPassStage::Overlay,
                RenderPassStage::Debug,
                RenderPassStage::Ui
            ]
        );

        let core2d = compiled_pipeline_with_stages(vec![
            RenderPassStage::Opaque2d,
            RenderPassStage::PostProcess,
            RenderPassStage::Ui,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
        ]);
        assert_eq!(
            active_late_graph_stages(&core2d).collect::<Vec<_>>(),
            vec![
                RenderPassStage::Ui,
                RenderPassStage::Overlay,
                RenderPassStage::Debug
            ]
        );
    }

    fn compiled_pipeline_with_stages(stages: Vec<RenderPassStage>) -> CompiledRenderPipeline {
        CompiledRenderPipeline::from_parts(crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: RenderPipelineHandle::new(100),
            name: "stage-order-test".to_string(),
            renderer_name: "stage-order-test".to_string(),
            stages,
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            half_resolution_transparency_depth_sigma:
                crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph: RenderGraphBuilder::new("stage-order-test")
                .compile()
                .expect("stage order test graph"),
        })
    }
}
