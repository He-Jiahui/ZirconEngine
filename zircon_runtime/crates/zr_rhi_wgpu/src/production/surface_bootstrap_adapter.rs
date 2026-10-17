//! 已选适配器及主表面一次性交给设备，后续表面帧由同一代际管理。
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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn request_render_device(
        self,
        native_descriptor: &wgpu::DeviceDescriptor<'_>,
        profile: RenderDeviceProfile,
    ) -> Result<(WgpuRenderDevice, SurfaceSessionCreateOutcome), RhiError> {
        pollster::block_on(self.request_render_device_async(native_descriptor, profile))
    }

    pub async fn request_render_device_async(
        self,
        native_descriptor: &wgpu::DeviceDescriptor<'_>,
        profile: RenderDeviceProfile,
    ) -> Result<(WgpuRenderDevice, SurfaceSessionCreateOutcome), RhiError> {
        let (owner, _, outcome) = self
            .request_device_inner(native_descriptor, move |facts, _| {
                if profile.adapter() != facts {
                    return Err(RhiError::NativeContextAdapterMismatch {
                        profile_adapter: profile.adapter().clone(),
                        context_adapter: facts.clone(),
                    });
                }
                Ok(profile)
            })
            .await?;
        Ok((owner, outcome))
    }

    /// Builds the profile from the actual negotiated device and transfers the retained surface.
    /// The typed device handle is for pipeline construction; queue and submission authority stay
    /// with the returned render-device owner.
    pub async fn request_graphics_device_async(
        self,
        native_descriptor: &wgpu::DeviceDescriptor<'_>,
        request: crate::WgpuDeviceRequest,
    ) -> Result<(WgpuRenderDevice, wgpu::Device, SurfaceSessionCreateOutcome), RhiError> {
        if native_descriptor.required_features != request.requested_features() {
            return Err(RhiError::SurfaceUnavailable(
                "device descriptor and negotiated features differ".into(),
            ));
        }
        self.request_device_inner(native_descriptor, move |facts, device| {
            Ok(crate::initial_wgpu_render_device_profile(
                facts.clone(),
                device,
                &request,
            ))
        })
        .await
    }

    async fn request_device_inner(
        self,
        native_descriptor: &wgpu::DeviceDescriptor<'_>,
        profile: impl FnOnce(
            &RenderAdapterFacts,
            &wgpu::Device,
        ) -> Result<RenderDeviceProfile, RhiError>,
    ) -> Result<(WgpuRenderDevice, wgpu::Device, SurfaceSessionCreateOutcome), RhiError> {
        let Self {
            surface,
            adapter,
            adapter_facts,
            descriptor,
            instance,
        } = self;
        let (device, queue) = adapter
            .request_device(native_descriptor)
            .await
            .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))?;
        let profile = profile(&adapter_facts, &device)?;
        let context = WgpuRenderDeviceContext::new(instance, adapter, device.clone(), queue);
        let render_device = WgpuRenderDevice::new(context, profile)?;
        let outcome = render_device.adopt_surface_session(descriptor, surface)?;
        Ok((render_device, device, outcome))
    }
}
