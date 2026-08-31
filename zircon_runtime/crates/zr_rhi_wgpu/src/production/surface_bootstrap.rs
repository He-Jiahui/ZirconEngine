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
    pub fn select_compatible_adapter(
        self,
        backends: wgpu::Backends,
        policy: &AdapterSelectionPolicy,
    ) -> Result<WgpuSurfaceAdapterBootstrap, RhiError> {
        let Self {
            surface,
            descriptor,
            instance,
        } = self;
        let surface_ref = &surface;
        let candidates = pollster::block_on(instance.enumerate_adapters(backends))
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
