use crate::core::TaskPool;
use crate::graphics::CompiledRenderPipeline;
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
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryAvailability,
};
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::mesh::MeshPipelineCache;
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::super::shadow::atlas::ShadowAtlasResources;
use super::super::super::super::sprite::SpriteRenderer;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::render::RenderGraphPassFrameServices;
use super::super::render::execute_graph_stage::{RenderGraphStageExecution, execute_graph_stage};

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn render_scene_passes(
        &mut self,
        device: &wgpu::Device,
        command_encoders: &mut FrameCommandEncoderSet,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        runtime_features: SceneRuntimeFeatureFlags,
        pipeline: &CompiledRenderPipeline,
        render_pass_executors: &RenderPassExecutorRegistry,
        graph_execution: &mut RenderGraphStageExecution<'_>,
        mesh_draw_lists: RenderPassMeshCommandLists<'_>,
        history_textures: Option<&SceneFrameHistoryTextures>,
        history_availability: SceneHistoryAvailability,
        parallel_recording: Option<(&TaskPool, usize)>,
    ) -> Result<(), GraphicsError> {
        if runtime_features.deferred_lighting_enabled {
            execute_deferred_graph_stage(
                &self.deferred,
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
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
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
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
                self.shadow_map_renderer.as_ref(),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
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
            let ambient_occlusion_post_process_stack = RenderPassPostProcessStackContext::new(
                &self.post_process,
                streamer,
                runtime_features,
                history_textures,
                history_availability,
            )
            .with_material_gbuffer_valid(runtime_features.deferred_lighting_enabled);
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::AmbientOcclusion,
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
                    post_process_stack: Some(ambient_occlusion_post_process_stack),
                    overlay_renderer: None,
                    prepared_overlays: None,
                    deferred: None,
                    particle_renderer: None,
                    sprite_renderer: None,
                    streamer: None,
                    ibl_bake_pipeline_cache: Some(&mut self.ibl_bake_pipeline_cache),
                    mesh_pipelines: None,
                    mesh_draw_lists: None,
                    hzb_occlusion_culler: self.hzb_occlusion_culler.as_ref(),
                    shadow_map_renderer: None,
                    shadow_atlas_resources: Some(&self.shadow_atlas_resources),
                    shadow_frame_plan: None,
                    parallel_recording,
                },
                graph_execution,
            )?;
        } else {
            execute_mesh_graph_stage(
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
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
                self.shadow_map_renderer.as_ref(),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
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
                self.shadow_map_renderer.as_ref(),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
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
                self.shadow_map_renderer.as_ref(),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
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
                streamer,
                runtime_features,
                history_textures,
                history_availability,
            );
            let deferred_lighting_result = execute_deferred_graph_stage(
                &self.deferred,
                &mut self.mesh_pipelines,
                &mut self.ibl_bake_pipeline_cache,
                mesh_draw_lists,
                device,
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
                self.shadow_map_renderer.as_ref(),
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
                self.shadow_map_renderer.as_ref(),
                Some(&self.shadow_atlas_resources),
                parallel_recording,
            )?;
            if runtime_features.sprite_rendering_enabled {
                if let Some(sprite_renderer) = self.sprite_renderer.as_ref() {
                    execute_sprite_graph_stage(
                        sprite_renderer,
                        &mut self.ibl_bake_pipeline_cache,
                        device,
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
        RenderGraphPassFrameServices {
            device,
            command_encoders,
            frame,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            surface_frame: None,
            screen_space_ui_renderer: None,
            post_process_stack: None,
            overlay_renderer,
            prepared_overlays: None,
            deferred: None,
            particle_renderer,
            sprite_renderer,
            streamer: Some(streamer),
            ibl_bake_pipeline_cache: Some(ibl_bake_pipeline_cache),
            mesh_pipelines: Some(mesh_pipelines),
            mesh_draw_lists: Some(mesh_draw_lists),
            hzb_occlusion_culler,
            shadow_map_renderer,
            shadow_atlas_resources,
            shadow_frame_plan: None,
            parallel_recording,
        },
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
        RenderGraphPassFrameServices {
            device,
            command_encoders,
            frame,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            surface_frame: None,
            screen_space_ui_renderer: None,
            post_process_stack,
            overlay_renderer: None,
            prepared_overlays: None,
            deferred: Some(deferred),
            particle_renderer: None,
            sprite_renderer: None,
            streamer,
            ibl_bake_pipeline_cache: Some(ibl_bake_pipeline_cache),
            mesh_pipelines: Some(mesh_pipelines),
            mesh_draw_lists: Some(mesh_draw_lists),
            hzb_occlusion_culler,
            shadow_map_renderer,
            shadow_atlas_resources,
            shadow_frame_plan: None,
            parallel_recording,
        },
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
        RenderGraphPassFrameServices {
            device,
            command_encoders,
            frame,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            surface_frame: None,
            screen_space_ui_renderer: None,
            post_process_stack: None,
            overlay_renderer: None,
            prepared_overlays: None,
            deferred: None,
            particle_renderer: None,
            sprite_renderer: Some(renderer),
            streamer: Some(streamer),
            ibl_bake_pipeline_cache: Some(ibl_bake_pipeline_cache),
            mesh_pipelines: None,
            mesh_draw_lists: None,
            hzb_occlusion_culler: None,
            shadow_map_renderer: None,
            shadow_atlas_resources: None,
            shadow_frame_plan: None,
            parallel_recording,
        },
        graph_execution,
    )
}

#[cfg(test)]
mod tests {
    use crate::graphics::pipeline::RenderPassStage;

    #[test]
    fn deferred_scene_executes_ambient_occlusion_between_gbuffer_and_lighting() {
        let source = include_str!("render_scene_passes.rs");
        let deferred = source
            .find("RenderPassStage::Deferred,")
            .expect("deferred GBuffer stage");
        let ambient_occlusion = source[deferred..]
            .find("RenderPassStage::AmbientOcclusion,")
            .map(|offset| deferred + offset)
            .expect("deferred ambient-occlusion stage");
        let lighting = source[ambient_occlusion..]
            .find("RenderPassStage::Lighting,")
            .map(|offset| ambient_occlusion + offset)
            .expect("deferred lighting stage");

        assert!(deferred < ambient_occlusion);
        assert!(ambient_occlusion < lighting);

        let alpha_mask = source
            .find("RenderPassStage::AlphaMask3d,")
            .expect("deferred alpha-mask stage");
        assert!(deferred < alpha_mask);
        assert!(alpha_mask < ambient_occlusion);
    }
}
