use super::wgpu_backend_caps;
use zr_rhi::{RenderOperation, RenderOperationSupport, RenderQueueClass};

#[test]
fn timestamp_capability_requires_query_and_encoder_writes() {
    let full = wgpu_backend_caps(
        "full",
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        wgpu::Limits::default(),
        false,
        false,
        false,
    );
    let query_only = wgpu_backend_caps(
        "query-only",
        wgpu::Features::TIMESTAMP_QUERY,
        wgpu::Limits::default(),
        false,
        false,
        false,
    );

    assert!(full.supports_gpu_timestamp);
    assert!(!query_only.supports_gpu_timestamp);
}

#[test]
fn optional_wgpu_feature_gates_report_negotiated_device_capabilities() {
    let caps = wgpu_backend_caps(
        "optional-features",
        wgpu::Features::SUBGROUP | wgpu::Features::PIPELINE_STATISTICS_QUERY,
        wgpu::Limits::default(),
        false,
        false,
        false,
    );

    assert!(caps.supports_subgroup);
    assert!(caps.supports_pipeline_statistics_query);
}

#[test]
fn multi_draw_count_capability_is_distinct_from_the_fixed_count_fallback() {
    let without_count = wgpu_backend_caps(
        "fixed-count-only",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        false,
        false,
        true,
    );
    let with_count = wgpu_backend_caps(
        "count-enabled",
        wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        wgpu::Limits::default(),
        false,
        false,
        true,
    );

    assert!(without_count.supports_multi_draw_indirect);
    assert!(!without_count.supports_multi_draw_indirect_count);
    assert!(with_count.supports_multi_draw_indirect_count);
}

#[test]
fn indirect_capabilities_require_adapter_indirect_execution() {
    let caps = wgpu_backend_caps(
        "indirect-downlevel-missing",
        wgpu::Features::MULTI_DRAW_INDIRECT_COUNT | wgpu::Features::INDIRECT_FIRST_INSTANCE,
        wgpu::Limits::default(),
        false,
        false,
        false,
    );

    assert!(!caps.supports_indirect_draw);
    assert!(!caps.supports_multi_draw_indirect);
    assert!(!caps.supports_multi_draw_indirect_count);
    assert!(!caps.supports_indirect_first_instance);
}

#[test]
fn neutral_operation_matrix_rejects_wgpu_features_without_a_neutral_command() {
    let caps = wgpu_backend_caps(
        "operation-contract",
        wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        wgpu::Limits::default(),
        false,
        false,
        true,
    );

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Compute));
    assert!(caps.supports_queue(RenderQueueClass::Copy));
    assert!(!caps.supports_async_compute);
    assert!(!caps.supports_async_copy);
    assert!(caps.supports_multi_draw_indirect_count);
    assert!(caps.supports_graphics_debugger_capture);

    for operation in RenderOperation::ALL {
        assert_eq!(
            caps.operation_support(operation),
            RenderOperationSupport::Unsupported,
            "{operation:?} requires the production neutral device introduced in M2"
        );
    }
}
