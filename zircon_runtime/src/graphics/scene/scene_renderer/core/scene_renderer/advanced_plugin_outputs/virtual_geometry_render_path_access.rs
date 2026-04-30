use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
};

use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

#[cfg_attr(not(test), allow(dead_code))]
impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_debug_snapshot(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshot> {
        self.virtual_geometry_render_path().debug_snapshot()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_selected_cluster_source(
        &self,
    ) -> RenderVirtualGeometrySelectedClusterSource {
        self.virtual_geometry_render_path()
            .selected_cluster_source()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_selected_cluster_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_render_path().selected_cluster_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_selected_cluster_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_render_path()
            .selected_cluster_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_visbuffer64_clear_value(
        &self,
    ) -> u64 {
        self.virtual_geometry_render_path()
            .visbuffer64_clear_value()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_visbuffer64_source(
        &self,
    ) -> RenderVirtualGeometryVisBuffer64Source {
        self.virtual_geometry_render_path().visbuffer64_source()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_visbuffer64_entry_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_render_path()
            .visbuffer64_entry_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_visbuffer64_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_render_path().visbuffer64_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_hardware_rasterization_source(
        &self,
    ) -> RenderVirtualGeometryHardwareRasterizationSource {
        self.virtual_geometry_render_path()
            .hardware_rasterization_source()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_hardware_rasterization_record_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_render_path()
            .hardware_rasterization_record_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_hardware_rasterization_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_render_path()
            .hardware_rasterization_buffer()
    }
}
