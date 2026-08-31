use crate::graphics::types::GraphicsError;
use std::sync::Arc;
use zr_rhi::{AdapterSelectionPolicy, RenderDeviceRequestPolicy};
use zr_rhi_wgpu::{WgpuRenderDevice, WgpuRenderDeviceContext, initial_wgpu_render_device_profile};

use super::config::RenderBackendConfig;
use super::render_backend::RenderBackend;
use super::request_device::request_device_with_policy;
use super::select_offscreen_adapter;

impl RenderBackend {
    pub(crate) fn new_offscreen() -> Result<Self, GraphicsError> {
        Self::new_offscreen_with_policy(&RenderDeviceRequestPolicy::mvp_baseline())
    }

    pub(crate) fn new_offscreen_with_policy(
        device_request_policy: &RenderDeviceRequestPolicy,
    ) -> Result<Self, GraphicsError> {
        let config = RenderBackendConfig::from_environment();
        let instance = wgpu::Instance::new(config.instance_descriptor());
        let (adapter, adapter_facts) = select_offscreen_adapter(
            &instance,
            config.backends,
            &AdapterSelectionPolicy::default(),
        )?;
        let requested_device = request_device_with_policy(&adapter, device_request_policy)?;
        let device_profile = initial_wgpu_render_device_profile(
            adapter_facts,
            &requested_device.device,
            &requested_device.profile_request,
        );
        let backend_name = format!("wgpu({})", adapter.get_info().backend.to_str());
        let diagnostic_router_limit = device_profile
            .diagnostic_readback_budget()
            .max_pending_requests()
            .saturating_add(
                device_profile
                    .diagnostic_readback_budget()
                    .max_completed_receipts(),
            );
        #[cfg(test)]
        let test_queue = requested_device.queue.clone();
        let render_device = Arc::new(WgpuRenderDevice::new(
            WgpuRenderDeviceContext::new(
                instance,
                adapter,
                requested_device.device.clone(),
                requested_device.queue,
            ),
            device_profile,
        )?);
        let system_textures =
            super::system_texture_generation_owner::SystemTextureGenerationOwner::new(
                render_device.as_ref(),
            );

        Ok(Self {
            system_textures,
            render_device,
            diagnostic_delivery_router: std::sync::Mutex::new(
                super::product_diagnostic_delivery_router::ProductDiagnosticDeliveryRouter::new(
                    diagnostic_router_limit,
                ),
            ),
            device: requested_device.device,
            #[cfg(test)]
            queue: test_queue,
            backend_name,
            config,
        })
    }
}
