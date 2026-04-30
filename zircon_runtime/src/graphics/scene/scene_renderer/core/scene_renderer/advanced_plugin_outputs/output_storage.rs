use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::graphics::scene::scene_renderer::core::scene_renderer::{
    VirtualGeometryCullOutputUpdate, VirtualGeometryIndirectOutputUpdate,
    VirtualGeometryLastOutputUpdate, VirtualGeometryRenderPathOutputUpdate,
};
use crate::graphics::scene::scene_renderer::{HybridGiGpuReadback, VirtualGeometryGpuReadback};

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn store_hybrid_gi_gpu_readback(
        &mut self,
        readback: Option<HybridGiGpuReadback>,
    ) {
        self.hybrid_gi_readback_mut().store_gpu_readback(readback);
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_last_outputs(
        &mut self,
        update: VirtualGeometryLastOutputUpdate,
    ) {
        let VirtualGeometryLastOutputUpdate {
            node_and_cluster_cull,
            render_path,
            indirect,
        } = update;

        self.store_virtual_geometry_cull_outputs(node_and_cluster_cull);
        self.store_virtual_geometry_render_path_outputs(render_path);
        self.store_virtual_geometry_indirect_outputs(indirect);
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_gpu_readback(
        &mut self,
        readback: Option<VirtualGeometryGpuReadback>,
    ) {
        self.virtual_geometry_readback_mut()
            .store_gpu_readback(readback);
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_cull_outputs(
        &mut self,
        update: VirtualGeometryCullOutputUpdate,
    ) {
        self.virtual_geometry_cull_mut().store(update);
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_render_path_outputs(
        &mut self,
        update: VirtualGeometryRenderPathOutputUpdate,
    ) {
        self.virtual_geometry_render_path_mut().store(update);
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_indirect_outputs(
        &mut self,
        update: VirtualGeometryIndirectOutputUpdate,
    ) {
        self.virtual_geometry_indirect_mut().store(update);
    }
}
