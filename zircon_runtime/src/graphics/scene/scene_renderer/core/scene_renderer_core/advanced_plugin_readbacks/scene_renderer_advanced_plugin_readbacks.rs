use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::{
    RuntimePrepareExternalBufferBinding, RuntimePrepareGpuPassProfile,
    RuntimePrepareGpuReadbackRequest,
};
use zr_rhi_wgpu::GpuReadbackQueue;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    pub(super) outputs: RenderPluginRendererOutputs,
    external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
    gpu_readbacks: Vec<RuntimePrepareGpuReadbackRequest>,
    gpu_pass_profiles: Vec<RuntimePrepareGpuPassProfile>,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn new() -> Self {
        Self {
            outputs: RenderPluginRendererOutputs::default(),
            external_buffer_bindings: Vec::new(),
            gpu_readbacks: Vec::new(),
            gpu_pass_profiles: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs(
        outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            outputs,
            external_buffer_bindings: Vec::new(),
            gpu_readbacks: Vec::new(),
            gpu_pass_profiles: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs_and_external_buffer_bindings(
        outputs: RenderPluginRendererOutputs,
        external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Self {
        Self {
            outputs,
            external_buffer_bindings,
            gpu_readbacks: Vec::new(),
            gpu_pass_profiles: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs_external_and_gpu_readbacks(
        outputs: RenderPluginRendererOutputs,
        external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
        gpu_readbacks: Vec<RuntimePrepareGpuReadbackRequest>,
    ) -> Self {
        Self {
            outputs,
            external_buffer_bindings,
            gpu_readbacks,
            gpu_pass_profiles: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn register_gpu_readbacks(
        &mut self,
        queue: &mut GpuReadbackQueue,
    ) -> Result<(), String> {
        let mut requests = std::mem::take(&mut self.gpu_readbacks).into_iter();
        while let Some(request) = requests.next() {
            if let Err(error) = request.register(queue) {
                for remaining in requests {
                    remaining.fail(error.to_string());
                }
                return Err(error.to_string());
            }
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn fail_gpu_readbacks(
        &mut self,
        error: impl Into<String>,
    ) {
        let error = error.into();
        for request in self.gpu_readbacks.drain(..) {
            request.fail(error.clone());
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_outputs(
        mut self,
    ) -> RenderPluginRendererOutputs {
        std::mem::take(&mut self.outputs)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn external_buffer_bindings(
        &self,
    ) -> &[RuntimePrepareExternalBufferBinding] {
        &self.external_buffer_bindings
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_gpu_pass_profiles(
        &mut self,
    ) -> Vec<RuntimePrepareGpuPassProfile> {
        std::mem::take(&mut self.gpu_pass_profiles)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn with_gpu_pass_profiles(
        mut self,
        gpu_pass_profiles: Vec<RuntimePrepareGpuPassProfile>,
    ) -> Self {
        self.gpu_pass_profiles = gpu_pass_profiles;
        self
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn is_empty(&self) -> bool {
        self.outputs.is_empty()
            && self.external_buffer_bindings.is_empty()
            && self.gpu_readbacks.is_empty()
            && self.gpu_pass_profiles.is_empty()
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn outputs_for_test(
        &self,
    ) -> &RenderPluginRendererOutputs {
        &self.outputs
    }
}

impl Drop for SceneRendererAdvancedPluginReadbacks {
    fn drop(&mut self) {
        self.fail_gpu_readbacks(
            "runtime prepare GPU readback request was dropped before registration",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::SceneRendererAdvancedPluginReadbacks;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
        RenderVirtualGeometryReadbackOutputs,
    };
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::RuntimePrepareExternalBufferBinding;

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
