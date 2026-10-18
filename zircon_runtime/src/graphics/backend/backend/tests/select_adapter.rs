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
