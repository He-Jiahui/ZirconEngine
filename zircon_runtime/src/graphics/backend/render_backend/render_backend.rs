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
#[path = "tests/render_backend.rs"]
mod tests;
