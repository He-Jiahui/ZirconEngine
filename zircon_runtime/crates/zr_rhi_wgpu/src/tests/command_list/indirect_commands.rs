use super::*;

#[test]
fn command_list_records_indexed_indirect_and_fixed_multi_draws() {
    let device = DeterministicRhiContractDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "indirect-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "indirect-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let pipeline = create_raster_pipeline_with_vertex_input(
        &device,
        "indexed-indirect",
        vertex_shader,
        fragment_shader,
        create_raster_vertex_input_layout(),
    );
    let vertex_buffer = device
        .create_buffer(&BufferDesc::new(
            "indirect-vertices",
            36,
            BufferUsage::VERTEX,
        ))
        .unwrap();
    let instance_buffer = device
        .create_buffer(&BufferDesc::new(
            "indirect-instances",
            32,
            BufferUsage::VERTEX,
        ))
        .unwrap();
    let index_buffer = device
        .create_buffer(&BufferDesc::new("indirect-indices", 12, BufferUsage::INDEX))
        .unwrap();
    let arguments = device
        .create_buffer(&BufferDesc::new(
            "indexed-indirect-arguments",
            64,
            BufferUsage::INDIRECT,
        ))
        .unwrap();
    let count_buffer = device
        .create_buffer(&BufferDesc::new(
            "indexed-indirect-count",
            4,
            BufferUsage::INDIRECT,
        ))
        .unwrap();
    let color = create_render_attachment(&device, "indirect-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "indirect-depth", TextureFormat::Depth24Plus);

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "indexed-indirect")
        .unwrap();
    begin_default_render_pass(&mut *command_list, color, depth);
    command_list.set_pipeline(pipeline);
    command_list.set_vertex_buffer(0, vertex_buffer, 0, 36);
    command_list.set_vertex_buffer(1, instance_buffer, 0, 32);
    command_list.set_index_buffer(index_buffer, 0, 12, IndexFormat::Uint16);
    command_list.draw_indexed_indirect(arguments, 4);
    command_list.multi_draw_indexed_indirect(arguments, 24, 2);
    command_list.multi_draw_indexed_indirect_count(arguments, 0, count_buffer, 0, 2);
    command_list.end_render_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginRenderPass {
                label: "test-render-pass".to_string(),
                color_attachments: vec![color_attachment(color)],
                depth_stencil_attachment: Some(depth_attachment(depth)),
            },
            zr_rhi::CommandListCommand::SetPipeline { pipeline },
            zr_rhi::CommandListCommand::SetVertexBuffer {
                slot: 0,
                buffer: vertex_buffer,
                offset: 0,
                size: 36,
            },
            zr_rhi::CommandListCommand::SetVertexBuffer {
                slot: 1,
                buffer: instance_buffer,
                offset: 0,
                size: 32,
            },
            zr_rhi::CommandListCommand::SetIndexBuffer {
                buffer: index_buffer,
                offset: 0,
                size: 12,
                format: IndexFormat::Uint16,
            },
            zr_rhi::CommandListCommand::DrawIndexedIndirect {
                arguments,
                offset: 4,
            },
            zr_rhi::CommandListCommand::MultiDrawIndexedIndirect {
                arguments,
                offset: 24,
                count: 2,
            },
            zr_rhi::CommandListCommand::MultiDrawIndexedIndirectCount {
                arguments,
                offset: 0,
                count_buffer,
                count_offset: 0,
                max_count: 2,
            },
            zr_rhi::CommandListCommand::EndRenderPass,
        ]
    );
    assert_eq!(
        device
            .submission_status(device.submit(command_list).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );
}

#[test]
fn indirect_draws_reject_invalid_argument_buffer_usage_alignment_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "invalid-indirect-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "invalid-indirect-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let pipeline =
        create_raster_pipeline(&device, "invalid-indirect", vertex_shader, fragment_shader);
    let color = create_render_attachment(
        &device,
        "invalid-indirect-color",
        TextureFormat::Rgba8UnormSrgb,
    );
    let depth = create_render_attachment(
        &device,
        "invalid-indirect-depth",
        TextureFormat::Depth24Plus,
    );
    let wrong_usage = device
        .create_buffer(&BufferDesc::new(
            "wrong-indirect-usage",
            64,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut wrong_usage_list = device
        .create_command_list(RenderQueueClass::Graphics, "wrong-indirect-usage")
        .unwrap();
    begin_default_render_pass(&mut *wrong_usage_list, color, depth);
    wrong_usage_list.set_pipeline(pipeline);
    wrong_usage_list.draw_indirect(wrong_usage, 0);
    wrong_usage_list.end_render_pass();
    assert_eq!(
        device.submit(wrong_usage_list).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: wrong_usage.diagnostic_id(),
            required: BufferUsage::INDIRECT,
            actual: BufferUsage::COPY_DST,
        }
    );

    let arguments = device
        .create_buffer(&BufferDesc::new(
            "invalid-indirect-arguments",
            32,
            BufferUsage::INDIRECT,
        ))
        .unwrap();
    let mut unaligned = device
        .create_command_list(RenderQueueClass::Graphics, "unaligned-indirect")
        .unwrap();
    begin_default_render_pass(&mut *unaligned, color, depth);
    unaligned.set_pipeline(pipeline);
    unaligned.draw_indirect(arguments, 2);
    unaligned.end_render_pass();
    assert_eq!(
        device.submit(unaligned).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "indirect argument offset must be a multiple of four".to_string(),
        }
    );

    let mut out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "out-of-range-multi-indirect")
        .unwrap();
    begin_default_render_pass(&mut *out_of_range, color, depth);
    out_of_range.set_pipeline(pipeline);
    out_of_range.multi_draw_indexed_indirect(arguments, 0, 2);
    out_of_range.end_render_pass();
    assert_eq!(
        device.submit(out_of_range).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "indirect argument range exceeds buffer".to_string(),
        }
    );

    let count_buffer = device
        .create_buffer(&BufferDesc::new(
            "invalid-indirect-count",
            2,
            BufferUsage::INDIRECT,
        ))
        .unwrap();
    let arguments = device
        .create_buffer(&BufferDesc::new(
            "count-indirect-arguments",
            40,
            BufferUsage::INDIRECT,
        ))
        .unwrap();
    let index_buffer = device
        .create_buffer(&BufferDesc::new(
            "invalid-indirect-count-indices",
            6,
            BufferUsage::INDEX,
        ))
        .unwrap();
    let mut invalid_count = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-indirect-count")
        .unwrap();
    begin_default_render_pass(&mut *invalid_count, color, depth);
    invalid_count.set_pipeline(pipeline);
    invalid_count.set_index_buffer(index_buffer, 0, 6, IndexFormat::Uint16);
    invalid_count.multi_draw_indexed_indirect_count(arguments, 0, count_buffer, 0, 2);
    invalid_count.end_render_pass();
    assert_eq!(
        device.submit(invalid_count).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "indirect count range exceeds buffer".to_string(),
        }
    );
}

#[test]
fn command_list_records_compute_indirect_dispatch() {
    let device = DeterministicRhiContractDevice::new_headless();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "indirect-dispatch",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let pipeline = create_compute_pipeline(&device, "indirect-dispatch", shader);
    let arguments = device
        .create_buffer(&BufferDesc::new(
            "indirect-dispatch-arguments",
            12,
            BufferUsage::INDIRECT,
        ))
        .unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "indirect-dispatch")
        .unwrap();
    command_list.begin_compute_pass("culling");
    command_list.set_pipeline(pipeline);
    command_list.dispatch_compute_indirect(arguments, 0);
    command_list.end_compute_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginComputePass {
                label: "culling".to_string(),
            },
            zr_rhi::CommandListCommand::SetPipeline { pipeline },
            zr_rhi::CommandListCommand::DispatchComputeIndirect {
                arguments,
                offset: 0,
            },
            zr_rhi::CommandListCommand::EndComputePass,
        ]
    );
    assert_eq!(
        device
            .submission_status(device.submit(command_list).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );
}
