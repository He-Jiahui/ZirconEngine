use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::scene::scene_renderer::core::scene_renderer::{
    VirtualGeometryCullOutputUpdate, VirtualGeometryIndirectOutputUpdate,
    VirtualGeometryLastOutputUpdate, VirtualGeometryRenderPathOutputUpdate,
};

impl SceneRendererAdvancedPluginOutputs {
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

    pub(in crate::graphics::scene::scene_renderer::core) fn store_plugin_renderer_outputs(
        &mut self,
        outputs: RenderPluginRendererOutputs,
    ) {
        *self.plugin_renderer_outputs_mut() = outputs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
        RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn stores_neutral_plugin_renderer_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        let plugin_outputs = RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![1, 2, 3],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            hybrid_gi: RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![7, 9],
                ..RenderHybridGiReadbackOutputs::default()
            },
        };

        outputs.store_plugin_renderer_outputs(plugin_outputs.clone());

        assert_eq!(outputs.plugin_renderer_outputs(), &plugin_outputs);
        assert!(outputs.has_virtual_geometry_gpu_readback());
    }
}
