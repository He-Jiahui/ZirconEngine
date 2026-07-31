use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::RuntimePrepareExternalBufferBinding;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    pub(super) outputs: RenderPluginRendererOutputs,
    external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn new() -> Self {
        Self {
            outputs: RenderPluginRendererOutputs::default(),
            external_buffer_bindings: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs(
        outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            outputs,
            external_buffer_bindings: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs_and_external_buffer_bindings(
        outputs: RenderPluginRendererOutputs,
        external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Self {
        Self {
            outputs,
            external_buffer_bindings,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_outputs(
        self,
    ) -> RenderPluginRendererOutputs {
        self.outputs
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn external_buffer_bindings(
        &self,
    ) -> &[RuntimePrepareExternalBufferBinding] {
        &self.external_buffer_bindings
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn is_empty(&self) -> bool {
        self.outputs.is_empty() && self.external_buffer_bindings.is_empty()
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn outputs_for_test(
        &self,
    ) -> &RenderPluginRendererOutputs {
        &self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::SceneRendererAdvancedPluginReadbacks;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
        RenderVirtualGeometryReadbackOutputs,
    };
    use crate::graphics::RuntimePrepareExternalBufferBinding;
    use crate::graphics::backend::RenderBackend;

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
            ..RenderPluginRendererOutputs::default()
        };

        let readbacks = SceneRendererAdvancedPluginReadbacks::from_outputs(outputs.clone());

        assert_eq!(readbacks.outputs, outputs);
        assert!(!readbacks.is_empty());
        assert!(SceneRendererAdvancedPluginReadbacks::new().is_empty());
    }

    #[test]
    fn advanced_plugin_readbacks_hold_runtime_prepare_external_buffer_bindings() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-advanced-plugin-readbacks-external-buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let readbacks =
            SceneRendererAdvancedPluginReadbacks::from_outputs_and_external_buffer_bindings(
                RenderPluginRendererOutputs::default(),
                vec![RuntimePrepareExternalBufferBinding::new(
                    "particles.gpu.counters",
                    "particles.gpu.counters:test-runtime-prepare",
                    &buffer,
                )],
            );

        assert!(!readbacks.is_empty());
        assert_eq!(readbacks.external_buffer_bindings().len(), 1);
        assert_eq!(
            readbacks.external_buffer_bindings()[0].logical_name(),
            "particles.gpu.counters"
        );
        assert_eq!(
            readbacks.external_buffer_bindings()[0].backing_name(),
            "particles.gpu.counters:test-runtime-prepare"
        );
    }
}
