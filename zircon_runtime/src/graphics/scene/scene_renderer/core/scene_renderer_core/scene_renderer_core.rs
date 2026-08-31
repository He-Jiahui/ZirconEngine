use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::types::GraphicsError;
use zr_rhi::{DeviceGeneration, DeviceId};

use super::super::super::deferred::DeferredSceneResources;
use super::super::super::environment::ibl_bake_runtime_writeback::IblBakeRuntimeGraphWritebackQueue;
use super::super::super::environment::realtime_ibl_capture_wgpu::RealtimeIblCaptureWgpuPipelines;
use super::super::super::environment::realtime_ibl_time_slice::IblRealtimeBufferSlot;
use super::super::super::environment::{IblBakeWgpuPipelineCache, RealtimeIblRuntime};
use super::super::super::graph_execution::TransientResourcePool;
use super::super::super::hzb::HzbOcclusionCuller;
use super::super::super::mesh::{
    CachedMeshDrawCommands, MeshIndirectDrawWorkspace, MeshPipelineCache,
};
use super::super::super::overlay::ViewportOverlayRenderer;
use super::super::super::particle::ParticleRenderer;
use super::super::super::post_process::ScenePostProcessResources;
use super::super::super::scene_clear::SceneRegionClearResources;
use super::super::super::shadow::ShadowMapRenderer;
use super::super::super::shadow::atlas::{ShadowAtlasAllocator, ShadowAtlasResources};
use super::super::super::sprite::SpriteRenderer;
use super::super::super::ui::ScreenSpaceUiRenderer;
use super::super::SceneRendererDeferredLightingProfile;
use super::{
    SceneEnvironmentBrdfLut, SceneEnvironmentCubemap, SceneHitProxyResources,
    SceneRendererAdvancedPluginResources, SceneRendererNeutralGraphBuffers,
};

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCore {
    /// Identity of the native generation used to construct every persistent core resource.
    ///
    /// The core currently has no in-place device replacement path, so render entry points must
    /// reject a changed backend epoch before touching any retained WGPU object.
    pub(in crate::graphics::scene::scene_renderer::core) device_id: DeviceId,
    pub(in crate::graphics::scene::scene_renderer::core) device_generation: DeviceGeneration,
    pub(in crate::graphics::scene::scene_renderer::core) texture_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) scene_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) scene_uniform_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::core) scene_environment_sh9_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::core) scene_environment_cubemap:
        SceneEnvironmentCubemap,
    pub(in crate::graphics::scene::scene_renderer::core) scene_environment_brdf_lut:
        SceneEnvironmentBrdfLut,
    pub(in crate::graphics::scene::scene_renderer::core) scene_bind_group: wgpu::BindGroup,
    pub(in crate::graphics::scene::scene_renderer::core) scene_color_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core) final_color_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core) depth_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core) global_material_mip_bias: f32,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_command_generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) material_texture_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_pipelines: MeshPipelineCache,
    pub(in crate::graphics::scene::scene_renderer::core) ibl_bake_pipeline_cache:
        IblBakeWgpuPipelineCache,
    pub(in crate::graphics::scene::scene_renderer::core) environment_capture_mip_pipelines:
        RealtimeIblCaptureWgpuPipelines,
    pub(in crate::graphics::scene::scene_renderer::core) realtime_ibl: RealtimeIblRuntime,
    pub(in crate::graphics::scene::scene_renderer::core) scene_bind_group_realtime_ibl_slot:
        Option<IblRealtimeBufferSlot>,
    pub(in crate::graphics::scene::scene_renderer::core) cached_mesh_draw_commands:
        CachedMeshDrawCommands,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_indirect_draw_workspace:
        MeshIndirectDrawWorkspace,
    pub(in crate::graphics::scene::scene_renderer::core) neutral_graph_buffers:
        SceneRendererNeutralGraphBuffers,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_scene: GpuScene,
    pub(in crate::graphics::scene::scene_renderer::core) hit_proxy_resources:
        SceneHitProxyResources,
    pub(in crate::graphics::scene::scene_renderer::core) hzb_occlusion_culler:
        Option<HzbOcclusionCuller>,
    pub(in crate::graphics::scene::scene_renderer::core) scene_clear:
        Option<SceneRegionClearResources>,
    pub(in crate::graphics::scene::scene_renderer::core) deferred_lighting_profile:
        SceneRendererDeferredLightingProfile,
    pub(in crate::graphics::scene::scene_renderer::core) shadow_map_renderer:
        Option<ShadowMapRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core) shadow_atlas_allocator:
        ShadowAtlasAllocator,
    pub(in crate::graphics::scene::scene_renderer::core) shadow_atlas_resources:
        ShadowAtlasResources,
    pub(in crate::graphics::scene::scene_renderer::core) deferred: DeferredSceneResources,
    pub(in crate::graphics::scene::scene_renderer::core) particle_renderer:
        Option<ParticleRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core) sprite_renderer: Option<SpriteRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core) post_process: ScenePostProcessResources,
    pub(in crate::graphics::scene::scene_renderer::core) overlay_renderer: ViewportOverlayRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) screen_space_ui_renderer:
        Option<ScreenSpaceUiRenderer>,
    pub(in crate::graphics::scene::scene_renderer::core) transient_resource_pool:
        TransientResourcePool,
    pub(in crate::graphics::scene::scene_renderer::core) diagnostic_frame_index: u64,
    pub(in crate::graphics::scene::scene_renderer::core) ibl_bake_runtime_writebacks:
        IblBakeRuntimeGraphWritebackQueue,
    pub(in crate::graphics::scene::scene_renderer::core) advanced_plugin_resources:
        SceneRendererAdvancedPluginResources,
}

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core) fn ensure_device_epoch(
        &self,
        backend: &RenderBackend,
    ) -> Result<(), GraphicsError> {
        let profile = backend.device_profile();
        let actual_device_id = profile.device_id();
        let actual_generation = profile.generation();
        if actual_device_id == self.device_id && actual_generation == self.device_generation {
            return Ok(());
        }

        Err(GraphicsError::SceneRendererDeviceEpochMismatch {
            expected_device_id: self.device_id,
            expected_generation: self.device_generation,
            actual_device_id,
            actual_generation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SceneRendererDeferredLightingProfile;

    #[test]
    fn renderer_core_imports_the_default_deferred_lighting_profile() {
        assert_eq!(
            SceneRendererDeferredLightingProfile::default(),
            SceneRendererDeferredLightingProfile::FullScene
        );
    }

    #[test]
    fn renderer_core_guards_all_render_entrypoints_with_its_construction_epoch() {
        let core = include_str!("scene_renderer_core.rs");
        assert!(core.contains("device_id: DeviceId"));
        assert!(core.contains("device_generation: DeviceGeneration"));
        assert!(core.contains("GraphicsError::SceneRendererDeviceEpochMismatch"));

        let direct = include_str!("../scene_renderer_core_render_scene/render_scene.rs");
        let direct_guard = direct
            .find("self.ensure_device_epoch(backend)?;")
            .expect("direct render must admit the core device epoch");
        let direct_device = direct
            .find("let device = &backend.device;")
            .expect("direct render device borrow");
        assert!(direct_guard < direct_device);

        let compiled =
            include_str!("../scene_renderer_core_render_compiled_scene/render/render.rs");
        let compiled_guard = compiled
            .find("self.ensure_device_epoch(backend)?;")
            .expect("compiled render must admit the core device epoch");
        let compiled_device = compiled
            .find("let device = &backend.device;")
            .expect("compiled render device borrow");
        assert!(compiled_guard < compiled_device);
    }
}
