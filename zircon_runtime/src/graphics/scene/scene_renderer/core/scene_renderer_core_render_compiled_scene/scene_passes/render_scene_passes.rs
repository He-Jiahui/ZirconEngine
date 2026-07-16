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
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::CompiledRenderPipeline;

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::environment::IblBakeWgpuPipelineCache;
use super::super::super::super::mesh::MeshPipelineCache;
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::super::shadow::atlas::ShadowAtlasResources;
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
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Deferred,
                Some(streamer),
                self.hzb_occlusion_culler.as_ref(),
                None,
                None,
                Some(&self.shadow_atlas_resources),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    &mut self.ibl_bake_pipeline_cache,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    &self.scene_bind_group_layout,
                    self.scene_color_format,
                    self.depth_format,
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
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                None,
                RenderPassStage::Opaque3d,
                None,
                self.hzb_occlusion_culler.as_ref(),
                None,
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    &mut self.ibl_bake_pipeline_cache,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    &self.scene_bind_group_layout,
                    self.scene_color_format,
                    self.depth_format,
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
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                None,
                RenderPassStage::AlphaMask3d,
                None,
                self.hzb_occlusion_culler.as_ref(),
                None,
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    &mut self.ibl_bake_pipeline_cache,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    &self.scene_bind_group_layout,
                    self.scene_color_format,
                    self.depth_format,
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
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                Some(&mut self.overlay_renderer),
                RenderPassStage::Transparent3d,
                Some(&self.sprite_renderer),
                self.hzb_occlusion_culler.as_ref(),
                Some(&self.particle_renderer),
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    &mut self.ibl_bake_pipeline_cache,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    &self.scene_bind_group_layout,
                    self.scene_color_format,
                    self.depth_format,
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
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                RenderPassStage::Lighting,
                None,
                self.hzb_occlusion_culler.as_ref(),
                Some(post_process_stack),
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
            );
            pop_group(encoder);
            deferred_lighting_result?;
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                encoder,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                &mut self.screen_space_ui_renderer,
                Some(&mut self.overlay_renderer),
                RenderPassStage::Transparent3d,
                Some(&self.sprite_renderer),
                self.hzb_occlusion_culler.as_ref(),
                Some(&self.particle_renderer),
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
            )?;
            if runtime_features.sprite_rendering_enabled {
                execute_sprite_graph_stage(
                    &self.sprite_renderer,
                    &mut self.ibl_bake_pipeline_cache,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    &self.scene_bind_group_layout,
                    self.scene_color_format,
                    self.depth_format,
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
                    &mut self.ibl_bake_pipeline_cache,
                    device,
                    queue,
                    encoder,
                    &self.scene_bind_group,
                    &self.scene_bind_group_layout,
                    self.scene_color_format,
                    self.depth_format,
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
    ibl_bake_pipeline_cache: &mut IblBakeWgpuPipelineCache,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene_bind_group: &wgpu::BindGroup,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    screen_space_ui_renderer: &mut crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer,
    overlay_renderer: Option<
        &mut crate::graphics::scene::scene_renderer::overlay::ViewportOverlayRenderer,
    >,
    stage: RenderPassStage,
    sprite_renderer: Option<&SpriteRenderer>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    particle_renderer: Option<&ParticleRenderer>,
    shadow_map_renderer: Option<&crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
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
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        screen_space_ui_renderer,
        None,
        overlay_renderer,
        None,
        None,
        particle_renderer,
        sprite_renderer,
        Some(streamer),
        Some(mesh_pipelines),
        Some(ibl_bake_pipeline_cache),
        Some(mesh_draw_lists),
        hzb_occlusion_culler,
        shadow_map_renderer,
        shadow_atlas_resources,
        None,
        graph_execution,
    );
    pop_group(encoder);
    result
}

#[allow(clippy::too_many_arguments)]
fn execute_deferred_graph_stage(
    deferred: &DeferredSceneResources,
    mesh_pipelines: &mut MeshPipelineCache,
    ibl_bake_pipeline_cache: &mut IblBakeWgpuPipelineCache,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene_bind_group: &wgpu::BindGroup,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    screen_space_ui_renderer: &mut crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer,
    stage: RenderPassStage,
    streamer: Option<&ResourceStreamer>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    shadow_map_renderer: Option<&crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
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
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        screen_space_ui_renderer,
        post_process_stack,
        None,
        None,
        Some(deferred),
        None,
        None,
        streamer,
        Some(mesh_pipelines),
        Some(ibl_bake_pipeline_cache),
        Some(mesh_draw_lists),
        hzb_occlusion_culler,
        shadow_map_renderer,
        shadow_atlas_resources,
        None,
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
    ibl_bake_pipeline_cache: &mut IblBakeWgpuPipelineCache,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene_bind_group: &wgpu::BindGroup,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
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
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        screen_space_ui_renderer,
        None,
        None,
        None,
        None,
        None,
        Some(renderer),
        Some(streamer),
        None,
        Some(ibl_bake_pipeline_cache),
        None,
        None,
        None,
        None,
        None,
        graph_execution,
    );
    pop_group(encoder);
    result
}
