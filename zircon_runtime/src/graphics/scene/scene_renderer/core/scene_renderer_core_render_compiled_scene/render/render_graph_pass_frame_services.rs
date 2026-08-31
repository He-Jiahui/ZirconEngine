use crate::core::TaskPool;
use crate::graphics::backend::ViewportSurface;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::environment::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderPassMeshCommandLists, RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::shadow::{ShadowFramePlan, ShadowMapRenderer};
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::types::ViewportRenderFrame;

/// Frame-scoped services selected by the caller for a compiled graph pass domain.
///
/// The execution packet supplies graph order; this DTO only carries renderer services
/// and must not encode stage ordering or resource scheduling policy.
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene)
struct RenderGraphPassFrameServices
<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) device:
        &'a wgpu::Device,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) command_encoders:
        &'a mut FrameCommandEncoderSet,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) frame:
        &'a ViewportRenderFrame,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) scene_bind_group_layout:
        &'a wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) target_format:
        wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) depth_format:
        wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) scene_bind_group:
        &'a wgpu::BindGroup,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) surface_frame:
        Option<(
            &'a ViewportSurface,
            &'a zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget,
        )>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) screen_space_ui_renderer:
        Option<&'a mut ScreenSpaceUiRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) post_process_stack:
        Option<RenderPassPostProcessStackContext<'a>>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) overlay_renderer:
        Option<&'a mut ViewportOverlayRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) prepared_overlays:
        Option<&'a PreparedOverlayBuffers>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) deferred:
        Option<&'a DeferredSceneResources>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) particle_renderer:
        Option<&'a ParticleRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) sprite_renderer:
        Option<&'a SpriteRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) streamer:
        Option<&'a ResourceStreamer>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) ibl_bake_pipeline_cache:
        Option<&'a mut IblBakeWgpuPipelineCache>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) mesh_pipelines:
        Option<&'a mut MeshPipelineCache>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) mesh_draw_lists:
        Option<RenderPassMeshCommandLists<'a>>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) hzb_occlusion_culler:
        Option<&'a HzbOcclusionCuller>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) shadow_map_renderer:
        Option<&'a ShadowMapRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) shadow_atlas_resources:
        Option<&'a ShadowAtlasResources>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) shadow_frame_plan:
        Option<&'a ShadowFramePlan>,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) parallel_recording:
        Option<(&'a TaskPool, usize)>,
}
