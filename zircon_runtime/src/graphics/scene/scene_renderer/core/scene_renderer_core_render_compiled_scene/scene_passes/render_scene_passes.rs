use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    pop_group, push_group, RENDERDOC_MARKER_DEFERRED_LIGHTING, RENDERDOC_MARKER_MAIN_SCENE,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutorRegistry, RenderPassMeshCommandLists, RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::CompiledRenderPipeline;

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::mesh::MeshPipelineCache;
use super::super::super::super::particle::ParticleRenderer;
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
        target: &OffscreenTarget,
        runtime_features: SceneRuntimeFeatureFlags,
        pipeline: &CompiledRenderPipeline,
        render_pass_executors: &RenderPassExecutorRegistry,
        graph_execution: &mut RenderGraphStageExecution<'_>,
        mesh_draw_lists: RenderPassMeshCommandLists<'_>,
        history_textures: Option<&SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Result<(), GraphicsError> {
        if runtime_features.deferred_lighting_enabled {
            execute_deferred_graph_stage(
                &self.deferred,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Deferred,
                None,
                None,
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    pipeline,
                    render_pass_executors,
                    graph_execution,
                    &mut self.screen_space_ui_renderer,
                    RenderPassStage::Opaque2d,
                )?;
            }
        } else {
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Opaque3d,
                None,
                Some(&self.shadow_map_renderer),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    pipeline,
                    render_pass_executors,
                    graph_execution,
                    &mut self.screen_space_ui_renderer,
                    RenderPassStage::Opaque2d,
                )?;
            }
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::AlphaMask3d,
                None,
                Some(&self.shadow_map_renderer),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    pipeline,
                    render_pass_executors,
                    graph_execution,
                    &mut self.screen_space_ui_renderer,
                    RenderPassStage::AlphaMask2d,
                )?;
            }
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Transparent3d,
                Some(&self.particle_renderer),
                Some(&self.shadow_map_renderer),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    pipeline,
                    render_pass_executors,
                    graph_execution,
                    &mut self.screen_space_ui_renderer,
                    RenderPassStage::Transparent2d,
                )?;
            }
        }

        if runtime_features.deferred_lighting_enabled {
            push_group(encoder, RENDERDOC_MARKER_DEFERRED_LIGHTING);
            let post_process_stack = RenderPassPostProcessStackContext::new(
                &self.post_process,
                target,
                streamer,
                runtime_features,
                history_textures,
                history_available,
            );
            let deferred_lighting_result = execute_deferred_graph_stage(
                &self.deferred,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Lighting,
                Some(post_process_stack),
                Some(&self.shadow_map_renderer),
            );
            pop_group(encoder);
            deferred_lighting_result?;
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Transparent3d,
                Some(&self.particle_renderer),
                Some(&self.shadow_map_renderer),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    pipeline,
                    render_pass_executors,
                    graph_execution,
                    &mut self.screen_space_ui_renderer,
                    RenderPassStage::AlphaMask2d,
                )?;
            }
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    streamer,
                    frame,
                    pipeline,
                    render_pass_executors,
                    graph_execution,
                    &mut self.screen_space_ui_renderer,
                    RenderPassStage::Transparent2d,
                )?;
            }
        }

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_mesh_graph_stage(
    mesh_pipelines: &mut MeshPipelineCache,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene_bind_group: &wgpu::BindGroup,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    screen_space_ui_renderer: &mut crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer,
    stage: RenderPassStage,
    particle_renderer: Option<&ParticleRenderer>,
    shadow_map_renderer: Option<&crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer>,
) -> Result<(), GraphicsError> {
    push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
    let result = execute_graph_stage(
        pipeline,
        render_pass_executors,
        stage,
        device,
        queue,
        encoder,
        frame,
        scene_bind_group,
        screen_space_ui_renderer,
        None,
        None,
        None,
        None,
        None,
        particle_renderer,
        None,
        Some(streamer),
        Some(mesh_pipelines),
        Some(mesh_draw_lists),
        shadow_map_renderer,
        graph_execution,
    );
    pop_group(encoder);
    result
}

#[allow(clippy::too_many_arguments)]
fn execute_deferred_graph_stage(
    deferred: &DeferredSceneResources,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene_bind_group: &wgpu::BindGroup,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    screen_space_ui_renderer: &mut crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer,
    stage: RenderPassStage,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    shadow_map_renderer: Option<&crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer>,
) -> Result<(), GraphicsError> {
    let pushes_main_scene_group = matches!(stage, RenderPassStage::Deferred);
    if pushes_main_scene_group {
        push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
    }
    let result = execute_graph_stage(
        pipeline,
        render_pass_executors,
        stage,
        device,
        queue,
        encoder,
        frame,
        scene_bind_group,
        screen_space_ui_renderer,
        post_process_stack,
        None,
        None,
        None,
        Some(deferred),
        None,
        None,
        None,
        None,
        Some(mesh_draw_lists),
        shadow_map_renderer,
        graph_execution,
    );
    if pushes_main_scene_group {
        pop_group(encoder);
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn execute_sprite_graph_stage(
    renderer: &SpriteRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene_bind_group: &wgpu::BindGroup,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    screen_space_ui_renderer: &mut crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer,
    stage: RenderPassStage,
) -> Result<(), GraphicsError> {
    push_group(encoder, RENDERDOC_MARKER_MAIN_SCENE);
    let result = execute_graph_stage(
        pipeline,
        render_pass_executors,
        stage,
        device,
        queue,
        encoder,
        frame,
        scene_bind_group,
        screen_space_ui_renderer,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(renderer),
        Some(streamer),
        None,
        None,
        None,
        graph_execution,
    );
    pop_group(encoder);
    result
}
