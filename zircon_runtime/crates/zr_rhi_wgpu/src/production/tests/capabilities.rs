use zr_rhi::{RenderOperation, RenderOperationSupport};

use super::super::device::capabilities::production_caps_from_wgpu;
use super::test_profile;

#[test]
fn production_capability_view_advertises_only_implemented_neutral_abi() {
    let native_features = wgpu::Features::TIMESTAMP_QUERY
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
        | wgpu::Features::SUBGROUP
        | wgpu::Features::PIPELINE_STATISTICS_QUERY
        | wgpu::Features::BUFFER_BINDING_ARRAY
        | wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let caps = production_caps_from_wgpu(
        native_features,
        wgpu::Limits::default(),
        true,
        &test_profile(),
    );

    assert!(caps.supports_surface);
    assert!(caps.supports_gpu_timestamp);
    assert!(!caps.supports_subgroup);
    assert!(caps.supports_pipeline_statistics_query);
    assert!(!caps.supports_buffer_binding_array);
    assert!(!caps.supports_texture_binding_array);
    assert!(!caps.supports_non_uniform_resource_indexing);
    assert!(!caps.supports_partially_bound_binding_array);
    assert!(caps.supports_indirect_draw);
    assert!(caps.supports_multi_draw_indirect);
    assert!(caps.supports_multi_draw_indirect_count);
    assert!(!caps.supports_indirect_first_instance);
    assert!(!caps.supports_buffer_readback);
    assert!(!caps.supports_graphics_debugger_capture);

    let partial_query_caps = production_caps_from_wgpu(
        wgpu::Features::TIMESTAMP_QUERY,
        wgpu::Limits::default(),
        true,
        &test_profile(),
    );
    assert!(!partial_query_caps.supports_gpu_timestamp);
    assert!(!partial_query_caps.supports_pipeline_statistics_query);

    for operation in RenderOperation::ALL {
        let expected = match operation {
            RenderOperation::DirectDraw
            | RenderOperation::IndexedDraw
            | RenderOperation::IndirectDraw
            | RenderOperation::MultiDrawIndirect
            | RenderOperation::MultiDrawIndirectCount
            | RenderOperation::ComputeDispatch
            | RenderOperation::ComputeDispatchIndirect
            | RenderOperation::BufferToBufferCopy
            | RenderOperation::BufferToTextureCopy
            | RenderOperation::TextureToBufferCopy
            | RenderOperation::TextureToTextureCopy
            | RenderOperation::DebugMarker
            | RenderOperation::DebugGroup => RenderOperationSupport::Native,
            _ => RenderOperationSupport::Unsupported,
        };
        assert_eq!(caps.operation_support(operation), expected, "{operation:?}");
    }
}

#[test]
fn production_indirect_capabilities_require_adapter_indirect_execution() {
    let caps = production_caps_from_wgpu(
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        false,
        &test_profile(),
    );

    assert!(!caps.supports_indirect_draw);
    assert!(!caps.supports_multi_draw_indirect);
    assert!(!caps.supports_multi_draw_indirect_count);
    assert_eq!(
        caps.operation_support(RenderOperation::IndirectDraw),
        RenderOperationSupport::Unsupported
    );
    assert_eq!(
        caps.operation_support(RenderOperation::MultiDrawIndirect),
        RenderOperationSupport::Unsupported
    );
    assert_eq!(
        caps.operation_support(RenderOperation::MultiDrawIndirectCount),
        RenderOperationSupport::Unsupported
    );
    assert_eq!(
        caps.operation_support(RenderOperation::ComputeDispatchIndirect),
        RenderOperationSupport::Native
    );
}
