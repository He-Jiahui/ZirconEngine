use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    CompareFunction, DepthStencilStateDesc, PipelineDesc, PipelineHandle, PipelineKind,
    PipelineLayoutDesc, RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc,
    RenderPassColorLoadOp, RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc,
    RenderPassStoreOp, RenderQueueClass, RenderScissorRect, RenderViewportDesc, RhiError,
    ShaderModuleDesc, ShaderModuleHandle, ShaderStage, TextureDesc, TextureFormat, TextureHandle,
    TextureUsage,
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

fn create_raster_pipeline(device: &DeterministicRhiContractDevice) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new("viewport-layout", Vec::new()))
        .unwrap();
    let vertex = create_shader(
        device,
        "viewport-vs",
        ShaderStage::Vertex,
        "vs_main",
        "@vertex fn vs_main() {}",
    );
    let fragment = create_shader(
        device,
        "viewport-fs",
        ShaderStage::Fragment,
        "fs_main",
        "@fragment fn fs_main() {}",
    );
    device
        .create_pipeline(
            &PipelineDesc::new("viewport-pipeline", PipelineKind::Raster)
                .with_layout(layout)
                .with_vertex_shader(vertex)
                .with_fragment_shader(fragment)
                .with_raster_state(
                    zr_rhi::RasterPipelineStateDesc::single_color(TextureFormat::Rgba8UnormSrgb)
                        .with_depth_stencil(DepthStencilStateDesc::new(
                            TextureFormat::Depth24Plus,
                            true,
                            CompareFunction::LessEqual,
                        )),
                ),
        )
        .unwrap()
}

fn create_render_attachment(
    device: &DeterministicRhiContractDevice,
    label: &str,
    width: u32,
    height: u32,
    sample_count: u32,
    format: TextureFormat,
) -> TextureHandle {
    device
        .create_texture(
            &TextureDesc::new(
                label,
                width,
                height,
                format,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
            )
            .with_sample_count(sample_count),
        )
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

fn viewport(width: f32, height: f32) -> RenderViewportDesc {
    RenderViewportDesc::new(0.0, 0.0, width, height, 0.0, 1.0)
}

fn scissor(width: u32, height: u32) -> RenderScissorRect {
    RenderScissorRect::new(0, 0, width, height)
}

#[test]
fn command_list_records_viewport_and_scissor_inside_render_pass() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(
        &device,
        "viewport-color",
        64,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );
    let depth = create_render_attachment(
        &device,
        "viewport-depth",
        64,
        32,
        1,
        TextureFormat::Depth24Plus,
    );
    let pipeline = create_raster_pipeline(&device);
    let viewport = RenderViewportDesc::new(4.0, 2.0, 32.0, 24.0, 0.1, 0.9);
    let scissor = RenderScissorRect::new(8, 4, 16, 12);

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "viewport-scissor")
        .unwrap();
    command_list.begin_render_pass(
        "viewport-pass",
        vec![color_attachment(color)],
        Some(depth_attachment(depth)),
    );
    command_list.set_viewport(viewport);
    command_list.set_scissor_rect(scissor);
    command_list.set_pipeline(pipeline);
    command_list.draw(0, 3, 0, 1);
    command_list.end_render_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginRenderPass {
                label: "viewport-pass".to_string(),
                color_attachments: vec![color_attachment(color)],
                depth_stencil_attachment: Some(depth_attachment(depth)),
            },
            zr_rhi::CommandListCommand::SetViewport { viewport },
            zr_rhi::CommandListCommand::SetScissorRect { rect: scissor },
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
        .is_fence_complete(device.submit(command_list).unwrap())
        .unwrap());
}

#[test]
fn command_list_viewport_and_scissor_require_active_render_pass() {
    let device = DeterministicRhiContractDevice::new_headless();

    let mut viewport_without_pass = device
        .create_command_list(RenderQueueClass::Graphics, "viewport-without-pass")
        .unwrap();
    viewport_without_pass.set_viewport(viewport(32.0, 32.0));
    assert_eq!(
        device.submit(viewport_without_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "set_viewport requires an active render pass".to_string(),
        }
    );

    let mut scissor_without_pass = device
        .create_command_list(RenderQueueClass::Graphics, "scissor-without-pass")
        .unwrap();
    scissor_without_pass.set_scissor_rect(scissor(32, 32));
    assert_eq!(
        device.submit(scissor_without_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "set_scissor_rect requires an active render pass".to_string(),
        }
    );
}

#[test]
fn command_list_viewport_submit_validates_shape_and_depth_range() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(
        &device,
        "viewport-color",
        32,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );

    let mut zero_size = device
        .create_command_list(RenderQueueClass::Graphics, "zero-size-viewport")
        .unwrap();
    zero_size.begin_render_pass("viewport-pass", vec![color_attachment(color)], None);
    zero_size.set_viewport(RenderViewportDesc::new(0.0, 0.0, 0.0, 16.0, 0.0, 1.0));
    assert_eq!(
        device.submit(zero_size).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "viewport width and height must be greater than zero".to_string(),
        }
    );

    let mut invalid_depth = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-depth-viewport")
        .unwrap();
    invalid_depth.begin_render_pass("viewport-pass", vec![color_attachment(color)], None);
    invalid_depth.set_viewport(RenderViewportDesc::new(0.0, 0.0, 16.0, 16.0, 0.8, 0.2));
    assert_eq!(
        device.submit(invalid_depth).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "viewport depth range must stay within 0.0..=1.0 and min_depth must not exceed max_depth"
                .to_string(),
        }
    );

    let mut out_of_bounds = device
        .create_command_list(RenderQueueClass::Graphics, "out-of-bounds-viewport")
        .unwrap();
    out_of_bounds.begin_render_pass("viewport-pass", vec![color_attachment(color)], None);
    out_of_bounds.set_viewport(RenderViewportDesc::new(24.0, 0.0, 16.0, 16.0, 0.0, 1.0));
    assert_eq!(
        device.submit(out_of_bounds).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "viewport exceeds render pass extent 32x32".to_string(),
        }
    );
}

#[test]
fn command_list_scissor_submit_validates_extent() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(
        &device,
        "scissor-color",
        32,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );

    let mut zero_size = device
        .create_command_list(RenderQueueClass::Graphics, "zero-size-scissor")
        .unwrap();
    zero_size.begin_render_pass("scissor-pass", vec![color_attachment(color)], None);
    zero_size.set_scissor_rect(RenderScissorRect::new(0, 0, 0, 16));
    assert_eq!(
        device.submit(zero_size).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "scissor width and height must be greater than zero".to_string(),
        }
    );

    let mut out_of_bounds = device
        .create_command_list(RenderQueueClass::Graphics, "out-of-bounds-scissor")
        .unwrap();
    out_of_bounds.begin_render_pass("scissor-pass", vec![color_attachment(color)], None);
    out_of_bounds.set_scissor_rect(RenderScissorRect::new(24, 0, 16, 16));
    assert_eq!(
        device.submit(out_of_bounds).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "scissor rectangle exceeds render pass extent 32x32".to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_attachment_extent_and_sample_count() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_attachment(
        &device,
        "extent-color",
        32,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );
    let wrong_extent = create_render_attachment(
        &device,
        "wrong-extent-depth",
        16,
        32,
        1,
        TextureFormat::Depth24Plus,
    );
    let wrong_samples = create_render_attachment(
        &device,
        "wrong-sample-depth",
        32,
        32,
        4,
        TextureFormat::Depth24Plus,
    );

    let mut extent_mismatch = device
        .create_command_list(RenderQueueClass::Graphics, "extent-mismatch")
        .unwrap();
    extent_mismatch.begin_render_pass(
        "extent-mismatch",
        vec![color_attachment(color)],
        Some(depth_attachment(wrong_extent)),
    );
    assert_eq!(
        device.submit(extent_mismatch).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "depth/stencil attachment extent 16x32 does not match render pass extent 32x32"
                .to_string(),
        }
    );

    let mut sample_mismatch = device
        .create_command_list(RenderQueueClass::Graphics, "sample-mismatch")
        .unwrap();
    sample_mismatch.begin_render_pass(
        "sample-mismatch",
        vec![color_attachment(color)],
        Some(depth_attachment(wrong_samples)),
    );
    assert_eq!(
        device.submit(sample_mismatch).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason:
                "depth/stencil attachment sample_count 4 does not match render pass sample_count 1"
                    .to_string(),
        }
    );
}
