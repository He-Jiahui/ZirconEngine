use super::super::super::hybrid_gi::HybridGiGpuResources;
use super::super::super::mesh::{
    build_mesh_draws, BuiltMeshDraws, VirtualGeometryIndirectArgsGpuResources,
};
use super::super::super::virtual_geometry::VirtualGeometryGpuResources;
use super::advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

pub(crate) struct SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) hybrid_gi: HybridGiGpuResources,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry:
        VirtualGeometryGpuResources,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_args:
        VirtualGeometryIndirectArgsGpuResources,
}

impl SceneRendererAdvancedPluginResources {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            hybrid_gi: HybridGiGpuResources::new(device),
            virtual_geometry: VirtualGeometryGpuResources::new(device),
            virtual_geometry_indirect_args: VirtualGeometryIndirectArgsGpuResources::new(device),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn build_mesh_draws(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        model_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
    ) -> BuiltMeshDraws {
        build_mesh_draws(
            device,
            encoder,
            &self.virtual_geometry_indirect_args,
            model_layout,
            streamer,
            frame,
            virtual_geometry_enabled,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        let hybrid_gi_gpu_readback = self.hybrid_gi.execute_prepare(
            device,
            queue,
            encoder,
            streamer,
            frame.hybrid_gi_prepare.as_ref(),
            frame.hybrid_gi_scene_prepare.as_ref(),
            frame.hybrid_gi_resolve_runtime.as_ref(),
            frame.extract.lighting.hybrid_global_illumination.as_ref(),
            &frame.extract.geometry.meshes,
            &frame.extract.lighting.directional_lights,
            &frame.extract.lighting.point_lights,
            &frame.extract.lighting.spot_lights,
            frame
                .extract
                .lighting
                .hybrid_global_illumination
                .as_ref()
                .map(|hybrid_gi| hybrid_gi.probe_budget),
            frame
                .extract
                .lighting
                .hybrid_global_illumination
                .as_ref()
                .map(|hybrid_gi| hybrid_gi.tracing_budget),
        )?;
        let virtual_geometry_gpu_readback = self.virtual_geometry.execute_prepare(
            device,
            queue,
            encoder,
            frame.virtual_geometry_prepare.as_ref(),
            frame
                .extract
                .geometry
                .virtual_geometry
                .as_ref()
                .map(|virtual_geometry| virtual_geometry.page_budget),
        )?;

        Ok(SceneRendererAdvancedPluginReadbacks {
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
        })
    }
}
