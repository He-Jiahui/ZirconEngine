use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::core::framework::render::RenderPluginRendererOutputs;

impl SceneRendererAdvancedPluginOutputs {
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
            ..RenderPluginRendererOutputs::default()
        };

        outputs.store_plugin_renderer_outputs(plugin_outputs.clone());

        assert_eq!(outputs.plugin_renderer_outputs(), &plugin_outputs);
        assert!(outputs.has_virtual_geometry_gpu_readback());
    }
}
