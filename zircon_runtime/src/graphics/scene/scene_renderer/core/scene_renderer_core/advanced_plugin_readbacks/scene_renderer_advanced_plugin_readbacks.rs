use crate::core::framework::render::RenderPluginRendererOutputs;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    pub(super) outputs: RenderPluginRendererOutputs,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn new() -> Self {
        Self {
            outputs: RenderPluginRendererOutputs::default(),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs(
        outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self { outputs }
    }
}

#[cfg(test)]
mod tests {
    use super::SceneRendererAdvancedPluginReadbacks;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
        RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn advanced_plugin_readbacks_hold_neutral_plugin_renderer_outputs() {
        let outputs = RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![1, 2, 3],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            hybrid_gi: RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![7, 9],
                ..RenderHybridGiReadbackOutputs::default()
            },
        };

        let readbacks = SceneRendererAdvancedPluginReadbacks::from_outputs(outputs.clone());

        assert_eq!(readbacks.outputs, outputs);
    }
}
