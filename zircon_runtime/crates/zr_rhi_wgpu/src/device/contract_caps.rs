use zr_rhi::{RenderBackendCaps, RenderOperation, RenderOperationSupport, RenderQueueClass};

use crate::wgpu_device_limits;

pub(super) fn deterministic_contract_caps() -> RenderBackendCaps {
    let limits = wgpu::Limits::default();
    RenderBackendCaps::new("deterministic-rhi-contract-test")
        .with_device_limits(wgpu_device_limits(&limits))
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Compute)
        .with_queue(RenderQueueClass::Copy)
        .with_operation_support(RenderOperation::DirectDraw, RenderOperationSupport::Native)
        .with_operation_support(RenderOperation::IndexedDraw, RenderOperationSupport::Native)
        .with_operation_support(
            RenderOperation::IndirectDraw,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::MultiDrawIndirect,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::MultiDrawIndirectCount,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::ComputeDispatch,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::ComputeDispatchIndirect,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::BufferToBufferCopy,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::BufferToTextureCopy,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::TextureToBufferCopy,
            RenderOperationSupport::Native,
        )
        .with_operation_support(
            RenderOperation::TextureToTextureCopy,
            RenderOperationSupport::Native,
        )
        .with_operation_support(RenderOperation::DebugMarker, RenderOperationSupport::Native)
        .with_operation_support(RenderOperation::DebugGroup, RenderOperationSupport::Native)
        .with_surface_support(true)
        .with_offscreen_support(true)
        .with_storage_buffers(true)
        .with_fragment_writable_storage(true)
        .with_max_storage_buffers_per_shader_stage(limits.max_storage_buffers_per_shader_stage)
        .with_max_storage_buffer_binding_size(u64::from(limits.max_storage_buffer_binding_size))
        .with_indirect_draw(true)
        .with_multi_draw_indirect(true)
        .with_multi_draw_indirect_count(true)
        .with_buffer_readback(true)
        .with_debug_markers(true)
        .with_debug_groups(true)
}
