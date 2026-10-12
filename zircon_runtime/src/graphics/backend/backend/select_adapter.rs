use crate::graphics::types::GraphicsError;
use zr_rhi::{AdapterSelectionPolicy, RenderAdapterCatalog, RenderAdapterFacts};
use zr_rhi_wgpu::wgpu_adapter_facts;

/// Enumerates adapters once on the startup cold path and applies the backend-neutral policy.
pub(super) fn select_offscreen_adapter(
    instance: &wgpu::Instance,
    backends: wgpu::Backends,
    policy: &AdapterSelectionPolicy,
) -> Result<(wgpu::Adapter, RenderAdapterFacts), GraphicsError> {
    let candidates = pollster::block_on(instance.enumerate_adapters(backends))
        .into_iter()
        .map(|adapter| {
            let facts = wgpu_adapter_facts(&adapter.get_info(), adapter.features());
            (adapter, facts)
        })
        .collect::<Vec<_>>();
    let catalog =
        RenderAdapterCatalog::new(candidates.iter().map(|(_, facts)| facts.clone()).collect());
    let selected = catalog.select(policy)?.selected().clone();

    candidates
        .into_iter()
        .find(|(_, facts)| facts == &selected)
        .ok_or(GraphicsError::NoAdapter)
}
