use crate::{
    AccelerationStructureCaps, BufferHandle, CommandListCommand, DeviceGeneration, DeviceId,
    RenderBackendCaps, RenderDebugInstrumentationStatus, RenderOperation, RenderOperationSupport,
    RenderQueueClass, RenderResourceHandleAllocator, RhiError, TextureCopyRegion,
    UnsupportedRenderOperation,
};

#[test]
fn backend_caps_report_queue_classes_and_rt_support_independently() {
    let caps = RenderBackendCaps::new("test-backend")
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Compute)
        .with_surface_support(true)
        .with_pipeline_cache(true)
        .with_storage_buffers(true)
        .with_indirect_draw(true)
        .with_multi_draw_indirect(true)
        .with_indirect_first_instance(true)
        .with_buffer_readback(true)
        .with_neural_compute(true)
        .with_sparse_texture(true)
        .with_debug_markers(true)
        .with_debug_groups(true)
        .with_graphics_debugger_capture(true)
        .with_acceleration_structures(AccelerationStructureCaps::disabled());

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Compute));
    assert!(!caps.supports_queue(RenderQueueClass::Copy));
    assert!(caps.supports_surface);
    assert!(caps.supports_pipeline_cache);
    assert!(caps.supports_storage_buffers);
    assert!(caps.supports_indirect_draw);
    assert!(caps.supports_multi_draw_indirect);
    assert!(caps.supports_indirect_first_instance);
    assert!(caps.supports_buffer_readback);
    assert!(caps.supports_neural_compute);
    assert!(caps.supports_sparse_texture);
    assert!(caps.supports_debug_markers);
    assert!(caps.supports_debug_groups);
    assert!(caps.supports_graphics_debugger_capture);
    assert!(!caps.acceleration_structures.supported);
}

#[test]
fn backend_debug_instrumentation_status_is_derived_from_caps() {
    let caps = RenderBackendCaps::new("instrumented-test")
        .with_debug_markers(true)
        .with_debug_groups(true)
        .with_graphics_debugger_capture(true);

    let status = RenderDebugInstrumentationStatus::from_caps(&caps);

    assert_eq!(status.backend_name, "instrumented-test");
    assert!(status.debug_markers_supported);
    assert!(status.debug_groups_supported);
    assert!(status.graphics_debugger_capture_supported);
    assert!(!status.active_graphics_debugger_capture);
    assert_eq!(status.last_error, None);

    assert_eq!(
        RenderDebugInstrumentationStatus::unavailable("offline"),
        RenderDebugInstrumentationStatus {
            backend_name: "offline".to_string(),
            debug_markers_supported: false,
            debug_groups_supported: false,
            graphics_debugger_capture_supported: false,
            active_graphics_debugger_capture: false,
            last_error: None,
        }
    );
}

#[test]
fn operation_matrix_distinguishes_executable_and_unsupported_contracts() {
    let caps = RenderBackendCaps::new("operation-contract")
        .with_operation_support(RenderOperation::DirectDraw, RenderOperationSupport::Native)
        .with_operation_support(
            RenderOperation::MultiDrawIndirect,
            RenderOperationSupport::Emulated,
        );

    assert_eq!(
        caps.operation_support(RenderOperation::DirectDraw),
        RenderOperationSupport::Native
    );
    assert_eq!(
        caps.operation_support(RenderOperation::MultiDrawIndirect),
        RenderOperationSupport::Emulated
    );
    assert_eq!(
        caps.operation_support(RenderOperation::GraphicsDebuggerCapture),
        RenderOperationSupport::Unsupported
    );
    assert!(caps.supports_operation(RenderOperation::DirectDraw));
    assert!(caps.supports_operation(RenderOperation::MultiDrawIndirect));
    assert!(!caps.supports_operation(RenderOperation::GraphicsDebuggerCapture));
}

#[test]
fn operation_admission_returns_a_structured_rejection_for_unsupported_work() {
    let caps = RenderBackendCaps::new("operation-admission")
        .with_operation_support(RenderOperation::DirectDraw, RenderOperationSupport::Native)
        .with_operation_support(
            RenderOperation::MultiDrawIndirect,
            RenderOperationSupport::Emulated,
        );

    assert_eq!(
        caps.require_operation(RenderOperation::DirectDraw),
        Ok(RenderOperationSupport::Native)
    );
    assert_eq!(
        caps.require_operation(RenderOperation::MultiDrawIndirect),
        Ok(RenderOperationSupport::Emulated)
    );
    assert_eq!(
        caps.require_operation(RenderOperation::AsyncComputeQueue),
        Err(UnsupportedRenderOperation {
            operation: RenderOperation::AsyncComputeQueue,
            support: RenderOperationSupport::Unsupported,
        })
    );
}

#[test]
fn unsupported_operation_rejection_maps_to_the_rhi_error_taxonomy() {
    let rejection = UnsupportedRenderOperation {
        operation: RenderOperation::GraphicsDebuggerCapture,
        support: RenderOperationSupport::Unsupported,
    };

    assert_eq!(
        RhiError::from(rejection),
        RhiError::UnsupportedOperation {
            operation: RenderOperation::GraphicsDebuggerCapture,
            support: RenderOperationSupport::Unsupported,
        }
    );
}

#[test]
fn operation_inventory_is_contiguous_and_matches_the_matrix_extent() {
    assert_eq!(RenderOperation::ALL.len(), RenderOperation::COUNT);

    for (index, operation) in RenderOperation::ALL.into_iter().enumerate() {
        assert_eq!(operation as usize, index, "{operation:?}");
    }
}

#[test]
fn recorded_commands_map_to_their_required_neutral_operations() {
    let handles = RenderResourceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    let copy_region = TextureCopyRegion::new(1, 1);
    let source_buffer = handles.allocate_buffer().expect("allocate source buffer");
    let destination_buffer = handles
        .allocate_buffer()
        .expect("allocate destination buffer");
    let texture = handles.allocate_texture().expect("allocate texture");
    let mapped_commands = [
        (
            CommandListCommand::DebugMarker {
                label: "marker".to_string(),
            },
            RenderOperation::DebugMarker,
        ),
        (
            CommandListCommand::PushDebugGroup {
                label: "group".to_string(),
            },
            RenderOperation::DebugGroup,
        ),
        (
            CommandListCommand::PopDebugGroup,
            RenderOperation::DebugGroup,
        ),
        (
            CommandListCommand::CopyBufferToBuffer {
                source: source_buffer,
                destination: destination_buffer,
                source_offset: 0,
                destination_offset: 0,
                size: 1,
            },
            RenderOperation::BufferToBufferCopy,
        ),
        (
            CommandListCommand::CopyBufferToTexture {
                source: source_buffer,
                destination: texture,
                source_offset: 0,
                bytes_per_row: 4,
                region: copy_region,
            },
            RenderOperation::BufferToTextureCopy,
        ),
        (
            CommandListCommand::CopyTextureToBuffer {
                source: texture,
                destination: destination_buffer,
                destination_offset: 0,
                bytes_per_row: 4,
                region: copy_region,
            },
            RenderOperation::TextureToBufferCopy,
        ),
        (
            CommandListCommand::CopyTextureToTexture {
                source: texture,
                destination: handles
                    .allocate_texture()
                    .expect("allocate copy destination"),
                source_region: copy_region,
                destination_region: copy_region,
            },
            RenderOperation::TextureToTextureCopy,
        ),
        (
            CommandListCommand::Draw {
                vertex_start: 0,
                vertex_count: 1,
                instance_start: 0,
                instance_count: 1,
            },
            RenderOperation::DirectDraw,
        ),
        (
            CommandListCommand::DrawIndexed {
                index_start: 0,
                index_count: 1,
                base_vertex: 0,
                instance_start: 0,
                instance_count: 1,
            },
            RenderOperation::IndexedDraw,
        ),
        (
            CommandListCommand::DrawIndirect {
                arguments: source_buffer,
                offset: 0,
            },
            RenderOperation::IndirectDraw,
        ),
        (
            CommandListCommand::DrawIndexedIndirect {
                arguments: source_buffer,
                offset: 0,
            },
            RenderOperation::IndirectDraw,
        ),
        (
            CommandListCommand::MultiDrawIndirect {
                arguments: source_buffer,
                offset: 0,
                count: 2,
            },
            RenderOperation::MultiDrawIndirect,
        ),
        (
            CommandListCommand::MultiDrawIndexedIndirect {
                arguments: source_buffer,
                offset: 0,
                count: 2,
            },
            RenderOperation::MultiDrawIndirect,
        ),
        (
            CommandListCommand::MultiDrawIndexedIndirectCount {
                arguments: source_buffer,
                offset: 0,
                count_buffer: destination_buffer,
                count_offset: 0,
                max_count: 2,
            },
            RenderOperation::MultiDrawIndirectCount,
        ),
        (
            CommandListCommand::DispatchCompute { x: 1, y: 1, z: 1 },
            RenderOperation::ComputeDispatch,
        ),
        (
            CommandListCommand::DispatchComputeIndirect {
                arguments: source_buffer,
                offset: 0,
            },
            RenderOperation::ComputeDispatchIndirect,
        ),
    ];

    for (command, operation) in mapped_commands {
        assert_eq!(command.required_operation(), Some(operation));
    }
    assert_eq!(CommandListCommand::EndRenderPass.required_operation(), None);
}
