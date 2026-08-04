use crate::core::TaskPool;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    RENDERDOC_MARKER_DEFERRED_LIGHTING, RENDERDOC_MARKER_MAIN_SCENE, pop_group, push_group,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::environment::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderPassExecutorRegistry, RenderPassMeshCommandLists,
    RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::mesh::MeshPipelineCache;
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::super::shadow::atlas::ShadowAtlasResources;
use super::super::super::super::sprite::SpriteRenderer;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::render::execute_graph_stage::{RenderGraphStageExecution, execute_graph_stage};

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn render_scene_passes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_encoders: &mut FrameCommandEncoderSet,
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
        parallel_recording: Option<(&TaskPool, usize)>,
    ) -> Result<(), GraphicsError> {
        if runtime_features.deferred_lighting_enabled {
            execute_deferred_graph_stage(
                &self.deferred,
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                command_encoders,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                RenderPassStage::Deferred,
                Some(streamer),
                self.hzb_occlusion_culler.as_ref(),
                None,
                None,
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
                        queue,
                        command_encoders,
                        &self.scene_bind_group,
                        &self.scene_bind_group_layout,
                        self.scene_color_format,
                        self.depth_format,
                        streamer,
                        frame,
                        pipeline,
                        render_pass_executors,
                        graph_execution,
                        RenderPassStage::Opaque2d,
                        parallel_recording,
                    )?;
                }
            }
        } else {
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                command_encoders,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                None,
                RenderPassStage::Opaque3d,
                None,
                self.hzb_occlusion_culler.as_ref(),
                None,
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
                        queue,
                        command_encoders,
                        &self.scene_bind_group,
                        &self.scene_bind_group_layout,
                        self.scene_color_format,
                        self.depth_format,
                        streamer,
                        frame,
                        pipeline,
                        render_pass_executors,
                        graph_execution,
                        RenderPassStage::Opaque2d,
                        parallel_recording,
                    )?;
                }
            }
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                command_encoders,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                None,
                RenderPassStage::AlphaMask3d,
                None,
                self.hzb_occlusion_culler.as_ref(),
                None,
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
                        queue,
                        command_encoders,
                        &self.scene_bind_group,
                        &self.scene_bind_group_layout,
                        self.scene_color_format,
                        self.depth_format,
                        streamer,
                        frame,
                        pipeline,
                        render_pass_executors,
                        graph_execution,
                        RenderPassStage::AlphaMask2d,
                        parallel_recording,
                    )?;
                }
            }
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                command_encoders,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                Some(&mut self.overlay_renderer),
                RenderPassStage::Transparent3d,
                self.sprite_renderer.as_ref(),
                self.hzb_occlusion_culler.as_ref(),
                self.particle_renderer.as_ref(),
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
                        queue,
                        command_encoders,
                        &self.scene_bind_group,
                        &self.scene_bind_group_layout,
                        self.scene_color_format,
                        self.depth_format,
                        streamer,
                        frame,
                        pipeline,
                        render_pass_executors,
                        graph_execution,
                        RenderPassStage::Transparent2d,
                        parallel_recording,
                    )?;
                }
            }
        }

        if runtime_features.deferred_lighting_enabled {
            push_group(
                command_encoders.serial_encoder(device),
                RENDERDOC_MARKER_DEFERRED_LIGHTING,
            );
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
                command_encoders,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                RenderPassStage::Lighting,
                None,
                self.hzb_occlusion_culler.as_ref(),
                Some(post_process_stack),
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            );
            pop_group(command_encoders.serial_encoder(device));
            deferred_lighting_result?;
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
                queue,
                command_encoders,
                &self.scene_bind_group,
                &self.scene_bind_group_layout,
                self.scene_color_format,
                self.depth_format,
                streamer,
                frame,
                pipeline,
                render_pass_executors,
                graph_execution,
                Some(&mut self.overlay_renderer),
                RenderPassStage::Transparent3d,
                self.sprite_renderer.as_ref(),
                self.hzb_occlusion_culler.as_ref(),
                self.particle_renderer.as_ref(),
                Some(&self.shadow_map_renderer),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
                        queue,
                        command_encoders,
                        &self.scene_bind_group,
                        &self.scene_bind_group_layout,
                        self.scene_color_format,
                        self.depth_format,
                        streamer,
                        frame,
                        pipeline,
                        render_pass_executors,
                        graph_execution,
                        RenderPassStage::AlphaMask2d,
                        parallel_recording,
                    )?;
                }
            }
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
                        queue,
                        command_encoders,
                        &self.scene_bind_group,
                        &self.scene_bind_group_layout,
                        self.scene_color_format,
                        self.depth_format,
                        streamer,
                        frame,
                        pipeline,
                        render_pass_executors,
                        graph_execution,
                        RenderPassStage::Transparent2d,
                        parallel_recording,
                    )?;
                }
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
    command_encoders: &mut FrameCommandEncoderSet,
    scene_bind_group: &wgpu::BindGroup,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    overlay_renderer: Option<
        &mut crate::graphics::scene::scene_renderer::overlay::ViewportOverlayRenderer,
    >,
    stage: RenderPassStage,
    sprite_renderer: Option<&SpriteRenderer>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    particle_renderer: Option<&ParticleRenderer>,
    shadow_map_renderer: Option<&crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
    parallel_recording: Option<(&TaskPool, usize)>,
) -> Result<(), GraphicsError> {
    push_group(
        command_encoders.serial_encoder(device),
        RENDERDOC_MARKER_MAIN_SCENE,
    );
    let result = execute_graph_stage(
        pipeline,
        render_pass_executors,
        stage,
        device,
        queue,
        command_encoders,
        frame,
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        None,
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
        parallel_recording,
        graph_execution,
    );
    pop_group(command_encoders.serial_encoder(device));
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
    command_encoders: &mut FrameCommandEncoderSet,
    scene_bind_group: &wgpu::BindGroup,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    stage: RenderPassStage,
    streamer: Option<&ResourceStreamer>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    shadow_map_renderer: Option<&crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
    parallel_recording: Option<(&TaskPool, usize)>,
) -> Result<(), GraphicsError> {
    let pushes_main_scene_group = matches!(stage, RenderPassStage::Deferred);
    if pushes_main_scene_group {
        push_group(
            command_encoders.serial_encoder(device),
            RENDERDOC_MARKER_MAIN_SCENE,
        );
    }
    let result = execute_graph_stage(
        pipeline,
        render_pass_executors,
        stage,
        device,
        queue,
        command_encoders,
        frame,
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        None,
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
        parallel_recording,
        graph_execution,
    );
    if pushes_main_scene_group {
        pop_group(command_encoders.serial_encoder(device));
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn execute_sprite_graph_stage(
    renderer: &SpriteRenderer,
    ibl_bake_pipeline_cache: &mut IblBakeWgpuPipelineCache,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    command_encoders: &mut FrameCommandEncoderSet,
    scene_bind_group: &wgpu::BindGroup,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    pipeline: &CompiledRenderPipeline,
    render_pass_executors: &RenderPassExecutorRegistry,
    graph_execution: &mut RenderGraphStageExecution<'_>,
    stage: RenderPassStage,
    parallel_recording: Option<(&TaskPool, usize)>,
) -> Result<(), GraphicsError> {
    execute_graph_stage(
        pipeline,
        render_pass_executors,
        stage,
        device,
        queue,
        command_encoders,
        frame,
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        None,
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
        parallel_recording,
        graph_execution,
    )
}
