use super::config::RenderBackendConfig;
use crate::rhi::RenderBackendCaps;
use std::sync::{Arc, Mutex};
use zr_rhi::{DeviceAdmissionError, DeviceFaultGate, RenderDevice, RenderDeviceProfile};
use zr_rhi_wgpu::WgpuRenderDevice;

pub(crate) struct RenderBackend {
    // Declared before the device owner so native system resources drop first.
    pub(super) system_textures:
        super::system_texture_generation_owner::SystemTextureGenerationOwner,
    pub(crate) render_device: Arc<WgpuRenderDevice>,
    pub(super) diagnostic_delivery_router:
        Mutex<super::product_diagnostic_delivery_router::ProductDiagnosticDeliveryRouter>,
    pub(crate) device: wgpu::Device,
    #[cfg(test)]
    pub(crate) queue: wgpu::Queue,
    pub(crate) backend_name: String,
    pub(crate) config: RenderBackendConfig,
}

impl RenderBackend {
    pub(crate) fn acquire_system_texture_lease(
        &self,
    ) -> Result<
        (
            super::system_texture_generation_owner::SystemTextureGenerationLease,
            super::system_texture_generation_owner::SystemTextureGenerationStartupReport,
        ),
        crate::graphics::types::GraphicsError,
    > {
        self.system_textures
            .acquire(self.render_device.as_ref(), &self.device)
    }

    /// Clones the negotiated WGPU state for a native UI surface on this backend's device.
    ///
    /// The clone keeps typed WGPU ownership rather than exposing native pointers. Callers can
    /// therefore compose a renderer product and retained UI image through the same queue.
    pub(crate) fn ui_surface_context(&self) -> zr_rhi_wgpu::WgpuUiSurfaceContext {
        self.render_device.ui_surface_context()
    }

    /// Immutable cold-path identity and feature receipt for this device lifetime.
    pub(crate) fn device_profile(&self) -> &RenderDeviceProfile {
        self.render_device.profile()
    }

    /// One atomic fault-gate admission check for future resource and submission owners.
    pub(crate) fn ensure_device_admission(&self) -> Result<(), DeviceAdmissionError> {
        self.render_device.ensure_device_admission()
    }

    /// Shares the backend-owned fault state with a submission owner without
    /// creating a second device-health authority.
    pub(crate) fn device_fault_gate(&self) -> Arc<DeviceFaultGate> {
        self.render_device.device_fault_gate()
    }

    pub(crate) fn caps(&self) -> RenderBackendCaps {
        self.render_device.caps().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::RenderBackend;
    use zr_rhi::DeviceGeneration;

    #[test]
    fn backend_caps_delegate_to_the_generation_owner() {
        let source = include_str!("render_backend.rs");
        let caps_source = source
            .split("fn caps")
            .nth(1)
            .and_then(|source| source.split("#[cfg(test)]").next())
            .expect("render backend should expose its neutral capability view");

        assert!(caps_source.contains("self.render_device.caps().clone()"));
        assert!(!caps_source.contains("self.adapter"));
        assert!(!caps_source.contains("self.device"));
    }

    #[test]
    fn offscreen_backend_keeps_the_mvp_device_profile_with_its_live_wgpu_owner() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };

        let profile = backend.device_profile();

        assert_eq!(profile.generation(), DeviceGeneration::initial());
        assert!(!profile.adapter().name.is_empty());
        assert!(
            profile.requested_features().is_empty(),
            "the MVP base frame must not enable optional WGPU device features"
        );
        assert!(backend.ensure_device_admission().is_ok());
    }

    #[test]
    fn outer_backend_does_not_retain_a_second_native_adapter_owner() {
        let source = include_str!("render_backend.rs");
        let owner = source
            .split("pub(crate) struct RenderBackend")
            .nth(1)
            .and_then(|source| source.split("impl RenderBackend").next())
            .expect("render backend owner must remain inspectable");

        assert!(!owner.contains("wgpu::Adapter"));
        assert!(!owner.contains("adapter:"));
    }

    #[test]
    fn outer_backend_retains_raw_queue_only_for_unit_test_compatibility() {
        let source = include_str!("render_backend.rs");
        let owner = source
            .split("pub(crate) struct RenderBackend")
            .nth(1)
            .and_then(|source| source.split("impl RenderBackend").next())
            .expect("render backend owner must remain inspectable");
        let constructor = include_str!("render_backend_new_offscreen.rs");

        assert!(owner.contains("#[cfg(test)]\n    pub(crate) queue: wgpu::Queue"));
        assert!(
            constructor
                .contains("#[cfg(test)]\n        let test_queue = requested_device.queue.clone();")
        );
        assert!(constructor.contains("requested_device.queue,"));
        assert!(!constructor.contains("requested_device.queue.clone(),"));
    }

    #[test]
    fn ui_surface_context_reuses_the_negotiated_backend_owners() {
        let source = include_str!("render_backend.rs");
        let context_source = source
            .split("fn ui_surface_context")
            .nth(1)
            .and_then(|source| source.split("fn device_profile").next())
            .expect("render backend should expose a UI context from its negotiated device");

        assert!(context_source.contains("self.render_device.ui_surface_context()"));
        assert!(!context_source.contains("request_device"));
    }

    #[test]
    fn system_textures_drop_before_their_wgpu_device_generation_owner() {
        let source = include_str!("render_backend.rs");
        let owner = source
            .split("pub(crate) struct RenderBackend")
            .nth(1)
            .and_then(|source| source.split("impl RenderBackend").next())
            .expect("render backend owner must remain inspectable");
        let system_textures = owner
            .find("system_textures:")
            .expect("system texture generation owner");
        let render_device = owner.find("render_device:").expect("WGPU device owner");

        assert!(system_textures < render_device);
    }
}
