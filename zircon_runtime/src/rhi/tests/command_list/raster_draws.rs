use super::*;

#[test]
fn command_list_records_raster_draws_and_submit_validates_bound_buffers() {
    let device = WgpuRenderDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "raster-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "raster-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let pipeline = create_raster_pipeline_with_vertex_input(
        &device,
        "raster-forward",
        vertex_shader,
        fragment_shader,
        create_raster_vertex_input_layout(),
    );
    let vertex_buffer = device
        .create_buffer(&BufferDesc::new("raster-vertices", 36, BufferUsage::VERTEX))
        .unwrap();
    let instance_buffer = device
        .create_buffer(&BufferDesc::new(
            "raster-instances",
            32,
            BufferUsage::VERTEX,
        ))
        .unwrap();
    let index_buffer = device
        .create_buffer(&BufferDesc::new("raster-indices", 12, BufferUsage::INDEX))
        .unwrap();
    let color = create_render_attachment(&device, "raster-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "raster-depth", TextureFormat::Depth24Plus);

    let mut draw = device
        .create_command_list(RenderQueueClass::Graphics, "raster-draw")
        .unwrap();
    begin_default_render_pass(&mut *draw, color, depth);
    draw.set_pipeline(pipeline);
    draw.set_vertex_buffer(0, vertex_buffer, 0, 36);
    draw.set_vertex_buffer(1, instance_buffer, 0, 32);
    draw.set_index_buffer(index_buffer, 0, 12, IndexFormat::Uint16);
    draw.draw(0, 3, 0, 2);
    draw.draw_indexed(0, 6, 0, 0, 2);
    draw.end_render_pass();

    assert_eq!(
        draw.recorded_commands(),
        &[
            crate::rhi::CommandListCommand::BeginRenderPass {
                label: "test-render-pass".to_string(),
                color_attachments: vec![color_attachment(color)],
                depth_stencil_attachment: Some(depth_attachment(depth)),
            },
            crate::rhi::CommandListCommand::SetPipeline { pipeline },
            crate::rhi::CommandListCommand::SetVertexBuffer {
                slot: 0,
                buffer: vertex_buffer,
                offset: 0,
                size: 36,
            },
            crate::rhi::CommandListCommand::SetVertexBuffer {
                slot: 1,
                buffer: instance_buffer,
                offset: 0,
                size: 32,
            },
            crate::rhi::CommandListCommand::SetIndexBuffer {
                buffer: index_buffer,
                offset: 0,
                size: 12,
                format: IndexFormat::Uint16,
            },
            crate::rhi::CommandListCommand::Draw {
                vertex_start: 0,
                vertex_count: 3,
                instance_start: 0,
                instance_count: 2,
            },
            crate::rhi::CommandListCommand::DrawIndexed {
                index_start: 0,
                index_count: 6,
                base_vertex: 0,
                instance_start: 0,
                instance_count: 2,
            },
            crate::rhi::CommandListCommand::EndRenderPass,
        ]
    );
    assert!(device
        .is_fence_complete(device.submit(draw).unwrap())
        .unwrap());
}

#[test]
fn command_list_allows_generated_vertex_draws_without_vertex_buffers() {
    let device = WgpuRenderDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "fullscreen-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "fullscreen-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let pipeline = create_raster_pipeline(&device, "fullscreen", vertex_shader, fragment_shader);
    let color =
        create_render_attachment(&device, "fullscreen-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "fullscreen-depth", TextureFormat::Depth24Plus);
    let mut draw = device
        .create_command_list(RenderQueueClass::Graphics, "fullscreen-draw")
        .unwrap();

    begin_default_render_pass(&mut *draw, color, depth);
    draw.set_pipeline(pipeline);
    draw.draw(0, 3, 0, 1);
    draw.end_render_pass();

    assert!(device
        .is_fence_complete(device.submit(draw).unwrap())
        .unwrap());
}

#[test]
fn command_list_raster_draw_submit_validates_pipeline_queue_and_counts() {
    let device = WgpuRenderDevice::new_headless();
    let compute_shader = device
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
    let compute_pipeline = create_compute_pipeline(&device, "compute-fill", compute_shader);
    let raster_pipeline =
        create_raster_pipeline(&device, "forward-generated", vertex_shader, fragment_shader);

    let mut missing_pipeline = device
        .create_command_list(RenderQueueClass::Graphics, "draw-without-pipeline")
        .unwrap();
    missing_pipeline.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(missing_pipeline).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "raster draw requires a bound raster pipeline".to_string(),
        }
    );

    let mut wrong_pipeline = device
        .create_command_list(RenderQueueClass::Graphics, "draw-with-compute-pipeline")
        .unwrap();
    wrong_pipeline.set_pipeline(compute_pipeline);
    wrong_pipeline.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(wrong_pipeline).unwrap_err(),
        RhiError::InvalidPipelineUsage {
            pipeline: compute_pipeline.raw(),
            required: PipelineKind::Raster,
            actual: PipelineKind::Compute,
        }
    );

    let mut compute_queue_draw = device
        .create_command_list(RenderQueueClass::Compute, "compute-queue-draw")
        .unwrap();
    compute_queue_draw.set_pipeline(raster_pipeline);
    compute_queue_draw.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(compute_queue_draw).unwrap_err(),
        RhiError::InvalidCommandQueue {
            queue: RenderQueueClass::Compute,
            command: "draw".to_string(),
        }
    );

    let mut zero_count = device
        .create_command_list(RenderQueueClass::Graphics, "zero-count-draw")
        .unwrap();
    zero_count.set_pipeline(raster_pipeline);
    zero_count.draw(0, 0, 0, 1);
    assert_eq!(
        device.submit(zero_count).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "draw and instance counts must be greater than zero".to_string(),
        }
    );
}
