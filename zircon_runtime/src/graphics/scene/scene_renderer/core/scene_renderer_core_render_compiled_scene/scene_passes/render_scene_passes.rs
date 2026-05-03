use crate::graphics::backend::OffscreenTarget;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistry;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::CompiledRenderPipeline;

use super::super::super::super::mesh::MeshDraw;
use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::render::execute_graph_stage::{execute_graph_stage, RenderGraphStageExecution};

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn render_scene_passes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        target: &mut OffscreenTarget,
        runtime_features: SceneRuntimeFeatureFlags,
        pipeline: &CompiledRenderPipeline,
        render_pass_executors: &RenderPassExecutorRegistry,
        graph_execution: &mut RenderGraphStageExecution<'_>,
        mesh_draws: &[MeshDraw],
        opaque_mesh_draws: &[&MeshDraw],
        transparent_mesh_draws: &[&MeshDraw],
    ) -> Result<(), GraphicsError> {
        if runtime_features.deferred_lighting_enabled {
            self.overlay_renderer.record_preview_sky(
                encoder,
                &target.final_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                frame,
            );
            self.normal_prepass.record(
                encoder,
                &target.normal_view,
                &target.depth_view,
                &self.scene_bind_group,
                opaque_mesh_draws.iter().copied(),
            );
            self.deferred.record_gbuffer_geometry(
                encoder,
                &target.gbuffer_albedo_view,
                &target.depth_view,
                &self.scene_bind_group,
                opaque_mesh_draws.iter().copied(),
            );
        } else {
            self.normal_prepass.record(
                encoder,
                &target.normal_view,
                &target.depth_view,
                &self.scene_bind_group,
                mesh_draws.iter(),
            );
            self.overlay_renderer.record_scene_content(
                encoder,
                device,
                &target.scene_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                mesh_draws,
                &mut self.mesh_pipelines,
                streamer,
                frame,
            );
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Transparent,
                device,
                queue,
                encoder,
                frame,
                &self.scene_bind_group,
                graph_execution,
            )?;
            if runtime_features.particle_rendering_enabled {
                self.particle_renderer.record(
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    frame,
                );
            }
        }

        if runtime_features.deferred_lighting_enabled {
            self.deferred.execute_lighting(
                device,
                encoder,
                &self.scene_bind_group,
                &target.gbuffer_albedo_view,
                &target.normal_view,
                &target.final_color_view,
                &target.scene_color_view,
            );
            self.overlay_renderer.record_meshes(
                encoder,
                device,
                &target.scene_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                transparent_mesh_draws.iter().copied(),
                &mut self.mesh_pipelines,
                streamer,
                frame,
            );
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Transparent,
                device,
                queue,
                encoder,
                frame,
                &self.scene_bind_group,
                graph_execution,
            )?;
            if runtime_features.particle_rendering_enabled {
                self.particle_renderer.record(
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    frame,
                );
            }
        }

        Ok(())
    }
}
