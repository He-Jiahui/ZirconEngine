use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    insert_marker, pop_group, push_group, RENDERDOC_MARKER_CLEAR,
    RENDERDOC_MARKER_DEFERRED_LIGHTING, RENDERDOC_MARKER_MAIN_SCENE, RENDERDOC_MARKER_PREPASS,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistry;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::CompiledRenderPipeline;

use super::super::super::super::mesh::MeshDraw;
use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::super::sprite::SpriteRenderer;
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
            insert_marker(encoder, RENDERDOC_MARKER_CLEAR);
            self.overlay_renderer.record_preview_sky(
                encoder,
                &target.final_color_view,
                &target.depth_view,
                &self.scene_bind_group,
                frame,
            );
            push_group(encoder, RENDERDOC_MARKER_PREPASS);
            self.normal_prepass.record(
                encoder,
                &target.normal_view,
                &target.depth_view,
                &self.scene_bind_group,
                opaque_mesh_draws.iter().copied(),
            );
            pop_group(encoder);
            push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
            self.deferred.record_gbuffer_geometry(
                encoder,
                &target.gbuffer_albedo_view,
                &target.depth_view,
                &self.scene_bind_group,
                opaque_mesh_draws.iter().copied(),
            );
            pop_group(encoder);
            if runtime_features.sprite_rendering_enabled {
                record_sprite_stage(
                    &self.sprite_renderer,
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    RenderPassStage::Opaque2d,
                );
            }
        } else {
            insert_marker(encoder, RENDERDOC_MARKER_CLEAR);
            push_group(encoder, RENDERDOC_MARKER_PREPASS);
            self.normal_prepass.record(
                encoder,
                &target.normal_view,
                &target.depth_view,
                &self.scene_bind_group,
                mesh_draws.iter(),
            );
            pop_group(encoder);
            push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
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
            pop_group(encoder);
            if runtime_features.sprite_rendering_enabled {
                record_sprite_stage(
                    &self.sprite_renderer,
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    RenderPassStage::Opaque2d,
                );
            }
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::AlphaMask3d,
                device,
                queue,
                encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                graph_execution,
            )?;
            if runtime_features.sprite_rendering_enabled {
                record_sprite_stage(
                    &self.sprite_renderer,
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    RenderPassStage::AlphaMask2d,
                );
            }
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Transparent3d,
                device,
                queue,
                encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                graph_execution,
            )?;
            if runtime_features.sprite_rendering_enabled {
                record_sprite_stage(
                    &self.sprite_renderer,
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    RenderPassStage::Transparent2d,
                );
            }
            if runtime_features.particle_rendering_enabled {
                push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
                self.particle_renderer.record(
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    frame,
                );
                pop_group(encoder);
            }
        }

        if runtime_features.deferred_lighting_enabled {
            push_group(encoder, RENDERDOC_MARKER_DEFERRED_LIGHTING);
            self.deferred.execute_lighting(
                device,
                encoder,
                &self.scene_bind_group,
                &target.gbuffer_albedo_view,
                &target.normal_view,
                &target.final_color_view,
                &target.scene_color_view,
            );
            pop_group(encoder);
            push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
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
            pop_group(encoder);
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::AlphaMask3d,
                device,
                queue,
                encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                graph_execution,
            )?;
            if runtime_features.sprite_rendering_enabled {
                record_sprite_stage(
                    &self.sprite_renderer,
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    RenderPassStage::AlphaMask2d,
                );
            }
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Transparent3d,
                device,
                queue,
                encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                graph_execution,
            )?;
            if runtime_features.sprite_rendering_enabled {
                record_sprite_stage(
                    &self.sprite_renderer,
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    RenderPassStage::Transparent2d,
                );
            }
            if runtime_features.particle_rendering_enabled {
                push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
                self.particle_renderer.record(
                    device,
                    encoder,
                    &target.scene_color_view,
                    &target.depth_view,
                    &self.scene_bind_group,
                    frame,
                );
                pop_group(encoder);
            }
        }

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn record_sprite_stage(
    renderer: &SpriteRenderer,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    color_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    scene_bind_group: &wgpu::BindGroup,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    stage: RenderPassStage,
) {
    push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
    renderer.record(
        device,
        encoder,
        color_view,
        depth_view,
        scene_bind_group,
        streamer,
        frame,
        stage,
    );
    pop_group(encoder);
}
