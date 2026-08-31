use zr_rhi::{
    RenderAdapterClass, RenderBackendKind, RenderDeviceFeature, RenderDeviceRequestPolicy,
};

use crate::{wgpu_adapter_facts, wgpu_device_limits, wgpu_features_for_device_request};

#[test]
fn wgpu_adapter_facts_preserve_stable_identity_and_admitted_features() {
    let facts = wgpu_adapter_facts(
        &wgpu::AdapterInfo {
            name: "Zircon test adapter".to_owned(),
            vendor: 0x10de,
            device: 0x2484,
            device_type: wgpu::DeviceType::DiscreteGpu,
            device_pci_bus_id: "0000:01:00.0".to_owned(),
            driver: "test-driver".to_owned(),
            driver_info: "1.0".to_owned(),
            backend: wgpu::Backend::Dx12,
            subgroup_min_size: 32,
            subgroup_max_size: 32,
            transient_saves_memory: false,
        },
        wgpu::Features::RG11B10UFLOAT_RENDERABLE | wgpu::Features::SUBGROUP,
    );

    assert_eq!(facts.backend, RenderBackendKind::Dx12);
    assert_eq!(facts.adapter_class, RenderAdapterClass::Discrete);
    assert_eq!(facts.vendor_id, 0x10de);
    assert_eq!(facts.device_id, 0x2484);
    assert_eq!(facts.driver_version, "test-driver 1.0");
    assert!(facts
        .supported_features
        .contains(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget));
    assert!(facts
        .supported_features
        .contains(RenderDeviceFeature::Subgroups));
    assert!(!facts
        .supported_features
        .contains(RenderDeviceFeature::GpuTimestamp));
}

#[test]
fn wgpu_device_request_maps_only_explicitly_admitted_neutral_features() {
    let adapter_features = wgpu::Features::RG11B10UFLOAT_RENDERABLE
        | wgpu::Features::TIMESTAMP_QUERY
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
    let baseline = wgpu_features_for_device_request(
        adapter_features,
        &RenderDeviceRequestPolicy::mvp_baseline(),
    )
    .expect("baseline negotiation must not fail");
    let profiled = wgpu_features_for_device_request(
        adapter_features,
        &RenderDeviceRequestPolicy::mvp_baseline()
            .with_optional_feature(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget)
            .with_optional_feature(RenderDeviceFeature::GpuTimestamp),
    )
    .expect("the adapter supports both selected optional profiles");

    assert!(baseline.is_empty());
    assert!(profiled.contains(wgpu::Features::RG11B10UFLOAT_RENDERABLE));
    assert!(profiled.contains(wgpu::Features::TIMESTAMP_QUERY));
    assert!(profiled.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS));
}

#[test]
fn wgpu_device_limits_keep_actual_negotiated_values_for_the_neutral_profile() {
    let limits = wgpu_device_limits(&wgpu::Limits {
        max_bind_groups: 5,
        max_texture_dimension_2d: 8_192,
        max_texture_array_layers: 512,
        max_sampled_textures_per_shader_stage: 16,
        max_binding_array_elements_per_shader_stage: 1_024,
        max_binding_array_sampler_elements_per_shader_stage: 256,
        max_storage_buffers_per_shader_stage: 8,
        max_storage_buffer_binding_size: 64 * 1024 * 1024,
        ..wgpu::Limits::default()
    });

    assert_eq!(limits.max_bind_groups, 5);
    assert_eq!(limits.max_texture_dimension_2d, 8_192);
    assert_eq!(limits.max_binding_array_elements_per_shader_stage, 1_024);
    assert_eq!(
        limits.max_binding_array_sampler_elements_per_shader_stage,
        256
    );
    assert_eq!(limits.max_storage_buffer_binding_size, 64 * 1024 * 1024);
}
