use super::*;

#[test]
fn command_list_keeps_queue_class_and_label() {
    let command_list = DeterministicRhiContractCommandList::new(RenderQueueClass::Graphics, "main");
    assert_eq!(command_list.queue_class(), RenderQueueClass::Graphics);
    assert_eq!(command_list.label(), Some("main"));
}

#[test]
fn command_list_records_buffer_copy_commands_and_submit_validates_resources() {
    let device = DeterministicRhiContractDevice::new_headless();
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
    let ticket = device.submit(command_list).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );

    let stale_destination = device
        .create_buffer(&BufferDesc::new(
            "copy-stale-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();
    device.destroy_buffer(stale_destination).unwrap();

    let mut unknown_destination = device
        .create_command_list(RenderQueueClass::Copy, "copy-unknown-destination")
        .unwrap();
    unknown_destination.copy_buffer_to_buffer(source, stale_destination, 0, 0, 4);

    assert_eq!(
        device.submit(unknown_destination).unwrap_err(),
        RhiError::UnknownBuffer(stale_destination.diagnostic_id())
    );

    let mut out_of_range = device
        .create_command_list(RenderQueueClass::Copy, "copy-out-of-range")
        .unwrap();
    out_of_range.copy_buffer_to_buffer(source, destination, 0, 12, 8);

    assert_eq!(
        device.submit(out_of_range).unwrap_err(),
        RhiError::BufferCopyOutOfRange {
            source_buffer: source.diagnostic_id(),
            destination_buffer: destination.diagnostic_id(),
            source_offset: 0,
            destination_offset: 12,
            size: 8,
        }
    );
}

#[test]
fn command_list_records_compute_dispatch_and_submit_validates_pipeline() {
    let device = DeterministicRhiContractDevice::new_headless();
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
            zr_rhi::CommandListCommand::SetPipeline {
                pipeline: compute_pipeline,
            },
            zr_rhi::CommandListCommand::DispatchCompute { x: 4, y: 2, z: 1 },
        ]
    );
    assert_eq!(
        device
            .submission_status(device.submit(compute).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );

    let mut wrong_pipeline = device
        .create_command_list(RenderQueueClass::Compute, "compute-with-raster-pipeline")
        .unwrap();
    wrong_pipeline.set_pipeline(raster_pipeline);
    wrong_pipeline.dispatch_compute(1, 1, 1);
    assert_eq!(
        device.submit(wrong_pipeline).unwrap_err(),
        RhiError::InvalidPipelineUsage {
            pipeline: raster_pipeline.diagnostic_id(),
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

#[test]
fn command_list_scopes_multiple_compute_dispatches_in_one_explicit_pass() {
    let device = DeterministicRhiContractDevice::new_headless();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "scoped-compute",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let pipeline = create_compute_pipeline(&device, "scoped-compute", shader);

    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "scoped-compute-pass")
        .unwrap();
    command_list.begin_compute_pass("culling");
    command_list.set_pipeline(pipeline);
    command_list.dispatch_compute(4, 2, 1);
    command_list.dispatch_compute(2, 1, 1);
    command_list.end_compute_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginComputePass {
                label: "culling".to_string(),
            },
            zr_rhi::CommandListCommand::SetPipeline { pipeline },
            zr_rhi::CommandListCommand::DispatchCompute { x: 4, y: 2, z: 1 },
            zr_rhi::CommandListCommand::DispatchCompute { x: 2, y: 1, z: 1 },
            zr_rhi::CommandListCommand::EndComputePass,
        ]
    );
    assert_eq!(
        device
            .submission_status(device.submit(command_list).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );

    let mut unclosed = device
        .create_command_list(RenderQueueClass::Compute, "unclosed-compute-pass")
        .unwrap();
    unclosed.begin_compute_pass("unclosed");
    assert_eq!(
        device.submit(unclosed).unwrap_err(),
        RhiError::InvalidComputePass {
            reason: "command list ended with an active compute pass".to_string(),
        }
    );
}

#[test]
fn diagnostic_compute_scope_requires_one_frame_qualified_plan_and_one_ticket() {
    let device = DeterministicRhiContractDevice::new_headless();
    let mut plan = DiagnosticQueryPlan::for_frame(91, DiagnosticReadbackBudget::default());
    let pass = plan.register_pass().unwrap();
    let timestamp = plan.reserve_timestamp_scope(pass).unwrap();
    let scope = plan.pass_scope(Some(timestamp), None).unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "diagnostic-compute")
        .unwrap();
    command_list.begin_compute_pass_with_diagnostics("diagnostic-culling", scope);
    command_list.end_compute_pass();

    let packet = device
        .create_submission_packet_with_diagnostic_query_plan(
            RenderQueueClass::Compute,
            vec![command_list],
            plan,
        )
        .unwrap();
    let ticket = device.submit_packet(packet).unwrap();

    assert_eq!(
        device.submission_status(ticket).unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );
}

#[test]
fn diagnostic_query_scope_cannot_be_reused_by_multiple_passes() {
    let device = DeterministicRhiContractDevice::new_headless();
    let mut plan = DiagnosticQueryPlan::for_frame(92, DiagnosticReadbackBudget::default());
    let pass = plan.register_pass().unwrap();
    let timestamp = plan.reserve_timestamp_scope(pass).unwrap();
    let scope = plan.pass_scope(Some(timestamp), None).unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "duplicate-diagnostic-scope")
        .unwrap();
    command_list.begin_compute_pass_with_diagnostics("first", scope);
    command_list.end_compute_pass();
    command_list.begin_compute_pass_with_diagnostics("second", scope);
    command_list.end_compute_pass();

    assert!(matches!(
        device.create_submission_packet_with_diagnostic_query_plan(
            RenderQueueClass::Compute,
            vec![command_list],
            plan,
        ),
        Err(RhiError::DiagnosticQueryPlan(
            DiagnosticQueryPlanError::DuplicateScope
        ))
    ));
}
