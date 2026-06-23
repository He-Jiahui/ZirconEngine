use super::*;

#[test]
fn command_list_keeps_queue_class_and_label() {
    let command_list = WgpuCommandList::new(RenderQueueClass::Graphics, "main");
    assert_eq!(command_list.queue_class(), RenderQueueClass::Graphics);
    assert_eq!(command_list.label(), Some("main"));
}

#[test]
fn command_list_records_buffer_copy_commands_and_submit_validates_resources() {
    let device = WgpuRenderDevice::new_headless();
    let source = device
        .create_buffer(&BufferDesc::new("copy-source", 32, BufferUsage::COPY_SRC))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "copy-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "copy-valid")
        .unwrap();
    command_list.push_debug_marker("upload source");
    command_list.copy_buffer_to_buffer(source, destination, 4, 8, 8);

    assert_eq!(command_list.recorded_command_count(), 2);
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    let mut unknown_destination = device
        .create_command_list(RenderQueueClass::Copy, "copy-unknown-destination")
        .unwrap();
    unknown_destination.copy_buffer_to_buffer(source, BufferHandle::new(9_999), 0, 0, 4);

    assert_eq!(
        device.submit(unknown_destination).unwrap_err(),
        RhiError::UnknownBuffer(9_999)
    );

    let mut out_of_range = device
        .create_command_list(RenderQueueClass::Copy, "copy-out-of-range")
        .unwrap();
    out_of_range.copy_buffer_to_buffer(source, destination, 0, 12, 8);

    assert_eq!(
        device.submit(out_of_range).unwrap_err(),
        RhiError::BufferCopyOutOfRange {
            source_buffer: source.raw(),
            destination_buffer: destination.raw(),
            source_offset: 0,
            destination_offset: 12,
            size: 8,
        }
    );
}

#[test]
fn command_list_records_compute_dispatch_and_submit_validates_pipeline() {
    let device = WgpuRenderDevice::new_headless();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "compute-fill",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "forward-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "forward-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let compute_pipeline = create_compute_pipeline(&device, "compute-fill", shader);
    let raster_pipeline =
        create_raster_pipeline(&device, "forward-opaque", vertex_shader, fragment_shader);

    let mut compute = device
        .create_command_list(RenderQueueClass::Compute, "compute-dispatch")
        .unwrap();
    compute.set_pipeline(compute_pipeline);
    compute.dispatch_compute(4, 2, 1);

    assert_eq!(
        compute.recorded_commands(),
        &[
            crate::rhi::CommandListCommand::SetPipeline {
                pipeline: compute_pipeline,
            },
            crate::rhi::CommandListCommand::DispatchCompute { x: 4, y: 2, z: 1 },
        ]
    );
    assert!(device
        .is_fence_complete(device.submit(compute).unwrap())
        .unwrap());

    let mut wrong_pipeline = device
        .create_command_list(RenderQueueClass::Compute, "compute-with-raster-pipeline")
        .unwrap();
    wrong_pipeline.set_pipeline(raster_pipeline);
    wrong_pipeline.dispatch_compute(1, 1, 1);
    assert_eq!(
        device.submit(wrong_pipeline).unwrap_err(),
        RhiError::InvalidPipelineUsage {
            pipeline: raster_pipeline.raw(),
            required: PipelineKind::Compute,
            actual: PipelineKind::Raster,
        }
    );

    let mut missing_pipeline = device
        .create_command_list(RenderQueueClass::Compute, "compute-without-pipeline")
        .unwrap();
    missing_pipeline.dispatch_compute(1, 1, 1);
    assert_eq!(
        device.submit(missing_pipeline).unwrap_err(),
        RhiError::InvalidComputeDispatch {
            reason: "compute dispatch requires a bound compute pipeline".to_string(),
        }
    );

    let mut copy_queue_dispatch = device
        .create_command_list(RenderQueueClass::Copy, "copy-queue-compute-dispatch")
        .unwrap();
    copy_queue_dispatch.set_pipeline(compute_pipeline);
    copy_queue_dispatch.dispatch_compute(1, 1, 1);
    assert_eq!(
        device.submit(copy_queue_dispatch).unwrap_err(),
        RhiError::InvalidCommandQueue {
            queue: RenderQueueClass::Copy,
            command: "dispatch_compute".to_string(),
        }
    );

    device.destroy_pipeline(compute_pipeline).unwrap();
    device.destroy_pipeline(raster_pipeline).unwrap();
    device.destroy_shader_module(shader).unwrap();
}
