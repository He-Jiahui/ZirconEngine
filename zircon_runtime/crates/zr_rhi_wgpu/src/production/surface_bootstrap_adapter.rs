use zr_rhi::{RenderAdapterFacts, RenderDeviceProfile, RhiError, SurfaceSessionCreateOutcome};

use super::{WgpuRenderDevice, WgpuRenderDeviceContext};

/// One-way handoff that binds a native surface to the adapter selected for it.
pub struct WgpuSurfaceAdapterBootstrap {
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    adapter_facts: RenderAdapterFacts,
    descriptor: zr_rhi::RenderSurfaceDescriptor,
    instance: wgpu::Instance,
}

impl WgpuSurfaceAdapterBootstrap {
    pub(crate) fn new(
        surface: wgpu::Surface<'static>,
        descriptor: zr_rhi::RenderSurfaceDescriptor,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        adapter_facts: RenderAdapterFacts,
    ) -> Self {
        Self {
            surface,
            adapter,
            adapter_facts,
            descriptor,
            instance,
        }
    }

    /// Borrows the selected adapter only to issue this generation's native device request.
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Returns the neutral facts that must seed the matching device profile.
    pub const fn adapter_facts(&self) -> &RenderAdapterFacts {
        &self.adapter_facts
    }

    /// Requests and transfers one device from the selected adapter with its native surface.
    pub fn request_render_device(
        self,
        native_descriptor: &wgpu::DeviceDescriptor<'_>,
        profile: RenderDeviceProfile,
    ) -> Result<(WgpuRenderDevice, SurfaceSessionCreateOutcome), RhiError> {
        let Self {
            surface,
            adapter,
            adapter_facts,
            descriptor,
            instance,
        } = self;
        if profile.adapter() != &adapter_facts {
            return Err(RhiError::NativeContextAdapterMismatch {
                profile_adapter: profile.adapter().clone(),
                context_adapter: adapter_facts,
            });
        }
        let (device, queue) = pollster::block_on(adapter.request_device(native_descriptor))
            .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))?;
        let context = WgpuRenderDeviceContext::new(instance, adapter, device, queue);
        let render_device = WgpuRenderDevice::new(context, profile)?;
        let outcome = render_device.adopt_surface_session(descriptor, surface)?;
        Ok((render_device, outcome))
    }
}
