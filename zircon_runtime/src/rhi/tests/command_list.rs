use crate::rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupHandle,
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindGroupLayoutHandle, BindingResourceType,
    BufferDesc, BufferHandle, BufferUsage, CommandList, CompareFunction, DepthStencilStateDesc,
    IndexFormat, PipelineDesc, PipelineHandle, PipelineKind, PipelineLayoutDesc,
    PipelineLayoutHandle, RasterPipelineStateDesc, RenderClearColor, RenderDevice,
    RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassDepthLoadOp,
    RenderPassDepthStencilAttachmentDesc, RenderPassStoreOp, RenderQueueClass, RhiError,
    ShaderModuleDesc, ShaderModuleHandle, ShaderStage, TextureDesc, TextureFormat, TextureHandle,
    TextureUsage, VertexAttributeDesc, VertexBufferLayoutDesc, VertexFormat, VertexInputLayoutDesc,
    VertexStepMode,
};
use crate::rhi_wgpu::{WgpuCommandList, WgpuRenderDevice};

fn create_compute_pipeline(
    device: &WgpuRenderDevice,
    label: &str,
    shader: ShaderModuleHandle,
) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            format!("{label}-layout"),
            Vec::new(),
        ))
        .unwrap();
    create_compute_pipeline_with_layout(device, label, shader, layout)
}

fn create_compute_pipeline_with_layout(
    device: &WgpuRenderDevice,
    label: &str,
    shader: ShaderModuleHandle,
    layout: PipelineLayoutHandle,
) -> PipelineHandle {
    device
        .create_pipeline(
            &PipelineDesc::new(label, PipelineKind::Compute)
                .with_layout(layout)
                .with_compute_shader(shader),
        )
        .unwrap()
}

fn create_raster_pipeline(
    device: &WgpuRenderDevice,
    label: &str,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
) -> PipelineHandle {
    create_raster_pipeline_with_vertex_input(
        device,
        label,
        vertex_shader,
        fragment_shader,
        VertexInputLayoutDesc::empty(),
    )
}

fn create_raster_pipeline_with_vertex_input(
    device: &WgpuRenderDevice,
    label: &str,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
    vertex_input: VertexInputLayoutDesc,
) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            format!("{label}-layout"),
            Vec::new(),
        ))
        .unwrap();
    create_raster_pipeline_with_layout_and_vertex_input(
        device,
        label,
        vertex_shader,
        fragment_shader,
        layout,
        vertex_input,
    )
}

fn create_raster_pipeline_with_layout_and_vertex_input(
    device: &WgpuRenderDevice,
    label: &str,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
    layout: PipelineLayoutHandle,
    vertex_input: VertexInputLayoutDesc,
) -> PipelineHandle {
    device
        .create_pipeline(
            &PipelineDesc::new(label, PipelineKind::Raster)
                .with_layout(layout)
                .with_vertex_shader(vertex_shader)
                .with_fragment_shader(fragment_shader)
                .with_raster_state(
                    RasterPipelineStateDesc::single_color(TextureFormat::Rgba8UnormSrgb)
                        .with_depth_stencil(DepthStencilStateDesc::new(
                            TextureFormat::Depth24Plus,
                            true,
                            CompareFunction::LessEqual,
                        ))
                        .with_vertex_input(vertex_input),
                ),
        )
        .unwrap()
}

fn create_uniform_bind_group_layout(
    device: &WgpuRenderDevice,
    label: &str,
) -> BindGroupLayoutHandle {
    device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            label,
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::UniformBuffer,
                vec![
                    ShaderStage::Vertex,
                    ShaderStage::Fragment,
                    ShaderStage::Compute,
                ],
            )],
        ))
        .unwrap()
}

fn create_uniform_bind_group(
    device: &WgpuRenderDevice,
    label: &str,
    layout: BindGroupLayoutHandle,
) -> BindGroupHandle {
    let buffer = device
        .create_buffer(&BufferDesc::new(
            format!("{label}-uniform"),
            64,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    device
        .create_bind_group(&BindGroupDesc::new(
            label,
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(buffer),
            )],
        ))
        .unwrap()
}

fn create_render_attachment(
    device: &WgpuRenderDevice,
    label: &str,
    format: TextureFormat,
) -> TextureHandle {
    device
        .create_texture(&TextureDesc::new(
            label,
            32,
            32,
            format,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap()
}

fn color_attachment(texture: TextureHandle) -> RenderPassColorAttachmentDesc {
    RenderPassColorAttachmentDesc::new(
        texture,
        RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
        RenderPassStoreOp::Store,
    )
}

fn depth_attachment(texture: TextureHandle) -> RenderPassDepthStencilAttachmentDesc {
    RenderPassDepthStencilAttachmentDesc::depth(
        texture,
        RenderPassDepthLoadOp::Clear(1.0),
        RenderPassStoreOp::Store,
    )
}

fn begin_default_render_pass(
    command_list: &mut dyn CommandList,
    color: TextureHandle,
    depth: TextureHandle,
) {
    command_list.begin_render_pass(
        "test-render-pass",
        vec![color_attachment(color)],
        Some(depth_attachment(depth)),
    );
}

fn create_raster_vertex_input_layout() -> VertexInputLayoutDesc {
    VertexInputLayoutDesc::new(vec![
        VertexBufferLayoutDesc::new(
            12,
            vec![VertexAttributeDesc::new(0, 0, VertexFormat::Float32x3)],
        ),
        VertexBufferLayoutDesc::new(
            16,
            vec![VertexAttributeDesc::new(1, 0, VertexFormat::Float32x4)],
        )
        .with_step_mode(VertexStepMode::Instance),
    ])
}

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

#[test]
fn command_list_records_bind_groups_and_submit_validates_raster_pipeline_layout() {
    let device = WgpuRenderDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "bound-raster-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "bound-raster-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let bind_group_layout = create_uniform_bind_group_layout(&device, "frame-layout");
    let pipeline_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "bound-raster-layout",
            vec![bind_group_layout],
        ))
        .unwrap();
    let bind_group = create_uniform_bind_group(&device, "frame-bindings", bind_group_layout);
    let pipeline = create_raster_pipeline_with_layout_and_vertex_input(
        &device,
        "bound-raster",
        vertex_shader,
        fragment_shader,
        pipeline_layout,
        VertexInputLayoutDesc::empty(),
    );
    let color =
        create_render_attachment(&device, "bound-raster-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "bound-raster-depth", TextureFormat::Depth24Plus);

    let mut draw = device
        .create_command_list(RenderQueueClass::Graphics, "bound-raster-draw")
        .unwrap();
    begin_default_render_pass(&mut *draw, color, depth);
    draw.set_pipeline(pipeline);
    draw.set_bind_group(0, bind_group);
    draw.draw(0, 3, 0, 1);
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
            crate::rhi::CommandListCommand::SetBindGroup {
                slot: 0,
                bind_group,
            },
            crate::rhi::CommandListCommand::Draw {
                vertex_start: 0,
                vertex_count: 3,
                instance_start: 0,
                instance_count: 1,
            },
            crate::rhi::CommandListCommand::EndRenderPass,
        ]
    );
    assert!(device
        .is_fence_complete(device.submit(draw).unwrap())
        .unwrap());

    let mut missing_bind_group = device
        .create_command_list(RenderQueueClass::Graphics, "missing-raster-bindings")
        .unwrap();
    begin_default_render_pass(&mut *missing_bind_group, color, depth);
    missing_bind_group.set_pipeline(pipeline);
    missing_bind_group.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(missing_bind_group).unwrap_err(),
        RhiError::InvalidBindGroupUsage {
            reason: "draw requires bind group slot 0 to be bound".to_string(),
        }
    );
}

#[test]
fn command_list_submit_validates_compute_pipeline_bind_groups() {
    let device = WgpuRenderDevice::new_headless();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "bound-compute",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let bind_group_layout = create_uniform_bind_group_layout(&device, "compute-bind-layout");
    let pipeline_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "bound-compute-layout",
            vec![bind_group_layout],
        ))
        .unwrap();
    let bind_group = create_uniform_bind_group(&device, "compute-bindings", bind_group_layout);
    let pipeline =
        create_compute_pipeline_with_layout(&device, "bound-compute", shader, pipeline_layout);

    let mut dispatch = device
        .create_command_list(RenderQueueClass::Compute, "bound-compute-dispatch")
        .unwrap();
    dispatch.set_pipeline(pipeline);
    dispatch.set_bind_group(0, bind_group);
    dispatch.dispatch_compute(2, 1, 1);
    assert!(device
        .is_fence_complete(device.submit(dispatch).unwrap())
        .unwrap());

    let mut missing_bind_group = device
        .create_command_list(RenderQueueClass::Compute, "missing-compute-bindings")
        .unwrap();
    missing_bind_group.set_pipeline(pipeline);
    missing_bind_group.dispatch_compute(1, 1, 1);
    assert_eq!(
        device.submit(missing_bind_group).unwrap_err(),
        RhiError::InvalidBindGroupUsage {
            reason: "dispatch_compute requires bind group slot 0 to be bound".to_string(),
        }
    );
}

#[test]
fn command_list_submit_validates_bind_group_layout_compatibility() {
    let device = WgpuRenderDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "layout-raster-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "layout-raster-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let expected_layout = create_uniform_bind_group_layout(&device, "expected-layout");
    let other_layout = create_uniform_bind_group_layout(&device, "other-layout");
    let pipeline_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "layout-check-pipeline-layout",
            vec![expected_layout],
        ))
        .unwrap();
    let expected_bind_group =
        create_uniform_bind_group(&device, "expected-bindings", expected_layout);
    let other_bind_group = create_uniform_bind_group(&device, "other-bindings", other_layout);
    let pipeline = create_raster_pipeline_with_layout_and_vertex_input(
        &device,
        "layout-check-raster",
        vertex_shader,
        fragment_shader,
        pipeline_layout,
        VertexInputLayoutDesc::empty(),
    );
    let color =
        create_render_attachment(&device, "layout-check-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "layout-check-depth", TextureFormat::Depth24Plus);

    let mut unknown_bind_group = device
        .create_command_list(RenderQueueClass::Graphics, "unknown-bind-group")
        .unwrap();
    unknown_bind_group.set_bind_group(0, BindGroupHandle::new(9_999));
    assert_eq!(
        device.submit(unknown_bind_group).unwrap_err(),
        RhiError::UnknownBindGroup(9_999)
    );

    let mut invalid_slot = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-bind-slot")
        .unwrap();
    invalid_slot.set_pipeline(pipeline);
    invalid_slot.set_bind_group(1, expected_bind_group);
    assert_eq!(
        device.submit(invalid_slot).unwrap_err(),
        RhiError::InvalidBindGroupUsage {
            reason: "bind group slot 1 is not declared by the active pipeline layout".to_string(),
        }
    );

    let mut mismatched_after_pipeline = device
        .create_command_list(RenderQueueClass::Graphics, "mismatched-after-pipeline")
        .unwrap();
    mismatched_after_pipeline.set_pipeline(pipeline);
    mismatched_after_pipeline.set_bind_group(0, other_bind_group);
    assert_eq!(
        device.submit(mismatched_after_pipeline).unwrap_err(),
        RhiError::InvalidBindGroupUsage {
            reason: format!(
                "bind group `{}` layout `{}` does not match pipeline layout slot 0 `{}`",
                other_bind_group.raw(),
                other_layout.raw(),
                expected_layout.raw()
            ),
        }
    );

    let mut mismatched_at_draw = device
        .create_command_list(RenderQueueClass::Graphics, "mismatched-at-draw")
        .unwrap();
    mismatched_at_draw.set_bind_group(0, other_bind_group);
    begin_default_render_pass(&mut *mismatched_at_draw, color, depth);
    mismatched_at_draw.set_pipeline(pipeline);
    mismatched_at_draw.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(mismatched_at_draw).unwrap_err(),
        RhiError::InvalidBindGroupUsage {
            reason: format!(
                "bind group slot 0 layout `{}` does not match pipeline layout `{}`",
                other_layout.raw(),
                expected_layout.raw()
            ),
        }
    );
}

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

#[test]
fn command_list_raster_draw_submit_validates_vertex_and_index_buffer_state() {
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
    let not_vertex = device
        .create_buffer(&BufferDesc::new("not-vertex", 36, BufferUsage::COPY_DST))
        .unwrap();
    let not_index = device
        .create_buffer(&BufferDesc::new("not-index", 12, BufferUsage::VERTEX))
        .unwrap();
    let color =
        create_render_attachment(&device, "raster-state-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "raster-state-depth", TextureFormat::Depth24Plus);

    let mut missing_instance = device
        .create_command_list(RenderQueueClass::Graphics, "missing-instance-buffer")
        .unwrap();
    begin_default_render_pass(&mut *missing_instance, color, depth);
    missing_instance.set_pipeline(pipeline);
    missing_instance.set_vertex_buffer(0, vertex_buffer, 0, 36);
    missing_instance.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(missing_instance).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "draw requires vertex buffer slot 1 to be bound".to_string(),
        }
    );

    let mut invalid_vertex_usage = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-vertex-usage")
        .unwrap();
    invalid_vertex_usage.set_pipeline(pipeline);
    invalid_vertex_usage.set_vertex_buffer(0, not_vertex, 0, 36);
    assert_eq!(
        device.submit(invalid_vertex_usage).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: not_vertex.raw(),
            required: BufferUsage::VERTEX,
            actual: BufferUsage::COPY_DST,
        }
    );

    let mut vertex_binding_out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "vertex-binding-out-of-range")
        .unwrap();
    vertex_binding_out_of_range.set_pipeline(pipeline);
    vertex_binding_out_of_range.set_vertex_buffer(0, vertex_buffer, 0, 40);
    assert_eq!(
        device.submit(vertex_binding_out_of_range).unwrap_err(),
        RhiError::BufferBindingOutOfRange {
            buffer: vertex_buffer.raw(),
            offset: 0,
            size: 40,
        }
    );

    let mut vertex_draw_out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "vertex-draw-out-of-range")
        .unwrap();
    begin_default_render_pass(&mut *vertex_draw_out_of_range, color, depth);
    vertex_draw_out_of_range.set_pipeline(pipeline);
    vertex_draw_out_of_range.set_vertex_buffer(0, vertex_buffer, 0, 24);
    vertex_draw_out_of_range.set_vertex_buffer(1, instance_buffer, 0, 32);
    vertex_draw_out_of_range.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(vertex_draw_out_of_range).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "vertex draw range exceeds vertex buffer slot 0".to_string(),
        }
    );

    let mut missing_index = device
        .create_command_list(RenderQueueClass::Graphics, "missing-index-buffer")
        .unwrap();
    begin_default_render_pass(&mut *missing_index, color, depth);
    missing_index.set_pipeline(pipeline);
    missing_index.set_vertex_buffer(0, vertex_buffer, 0, 36);
    missing_index.set_vertex_buffer(1, instance_buffer, 0, 32);
    missing_index.draw_indexed(0, 6, 0, 0, 1);
    assert_eq!(
        device.submit(missing_index).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "draw_indexed requires a bound index buffer".to_string(),
        }
    );

    let mut invalid_index_usage = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-index-usage")
        .unwrap();
    invalid_index_usage.set_pipeline(pipeline);
    invalid_index_usage.set_index_buffer(not_index, 0, 12, IndexFormat::Uint16);
    assert_eq!(
        device.submit(invalid_index_usage).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: not_index.raw(),
            required: BufferUsage::INDEX,
            actual: BufferUsage::VERTEX,
        }
    );

    let mut index_binding_misaligned = device
        .create_command_list(RenderQueueClass::Graphics, "index-binding-misaligned")
        .unwrap();
    index_binding_misaligned.set_pipeline(pipeline);
    index_binding_misaligned.set_index_buffer(index_buffer, 0, 3, IndexFormat::Uint16);
    assert_eq!(
        device.submit(index_binding_misaligned).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "index buffer binding size must be aligned to Uint16".to_string(),
        }
    );

    let mut index_draw_out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "index-draw-out-of-range")
        .unwrap();
    begin_default_render_pass(&mut *index_draw_out_of_range, color, depth);
    index_draw_out_of_range.set_pipeline(pipeline);
    index_draw_out_of_range.set_vertex_buffer(0, vertex_buffer, 0, 36);
    index_draw_out_of_range.set_vertex_buffer(1, instance_buffer, 0, 32);
    index_draw_out_of_range.set_index_buffer(index_buffer, 0, 4, IndexFormat::Uint16);
    index_draw_out_of_range.draw_indexed(0, 3, 0, 0, 1);
    assert_eq!(
        device.submit(index_draw_out_of_range).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "indexed draw range exceeds the bound index buffer".to_string(),
        }
    );
}

#[test]
fn command_list_buffer_copy_submit_validates_usage_flags() {
    let device = WgpuRenderDevice::new_headless();
    let invalid_source = device
        .create_buffer(&BufferDesc::new(
            "not-copy-source",
            16,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let valid_destination = device
        .create_buffer(&BufferDesc::new(
            "copy-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut source_command_list = device
        .create_command_list(RenderQueueClass::Copy, "invalid-source-copy")
        .unwrap();
    source_command_list.copy_buffer_to_buffer(invalid_source, valid_destination, 0, 0, 4);

    assert_eq!(
        device.submit(source_command_list).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: invalid_source.raw(),
            required: BufferUsage::COPY_SRC,
            actual: BufferUsage::UNIFORM,
        }
    );

    let valid_source = device
        .create_buffer(&BufferDesc::new("copy-source", 16, BufferUsage::COPY_SRC))
        .unwrap();
    let invalid_destination = device
        .create_buffer(&BufferDesc::new(
            "not-copy-destination",
            16,
            BufferUsage::STORAGE,
        ))
        .unwrap();
    let mut destination_command_list = device
        .create_command_list(RenderQueueClass::Copy, "invalid-destination-copy")
        .unwrap();
    destination_command_list.copy_buffer_to_buffer(valid_source, invalid_destination, 0, 0, 4);

    assert_eq!(
        device.submit(destination_command_list).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: invalid_destination.raw(),
            required: BufferUsage::COPY_DST,
            actual: BufferUsage::STORAGE,
        }
    );
}
