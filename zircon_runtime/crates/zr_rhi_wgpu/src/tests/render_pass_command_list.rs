use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    BufferDesc, BufferUsage, CompareFunction, DepthStencilStateDesc, PipelineDesc, PipelineHandle,
    PipelineKind, PipelineLayoutDesc, RenderClearColor, RenderDevice,
    RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassDepthLoadOp,
    RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp, RenderPassStoreOp,
    RenderQueueClass, RhiError, ShaderModuleDesc, ShaderModuleHandle, ShaderStage, TextureDesc,
    TextureFormat, TextureHandle, TextureUsage,
};

fn create_shader(
    device: &DeterministicRhiContractDevice,
    label: &str,
    stage: ShaderStage,
    entry_point: &str,
    source: &str,
) -> ShaderModuleHandle {
    device
        .create_shader_module(&ShaderModuleDesc::new(label, stage, entry_point, source))
        .unwrap()
}

fn create_raster_pipeline(
    device: &DeterministicRhiContractDevice,
    label: &str,
    color_format: TextureFormat,
    depth_format: Option<TextureFormat>,
) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            format!("{label}-layout"),
            Vec::new(),
        ))
        .unwrap();
    let vertex = create_shader(
        device,
        &format!("{label}-vs"),
        ShaderStage::Vertex,
        "vs_main",
        "@vertex fn vs_main() {}",
    );
    let fragment = create_shader(
        device,
        &format!("{label}-fs"),
        ShaderStage::Fragment,
        "fs_main",
        "@fragment fn fs_main() {}",
    );
    let mut raster_state = zr_rhi::RasterPipelineStateDesc::single_color(color_format);
    if let Some(depth_format) = depth_format {
        raster_state = raster_state.with_depth_stencil(DepthStencilStateDesc::new(
            depth_format,
            true,
            CompareFunction::LessEqual,
        ));
    }
    device
        .create_pipeline(
            &PipelineDesc::new(label, PipelineKind::Raster)
                .with_layout(layout)
                .with_vertex_shader(vertex)
                .with_fragment_shader(fragment)
                .with_raster_state(raster_state),
        )
        .unwrap()
}

fn create_compute_pipeline(device: &DeterministicRhiContractDevice) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new("compute-layout", Vec::new()))
        .unwrap();
    let shader = create_shader(
        device,
        "compute",
        ShaderStage::Compute,
        "main",
        "@compute @workgroup_size(1) fn main() {}",
    );
    device
        .create_pipeline(
            &PipelineDesc::new("compute", PipelineKind::Compute)
                .with_layout(layout)
                .with_compute_shader(shader),
        )
        .unwrap()
}

fn create_render_attachment(
    device: &DeterministicRhiContractDevice,
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

#[test]
fn command_list_records_render_pass_and_validates_raster_attachments() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(&device, "scene-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "scene-depth", TextureFormat::Depth24Plus);
    let pipeline = create_raster_pipeline(
        &device,
        "forward",
        TextureFormat::Rgba8UnormSrgb,
        Some(TextureFormat::Depth24Plus),
    );

    let mut draw = device
        .create_command_list(RenderQueueClass::Graphics, "forward-pass")
        .unwrap();
    draw.begin_render_pass(
        "forward-main",
        vec![color_attachment(color)],
        Some(depth_attachment(depth)),
    );
    draw.set_pipeline(pipeline);
    draw.draw(0, 3, 0, 1);
    draw.end_render_pass();

    assert_eq!(
        draw.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginRenderPass {
                label: "forward-main".to_string(),
                color_attachments: vec![color_attachment(color)],
                depth_stencil_attachment: Some(depth_attachment(depth)),
            },
            zr_rhi::CommandListCommand::SetPipeline { pipeline },
            zr_rhi::CommandListCommand::Draw {
                vertex_start: 0,
                vertex_count: 3,
                instance_start: 0,
                instance_count: 1,
            },
            zr_rhi::CommandListCommand::EndRenderPass,
        ]
    );
    assert!(device
        .is_fence_complete(device.submit(draw).unwrap())
        .unwrap());
}

#[test]
fn command_list_render_pass_submit_validates_pass_lifetime_and_queue() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(&device, "scene-color", TextureFormat::Rgba8UnormSrgb);
    let pipeline =
        create_raster_pipeline(&device, "fullscreen", TextureFormat::Rgba8UnormSrgb, None);

    let mut draw_without_pass = device
        .create_command_list(RenderQueueClass::Graphics, "draw-without-pass")
        .unwrap();
    draw_without_pass.set_pipeline(pipeline);
    draw_without_pass.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(draw_without_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "draw requires an active render pass".to_string(),
        }
    );

    let mut nested = device
        .create_command_list(RenderQueueClass::Graphics, "nested-render-pass")
        .unwrap();
    nested.begin_render_pass("outer", vec![color_attachment(color)], None);
    nested.begin_render_pass("inner", vec![color_attachment(color)], None);
    assert_eq!(
        device.submit(nested).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "render pass is already active".to_string(),
        }
    );

    let mut unclosed = device
        .create_command_list(RenderQueueClass::Graphics, "unclosed-render-pass")
        .unwrap();
    unclosed.begin_render_pass("unclosed", vec![color_attachment(color)], None);
    assert_eq!(
        device.submit(unclosed).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "command list ended with an active render pass".to_string(),
        }
    );

    let mut copy_queue = device
        .create_command_list(RenderQueueClass::Copy, "copy-render-pass")
        .unwrap();
    copy_queue.begin_render_pass("copy-pass", vec![color_attachment(color)], None);
    assert_eq!(
        device.submit(copy_queue).unwrap_err(),
        RhiError::InvalidCommandQueue {
            queue: RenderQueueClass::Copy,
            command: "begin_render_pass".to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_attachment_usage_and_formats() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(&device, "scene-color", TextureFormat::Rgba8UnormSrgb);
    let wrong_color = create_render_attachment(&device, "hdr-color", TextureFormat::Rgba16Float);
    let depth = create_render_attachment(&device, "scene-depth", TextureFormat::Depth24Plus);
    let not_attachment = device
        .create_texture(&TextureDesc::new(
            "sampled-only",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let pipeline = create_raster_pipeline(&device, "forward", TextureFormat::Rgba8UnormSrgb, None);

    let mut sampled_only = device
        .create_command_list(RenderQueueClass::Graphics, "sampled-only-attachment")
        .unwrap();
    sampled_only.begin_render_pass(
        "invalid-usage",
        vec![color_attachment(not_attachment)],
        None,
    );
    assert_eq!(
        device.submit(sampled_only).unwrap_err(),
        RhiError::InvalidTextureUsage {
            texture: not_attachment.raw(),
            required: TextureUsage::RENDER_ATTACHMENT,
            actual: TextureUsage::SAMPLED,
        }
    );

    let mut color_uses_depth = device
        .create_command_list(RenderQueueClass::Graphics, "depth-as-color")
        .unwrap();
    color_uses_depth.begin_render_pass("depth-as-color", vec![color_attachment(depth)], None);
    assert_eq!(
        device.submit(color_uses_depth).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 must use a color texture format".to_string(),
        }
    );

    let mut wrong_pipeline_format = device
        .create_command_list(RenderQueueClass::Graphics, "wrong-pipeline-format")
        .unwrap();
    wrong_pipeline_format.begin_render_pass(
        "wrong-format",
        vec![color_attachment(wrong_color)],
        None,
    );
    wrong_pipeline_format.set_pipeline(pipeline);
    wrong_pipeline_format.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(wrong_pipeline_format).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 format Rgba16Float does not match pipeline target Rgba8UnormSrgb"
                .to_string(),
        }
    );

    let mut duplicate = device
        .create_command_list(RenderQueueClass::Graphics, "duplicate-attachment")
        .unwrap();
    duplicate.begin_render_pass(
        "duplicate",
        vec![color_attachment(color), color_attachment(color)],
        None,
    );
    assert_eq!(
        device.submit(duplicate).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: format!(
                "texture `{}` mip 0 layer 0 is bound more than once in the render pass",
                color.raw()
            ),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_depth_stencil_attachment_contract() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(&device, "scene-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "scene-depth", TextureFormat::Depth24Plus);
    let stencil_depth =
        create_render_attachment(&device, "stencil-depth", TextureFormat::Depth24PlusStencil8);
    let depth_pipeline = create_raster_pipeline(
        &device,
        "depth-forward",
        TextureFormat::Rgba8UnormSrgb,
        Some(TextureFormat::Depth24Plus),
    );
    let color_only_pipeline =
        create_raster_pipeline(&device, "color-only", TextureFormat::Rgba8UnormSrgb, None);

    let mut missing_depth = device
        .create_command_list(RenderQueueClass::Graphics, "missing-depth")
        .unwrap();
    missing_depth.begin_render_pass("missing-depth", vec![color_attachment(color)], None);
    missing_depth.set_pipeline(depth_pipeline);
    missing_depth.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(missing_depth).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "raster pipeline expects a depth/stencil attachment".to_string(),
        }
    );

    let mut unexpected_depth = device
        .create_command_list(RenderQueueClass::Graphics, "unexpected-depth")
        .unwrap();
    unexpected_depth.begin_render_pass(
        "unexpected-depth",
        vec![color_attachment(color)],
        Some(depth_attachment(depth)),
    );
    unexpected_depth.set_pipeline(color_only_pipeline);
    unexpected_depth.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(unexpected_depth).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "render pass declares a depth/stencil attachment but pipeline does not"
                .to_string(),
        }
    );

    let mut stencil_on_depth_only = device
        .create_command_list(RenderQueueClass::Graphics, "stencil-on-depth-only")
        .unwrap();
    stencil_on_depth_only.begin_render_pass(
        "stencil-on-depth-only",
        vec![color_attachment(color)],
        Some(
            depth_attachment(depth)
                .with_stencil(RenderPassStencilLoadOp::Clear(0), RenderPassStoreOp::Store),
        ),
    );
    assert_eq!(
        device.submit(stencil_on_depth_only).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "stencil operations require a stencil-capable depth format".to_string(),
        }
    );

    let mut stencil_ok = device
        .create_command_list(RenderQueueClass::Graphics, "stencil-ok")
        .unwrap();
    stencil_ok.begin_render_pass(
        "stencil-ok",
        vec![color_attachment(color)],
        Some(
            depth_attachment(stencil_depth)
                .with_stencil(RenderPassStencilLoadOp::Load, RenderPassStoreOp::Discard),
        ),
    );
    stencil_ok.end_render_pass();
    assert!(device
        .is_fence_complete(device.submit(stencil_ok).unwrap())
        .unwrap());
}

#[test]
fn command_list_render_pass_rejects_compute_and_copy_work_inside_pass() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(&device, "scene-color", TextureFormat::Rgba8UnormSrgb);
    let compute_pipeline = create_compute_pipeline(&device);
    let source = device
        .create_buffer(&BufferDesc::new("source", 16, BufferUsage::COPY_SRC))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new("destination", 16, BufferUsage::COPY_DST))
        .unwrap();

    let mut compute_inside_pass = device
        .create_command_list(RenderQueueClass::Graphics, "compute-inside-pass")
        .unwrap();
    compute_inside_pass.begin_render_pass("render-pass", vec![color_attachment(color)], None);
    compute_inside_pass.set_pipeline(compute_pipeline);
    compute_inside_pass.dispatch_compute(1, 1, 1);
    assert_eq!(
        device.submit(compute_inside_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "dispatch_compute cannot be recorded inside an active render pass".to_string(),
        }
    );

    let mut copy_inside_pass = device
        .create_command_list(RenderQueueClass::Graphics, "copy-inside-pass")
        .unwrap();
    copy_inside_pass.begin_render_pass("render-pass", vec![color_attachment(color)], None);
    copy_inside_pass.copy_buffer_to_buffer(source, destination, 0, 0, 4);
    assert_eq!(
        device.submit(copy_inside_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "copy_buffer_to_buffer cannot be recorded inside an active render pass"
                .to_string(),
        }
    );
}
