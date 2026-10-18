use crate::graphics::types::GraphicsError;
use zr_rhi::{AdapterSelectionPolicy, RenderAdapterCatalog, RenderAdapterFacts};
use zr_rhi_wgpu::wgpu_adapter_facts;

/// Enumerates adapters once on the startup cold path and applies the backend-neutral policy.
pub(super) fn select_offscreen_adapter(
    instance: &wgpu::Instance,
    config: super::config::RenderBackendConfig,
    policy: &AdapterSelectionPolicy,
) -> Result<(wgpu::Adapter, RenderAdapterFacts), GraphicsError> {
    let candidates = pollster::block_on(instance.enumerate_adapters(config.backends))
        .into_iter()
        .map(|adapter| {
            let facts = wgpu_adapter_facts(&adapter.get_info(), adapter.features());
            (adapter, facts)
        })
        .collect::<Vec<_>>();
    let mut selected = None;
    for backend in preferred_backend_order(config.prefer_dx12) {
        let catalog = RenderAdapterCatalog::new(candidates.iter()
            .filter(|(adapter, _)| backend.is_none_or(|api| adapter.get_info().backend == api))
            .map(|(_, facts)| facts.clone()).collect());
        if let Ok(selection) = catalog.select(policy) {
            selected = Some(selection.selected().clone());
            break;
        }
    }
    let selected = selected.ok_or(GraphicsError::NoAdapter)?;

    let chosen = candidates
        .into_iter()
        .find(|(_, facts)| facts == &selected)
        .ok_or(GraphicsError::NoAdapter)?;
    let info = chosen.0.get_info();
    eprintln!("gpu_backend={} adapter={:?} device_type={:?}",
        info.backend.to_str(), info.name, info.device_type);
    Ok(chosen)
}

fn preferred_backend_order(prefer_dx12: bool) -> Vec<Option<wgpu::Backend>> {
    if prefer_dx12 {
        vec![Some(wgpu::Backend::Dx12), Some(wgpu::Backend::Vulkan), None]
    } else {
        vec![None]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_windows_order_prefers_dx12_then_vulkan_and_keeps_fallback() {
        assert_eq!(preferred_backend_order(true),
            vec![Some(wgpu::Backend::Dx12), Some(wgpu::Backend::Vulkan), None]);
    }

    #[test]
    fn explicit_backend_does_not_apply_platform_preference() {
        let config = super::super::config::RenderBackendConfig::from_env_values(
            Some("vulkan"), None, None);
        assert!(!config.prefer_dx12);
        assert_eq!(config.backends, wgpu::Backends::VULKAN);
        assert_eq!(preferred_backend_order(config.prefer_dx12), vec![None]);
    }
}
