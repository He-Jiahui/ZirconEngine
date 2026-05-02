use crate::graphics::scene::scene_renderer::core::scene_renderer::SceneRendererAdvancedPluginOutputs;
use crate::graphics::types::GraphicsError;

use super::scene_renderer_advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn collect_into_outputs(
        self,
        _device: &wgpu::Device,
        outputs: &mut SceneRendererAdvancedPluginOutputs,
    ) -> Result<(), GraphicsError> {
        self.collect_neutral_outputs_into(outputs);
        Ok(())
    }

    fn collect_neutral_outputs_into(self, outputs: &mut SceneRendererAdvancedPluginOutputs) {
        outputs.store_plugin_renderer_outputs(self.outputs);
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
    fn advanced_plugin_readbacks_collect_neutral_plugin_renderer_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        let readbacks =
            SceneRendererAdvancedPluginReadbacks::from_outputs(RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![1, 2, 3],
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![7, 9],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            });

        readbacks.collect_neutral_outputs_into(&mut outputs);

        assert_eq!(
            outputs
                .plugin_renderer_outputs()
                .virtual_geometry
                .page_table_entries,
            vec![1, 2, 3]
        );
        assert_eq!(
            outputs
                .plugin_renderer_outputs()
                .hybrid_gi
                .completed_probe_ids,
            vec![7, 9]
        );
    }
}
