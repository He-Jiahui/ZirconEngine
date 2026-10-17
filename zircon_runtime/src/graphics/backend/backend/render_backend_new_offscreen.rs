use crate::graphics::types::GraphicsError;
use std::sync::Arc;
use zr_rhi::{AdapterSelectionPolicy, RenderDeviceRequestPolicy};
use zr_rhi_wgpu::{initial_wgpu_render_device_profile, WgpuRenderDevice, WgpuRenderDeviceContext};

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
            diagnostic_delivery_router: std::sync::Arc::new(std::sync::Mutex::new(
                super::product_diagnostic_delivery_router::ProductDiagnosticDeliveryRouter::new(
                    diagnostic_router_limit,
                ),
            )),
            device: requested_device.device,
            #[cfg(test)]
            queue: test_queue,
            backend_name,
            config,
            #[cfg(target_arch = "wasm32")]
            canvas: None,
        })
    }
    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn new_canvas(
        canvas: web_sys::HtmlCanvasElement,
        identity: u64,
        policy: &RenderDeviceRequestPolicy,
    ) -> Result<Self, GraphicsError> {
        use zr_rhi::{
            PresentMode, RenderNativeSurfaceTarget, RenderSurfaceDescriptor, SwapchainDesc,
            TextureFormat,
        };
        let mut config = RenderBackendConfig::from_environment();
        config.backends = wgpu::Backends::BROWSER_WEBGPU;
        let instance = wgpu::Instance::new(config.instance_descriptor());
        let registration = zr_rhi_wgpu::register_canvas_surface(
            RenderNativeSurfaceTarget::Canvas { identity },
            canvas.clone(),
        )?;
        let descriptor = RenderSurfaceDescriptor::new(
            "zircon-browser-canvas",
            RenderNativeSurfaceTarget::Canvas { identity },
            SwapchainDesc {
                width: canvas.width().max(1),
                height: canvas.height().max(1),
                present_mode: PresentMode::Fifo,
                format: TextureFormat::Rgba8Unorm,
            },
        );
        let bootstrap =
            zr_rhi_wgpu::WgpuSurfaceBootstrap::from_canvas(instance, descriptor, canvas)?
                .select_compatible_adapter_async(
                    wgpu::Backends::BROWSER_WEBGPU,
                    &AdapterSelectionPolicy::default(),
                )
                .await?;
        let request = zr_rhi_wgpu::wgpu_device_request(bootstrap.adapter().features(), policy)?;
        let limits = super::request_device::required_render_limits(
            &bootstrap.adapter().limits(),
            request.requested_features(),
        );
        let descriptor = wgpu::DeviceDescriptor {
            label: Some("zircon-webgpu-device"),
            required_features: request.requested_features(),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        };
        let (render_device, device, outcome) = bootstrap
            .request_graphics_device_async(&descriptor, request)
            .await?;
        let render_device = Arc::new(render_device);
        let diagnostic_router_limit = render_device
            .profile()
            .diagnostic_readback_budget()
            .max_pending_requests()
            .saturating_add(
                render_device
                    .profile()
                    .diagnostic_readback_budget()
                    .max_completed_receipts(),
            );
        let session = match outcome {
            zr_rhi::SurfaceSessionCreateOutcome::Renderable(receipt)
            | zr_rhi::SurfaceSessionCreateOutcome::NonRenderable(receipt) => receipt,
        };
        let system_textures =
            super::system_texture_generation_owner::SystemTextureGenerationOwner::new(
                render_device.as_ref(),
            );
        Ok(Self {
            system_textures,
            render_device,
            diagnostic_delivery_router: Arc::new(std::sync::Mutex::new(
                super::product_diagnostic_delivery_router::ProductDiagnosticDeliveryRouter::new(
                    diagnostic_router_limit,
                ),
            )),
            device,
            backend_name: "wgpu(webgpu)".into(),
            config,
            canvas: Some((registration, std::sync::Mutex::new(Some(session)))),
        })
    }
}
