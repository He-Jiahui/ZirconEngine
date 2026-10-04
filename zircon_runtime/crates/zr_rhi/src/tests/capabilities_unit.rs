use super::{RenderBackendCaps, RenderDeviceLimits, RenderOperation, RenderOperationSupport};

#[test]
fn legacy_render_device_limits_default_compute_fields_to_zero() {
    let legacy = r#"{
            "max_bind_groups": 5,
            "max_texture_dimension_2d": 16384,
            "max_texture_array_layers": 2048,
            "max_sampled_textures_per_shader_stage": 16,
            "max_binding_array_elements_per_shader_stage": 0,
            "max_binding_array_sampler_elements_per_shader_stage": 0,
            "min_uniform_buffer_offset_alignment": 256,
            "min_storage_buffer_offset_alignment": 256,
            "max_storage_buffers_per_shader_stage": 8,
            "max_storage_buffer_binding_size": 134217728
        }"#;
    let limits: RenderDeviceLimits =
        serde_json::from_str(legacy).expect("legacy device limits should deserialize");
    assert_eq!(limits.max_compute_workgroup_size_x, 0);
    assert_eq!(limits.max_compute_workgroup_size_y, 0);
    assert_eq!(limits.max_compute_workgroup_size_z, 0);
    assert_eq!(limits.max_compute_invocations_per_workgroup, 0);
    assert_eq!(limits.max_compute_workgroups_per_dimension, 0);
}

#[test]
fn render_backend_caps_deserialize_literal_pre_device_diagnostics_payload() {
    let legacy = r#"{
            "backend_name": "wgpu(vulkan)",
            "queue_classes": ["Graphics"],
            "supports_surface": true,
            "supports_offscreen": true,
            "supports_async_compute": false,
            "supports_async_copy": true,
            "supports_pipeline_cache": true,
            "supports_storage_buffers": true,
            "supports_indirect_draw": true,
            "supports_multi_draw_indirect": false,
            "supports_indirect_first_instance": false,
            "supports_buffer_readback": true,
            "supports_buffer_binding_array": false,
            "supports_texture_binding_array": false,
            "supports_non_uniform_resource_indexing": false,
            "supports_partially_bound_binding_array": false,
            "supports_neural_compute": false,
            "supports_sparse_texture": false,
            "supports_debug_markers": true,
            "supports_debug_groups": true,
            "supports_graphics_debugger_capture": false,
            "acceleration_structures": {
                "supported": false,
                "inline_ray_query": false,
                "ray_tracing_pipeline": false,
                "max_instance_count": null
            }
        }"#;

    let decoded: RenderBackendCaps =
        serde_json::from_str(legacy).expect("deserialize literal legacy backend caps");

    assert_eq!(decoded.backend_name, "wgpu(vulkan)");
    assert!(decoded.adapter.is_none());
    assert!(decoded.device_limits.is_none());
    assert!(decoded.supports_storage_buffers);
    assert!(decoded.supports_buffer_readback);
    assert!(!decoded.supports_subgroup);
    assert!(!decoded.supports_pipeline_statistics_query);
    assert_eq!(
        decoded.operation_support(RenderOperation::DirectDraw),
        RenderOperationSupport::Unsupported
    );
}
