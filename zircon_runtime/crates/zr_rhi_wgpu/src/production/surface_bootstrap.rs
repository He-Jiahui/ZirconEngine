//! 主表面先参与适配器选择，再连同实例交给设备，保证表面与设备兼容。
use zr_rhi::{AdapterSelectionPolicy, RenderAdapterCatalog, RenderSurfaceDescriptor, RhiError};

use crate::wgpu_adapter_facts;

use super::surface::{create_native_surface, surface_descriptor_is_supported};
use super::WgpuSurfaceAdapterBootstrap;

/// Primary cold-start owner for the one native surface used to select and initialize a WGPU device.
///
/// The surface remains private until it transfers into [`WgpuRenderDevice`], so adapter
/// compatibility and the eventual frame leases always belong to the same native generation.
/// Existing device-created sessions are limited to secondary surfaces while the product
/// renderer completes its hard cutover to this path.
pub struct WgpuSurfaceBootstrap {
    surface: wgpu::Surface<'static>,
    descriptor: RenderSurfaceDescriptor,
    instance: wgpu::Instance,
}

impl WgpuSurfaceBootstrap {
    /// Retains the Canvas surface before requesting its compatible WebGPU adapter.
    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas(
        instance: wgpu::Instance,
        descriptor: RenderSurfaceDescriptor,
        canvas: web_sys::HtmlCanvasElement,
    ) -> Result<Self, RhiError> {
        if !matches!(descriptor.target, zr_rhi::RenderNativeSurfaceTarget::Canvas { identity } if identity != 0)
        {
            return Err(RhiError::SurfaceUnavailable(
                "a Canvas surface needs a nonzero browser identity".into(),
            ));
        }
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))?;
        Ok(Self {
            surface,
            descriptor,
            instance,
        })
    }

    /// Creates the native surface before adapter selection and retains the instance that owns it.
    pub fn new(
        instance: wgpu::Instance,
        descriptor: RenderSurfaceDescriptor,
    ) -> Result<Self, RhiError> {
        let surface = create_native_surface(&instance, descriptor.target)?;
        Ok(Self {
            surface,
            descriptor,
            instance,
        })
    }

    /// Selects a policy-approved adapter only from adapters that can present this surface.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_compatible_adapter(
        self,
        backends: wgpu::Backends,
        policy: &AdapterSelectionPolicy,
    ) -> Result<WgpuSurfaceAdapterBootstrap, RhiError> {
        pollster::block_on(self.select_compatible_adapter_async(backends, policy))
    }

    pub async fn select_compatible_adapter_async(
        self,
        backends: wgpu::Backends,
        policy: &AdapterSelectionPolicy,
    ) -> Result<WgpuSurfaceAdapterBootstrap, RhiError> {
        let Self {
            surface,
            mut descriptor,
            instance,
        } = self;
        let surface_ref = &surface;
        #[cfg(target_arch = "wasm32")]
        let adapters = {
            let _ = backends;
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(surface_ref),
                    ..Default::default()
                })
                .await
                .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))?;
            let capabilities = surface_ref.get_capabilities(&adapter);
            let selected = capabilities
                .formats
                .iter()
                .find_map(|format| match format {
                    wgpu::TextureFormat::Bgra8UnormSrgb => {
                        Some(zr_rhi::TextureFormat::Bgra8UnormSrgb)
                    }
                    wgpu::TextureFormat::Rgba8UnormSrgb => {
                        Some(zr_rhi::TextureFormat::Rgba8UnormSrgb)
                    }
                    wgpu::TextureFormat::Bgra8Unorm => Some(zr_rhi::TextureFormat::Bgra8Unorm),
                    wgpu::TextureFormat::Rgba8Unorm => Some(zr_rhi::TextureFormat::Rgba8Unorm),
                    _ => None,
                })
                .ok_or_else(|| {
                    RhiError::SurfaceUnavailable("Canvas has no SDR surface format".into())
                })?;
            descriptor.swapchain.format = selected;
            vec![adapter]
        };
        #[cfg(not(target_arch = "wasm32"))]
        let adapters = instance.enumerate_adapters(backends).await;
        let candidates = adapters
            .into_iter()
            .filter_map(|adapter| {
                let capabilities = surface_ref.get_capabilities(&adapter);
                surface_descriptor_is_supported(&capabilities, &descriptor).then(|| {
                    let facts = wgpu_adapter_facts(&adapter.get_info(), adapter.features());
                    (adapter, facts)
                })
            })
            .collect::<Vec<_>>();
        let catalog =
            RenderAdapterCatalog::new(candidates.iter().map(|(_, facts)| facts.clone()).collect());
        let selected = catalog.select(policy).map_err(|error| {
            RhiError::SurfaceUnavailable(format!(
                "no adapter compatible with the native surface satisfied selection policy: {error}"
            ))
        })?;
        let selected_facts = selected.selected();
        let (adapter, adapter_facts) = candidates
            .into_iter()
            .find(|(_, facts)| facts == selected_facts)
            .ok_or_else(|| {
                RhiError::SurfaceUnavailable(
                    "selected native-surface adapter disappeared during bootstrap".to_string(),
                )
            })?;
        Ok(WgpuSurfaceAdapterBootstrap::new(
            surface,
            descriptor,
            instance,
            adapter,
            adapter_facts,
        ))
    }
}
