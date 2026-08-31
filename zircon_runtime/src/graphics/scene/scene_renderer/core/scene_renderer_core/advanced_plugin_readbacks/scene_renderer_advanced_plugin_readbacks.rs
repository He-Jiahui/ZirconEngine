use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::RenderBackend;
use crate::graphics::{
    RuntimePrepareExternalBufferBinding, RuntimePrepareExternalBufferBindingPacket,
    RuntimePrepareFramePacket, RuntimePrepareGpuPassProfile, RuntimePrepareGpuReadbackRequest,
};
use crate::rhi::RenderDeviceProfile;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    pub(super) outputs: RenderPluginRendererOutputs,
    external_buffer_binding_packet: Option<RuntimePrepareExternalBufferBindingPacket>,
    gpu_readbacks: Vec<RuntimePrepareGpuReadbackRequest>,
    gpu_pass_profiles: Vec<RuntimePrepareGpuPassProfile>,
    runtime_prepare_frame_packet: RuntimePrepareFramePacket,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn new() -> Self {
        Self {
            outputs: RenderPluginRendererOutputs::default(),
            external_buffer_binding_packet: None,
            gpu_readbacks: Vec::new(),
            gpu_pass_profiles: Vec::new(),
            runtime_prepare_frame_packet: RuntimePrepareFramePacket::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs(
        outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            outputs,
            external_buffer_binding_packet: None,
            gpu_readbacks: Vec::new(),
            gpu_pass_profiles: Vec::new(),
            runtime_prepare_frame_packet: RuntimePrepareFramePacket::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs_and_external_buffer_bindings(
        device_profile: &RenderDeviceProfile,
        outputs: RenderPluginRendererOutputs,
        external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Self {
        Self {
            outputs,
            external_buffer_binding_packet: RuntimePrepareExternalBufferBindingPacket::new(
                device_profile,
                external_buffer_bindings,
            ),
            gpu_readbacks: Vec::new(),
            gpu_pass_profiles: Vec::new(),
            runtime_prepare_frame_packet: RuntimePrepareFramePacket::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs_external_and_gpu_readbacks(
        device_profile: &RenderDeviceProfile,
        outputs: RenderPluginRendererOutputs,
        external_buffer_bindings: Vec<RuntimePrepareExternalBufferBinding>,
        gpu_readbacks: Vec<RuntimePrepareGpuReadbackRequest>,
    ) -> Self {
        Self {
            outputs,
            external_buffer_binding_packet: RuntimePrepareExternalBufferBindingPacket::new(
                device_profile,
                external_buffer_bindings,
            ),
            gpu_readbacks,
            gpu_pass_profiles: Vec::new(),
            runtime_prepare_frame_packet: RuntimePrepareFramePacket::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn has_gpu_readbacks(&self) -> bool {
        !self.gpu_readbacks.is_empty()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn register_product_gpu_readbacks(
        &mut self,
        backend: &RenderBackend,
    ) -> Result<usize, crate::graphics::types::GraphicsError> {
        let mut admitted_count = 0_usize;
        let mut requests = std::mem::take(&mut self.gpu_readbacks).into_iter();
        while let Some(request) = requests.next() {
            match request.register(backend) {
                Ok(true) => admitted_count = admitted_count.saturating_add(1),
                Ok(false) => {}
                Err(error) => {
                    for remaining in requests {
                        remaining.fail(error.to_string());
                    }
                    return Err(error);
                }
            }
        }
        Ok(admitted_count)
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

    pub(in crate::graphics::scene::scene_renderer::core) fn external_buffer_binding_packet(
        &self,
    ) -> Option<&RuntimePrepareExternalBufferBindingPacket> {
        self.external_buffer_binding_packet.as_ref()
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

    pub(in crate::graphics::scene::scene_renderer::core) fn with_runtime_prepare_frame_packet(
        mut self,
        runtime_prepare_frame_packet: RuntimePrepareFramePacket,
    ) -> Self {
        self.runtime_prepare_frame_packet = runtime_prepare_frame_packet;
        self
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_runtime_prepare_buffer_uploads(
        &mut self,
    ) -> WgpuBufferUploadBatch {
        self.runtime_prepare_frame_packet.take_buffer_uploads()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn commit_runtime_prepare_frame_transactions(
        &mut self,
    ) {
        self.runtime_prepare_frame_packet
            .commit_frame_transactions();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn is_empty(&self) -> bool {
        self.outputs.is_empty()
            && self.external_buffer_binding_packet.is_none()
            && self.gpu_readbacks.is_empty()
            && self.gpu_pass_profiles.is_empty()
            && self.runtime_prepare_frame_packet.is_empty()
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
                backend.device_profile(),
                RenderPluginRendererOutputs::default(),
                vec![RuntimePrepareExternalBufferBinding::new(
                    "particles.gpu.counters",
                    "particles.gpu.counters:test-runtime-prepare",
                    &buffer,
                )],
            );

        assert!(!readbacks.is_empty());
        let bindings = readbacks
            .external_buffer_binding_packet()
            .expect("registered bindings must retain their device-qualified packet")
            .bindings();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].logical_name(), "particles.gpu.counters");
        assert_eq!(
            bindings[0].backing_name(),
            "particles.gpu.counters:test-runtime-prepare"
        );
    }

    #[test]
    fn advanced_plugin_readbacks_register_through_the_product_diagnostic_router() {
        let source = include_str!("scene_renderer_advanced_plugin_readbacks.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert!(source.contains("register_product_gpu_readbacks"));
        assert!(source.contains("request.register(backend)"));
        assert!(!source.contains("GpuReadbackQueue"));
        assert!(!source.contains("request_readback_external"));
    }
}
