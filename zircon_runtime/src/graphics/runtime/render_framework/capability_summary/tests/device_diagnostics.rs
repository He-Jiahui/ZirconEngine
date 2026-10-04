use crate::rhi::{RenderAdapterInfo, RenderBackendCaps, RenderDeviceLimits};

use super::render_device_diagnostics;

#[test]
fn render_device_diagnostics_projects_actual_rhi_device_facts() {
    let caps = RenderBackendCaps::new("wgpu(dx12)")
        .with_adapter(RenderAdapterInfo {
            name: "Zircon Test Adapter".to_owned(),
            device_type: "discrete_gpu".to_owned(),
        })
        .with_device_limits(RenderDeviceLimits {
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 256,
            max_sampled_textures_per_shader_stage: 16,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroups_per_dimension: 65_535,
            max_binding_array_elements_per_shader_stage: 500_000,
            max_binding_array_sampler_elements_per_shader_stage: 1_000,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 134_217_728,
        });

    let diagnostics = render_device_diagnostics(&caps).expect("device diagnostics");

    assert_eq!(diagnostics.adapter_name, "Zircon Test Adapter");
    assert_eq!(diagnostics.adapter_device_type, "discrete_gpu");
    assert_eq!(diagnostics.limits.max_bind_groups, 5);
    assert_eq!(diagnostics.limits.max_texture_dimension_2d, 16_384);
    assert_eq!(diagnostics.limits.max_texture_array_layers, 256);
    assert_eq!(diagnostics.limits.max_sampled_textures_per_shader_stage, 16);
    assert_eq!(
        diagnostics
            .limits
            .max_binding_array_elements_per_shader_stage,
        500_000
    );
    assert_eq!(
        diagnostics
            .limits
            .max_binding_array_sampler_elements_per_shader_stage,
        1_000
    );
    assert_eq!(diagnostics.limits.max_storage_buffers_per_shader_stage, 8);
    assert_eq!(
        diagnostics.limits.max_storage_buffer_binding_size,
        134_217_728
    );
}

#[test]
fn render_device_diagnostics_refuses_incomplete_rhi_device_facts() {
    assert!(render_device_diagnostics(&RenderBackendCaps::new("test")).is_none());
}
