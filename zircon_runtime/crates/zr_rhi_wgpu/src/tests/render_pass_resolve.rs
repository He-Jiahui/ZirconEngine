use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    PipelineDesc, PipelineHandle, PipelineKind, PipelineLayoutDesc, RasterPipelineStateDesc,
    RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassStoreOp, RenderQueueClass, RhiError, ShaderModuleDesc, ShaderModuleHandle,
    ShaderStage, TextureDesc, TextureFormat, TextureHandle, TextureUsage,
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
    sample_count: u32,
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
    device
        .create_pipeline(
            &PipelineDesc::new(label, PipelineKind::Raster)
                .with_layout(layout)
                .with_vertex_shader(vertex)
                .with_fragment_shader(fragment)
                .with_raster_state(
                    RasterPipelineStateDesc::single_color(color_format)
                        .with_sample_count(sample_count),
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

#[test]
fn command_list_records_msaa_resolve_target_and_validates_pipeline_sample_count() {
    let device = DeterministicRhiContractDevice::new_headless();
    let msaa_color = create_render_attachment(
        &device,
        "msaa-color",
        32,
        32,
        4,
        TextureFormat::Rgba8UnormSrgb,
    );
    let resolved_color = create_render_attachment(
        &device,
        "resolved-color",
        32,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );
    let pipeline =
        create_raster_pipeline(&device, "msaa-pipeline", TextureFormat::Rgba8UnormSrgb, 4);
    let attachment = color_attachment(msaa_color).with_resolve_target(resolved_color);

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "msaa-resolve")
        .unwrap();
    command_list.begin_render_pass("msaa-resolve", vec![attachment], None);
    command_list.set_pipeline(pipeline);
    command_list.draw(0, 3, 0, 1);
    command_list.end_render_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginRenderPass {
                label: "msaa-resolve".to_string(),
                color_attachments: vec![attachment],
                depth_stencil_attachment: None,
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
    assert_eq!(
        device
            .submission_status(device.submit(command_list).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );
}

#[test]
fn command_list_render_pass_submit_validates_resolve_source_and_target_sample_counts() {
    let device = DeterministicRhiContractDevice::new_headless();
    let single_sample_color = create_render_attachment(
        &device,
        "single-sample-color",
        32,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );
    let msaa_color = create_render_attachment(
        &device,
        "msaa-color",
        32,
        32,
        4,
        TextureFormat::Rgba8UnormSrgb,
    );
    let single_sample_resolve = create_render_attachment(
        &device,
        "single-sample-resolve",
        32,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );
    let msaa_resolve = create_render_attachment(
        &device,
        "msaa-resolve",
        32,
        32,
        4,
        TextureFormat::Rgba8UnormSrgb,
    );

    let mut single_sample_source = device
        .create_command_list(RenderQueueClass::Graphics, "single-sample-source")
        .unwrap();
    single_sample_source.begin_render_pass(
        "single-sample-source",
        vec![color_attachment(single_sample_color).with_resolve_target(single_sample_resolve)],
        None,
    );
    assert_eq!(
        device.submit(single_sample_source).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 resolve target requires a multisampled color attachment"
                .to_string(),
        }
    );

    let mut multisampled_resolve = device
        .create_command_list(RenderQueueClass::Graphics, "multisampled-resolve")
        .unwrap();
    multisampled_resolve.begin_render_pass(
        "multisampled-resolve",
        vec![color_attachment(msaa_color).with_resolve_target(msaa_resolve)],
        None,
    );
    assert_eq!(
        device.submit(multisampled_resolve).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 resolve target must be single-sampled".to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_resolve_target_format_extent_and_usage() {
    let device = DeterministicRhiContractDevice::new_headless();
    let msaa_color = create_render_attachment(
        &device,
        "msaa-color",
        32,
        32,
        4,
        TextureFormat::Rgba8UnormSrgb,
    );
    let wrong_format = create_render_attachment(
        &device,
        "wrong-format",
        32,
        32,
        1,
        TextureFormat::Rgba16Float,
    );
    let wrong_extent = create_render_attachment(
        &device,
        "wrong-extent",
        16,
        32,
        1,
        TextureFormat::Rgba8UnormSrgb,
    );
    let sampled_only = device
        .create_texture(&TextureDesc::new(
            "sampled-only-resolve",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED,
        ))
        .unwrap();

    let mut wrong_format_pass = device
        .create_command_list(RenderQueueClass::Graphics, "wrong-format-resolve")
        .unwrap();
    wrong_format_pass.begin_render_pass(
        "wrong-format-resolve",
        vec![color_attachment(msaa_color).with_resolve_target(wrong_format)],
        None,
    );
    assert_eq!(
        device.submit(wrong_format_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason:
                "color attachment 0 resolve target format Rgba16Float does not match color attachment format Rgba8UnormSrgb"
                    .to_string(),
        }
    );

    let mut wrong_extent_pass = device
        .create_command_list(RenderQueueClass::Graphics, "wrong-extent-resolve")
        .unwrap();
    wrong_extent_pass.begin_render_pass(
        "wrong-extent-resolve",
        vec![color_attachment(msaa_color).with_resolve_target(wrong_extent)],
        None,
    );
    assert_eq!(
        device.submit(wrong_extent_pass).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason:
                "color attachment 0 resolve target extent 16x32 does not match render pass extent 32x32"
                    .to_string(),
        }
    );

    let mut sampled_only_pass = device
        .create_command_list(RenderQueueClass::Graphics, "sampled-only-resolve")
        .unwrap();
    sampled_only_pass.begin_render_pass(
        "sampled-only-resolve",
        vec![color_attachment(msaa_color).with_resolve_target(sampled_only)],
        None,
    );
    assert_eq!(
        device.submit(sampled_only_pass).unwrap_err(),
        RhiError::InvalidTextureUsage {
            texture: sampled_only.diagnostic_id(),
            required: TextureUsage::RENDER_ATTACHMENT,
            actual: TextureUsage::SAMPLED,
        }
    );
}

#[test]
fn command_list_render_pass_submit_rejects_duplicate_resolve_bindings() {
    let device = DeterministicRhiContractDevice::new_headless();
    let msaa_color = create_render_attachment(
        &device,
        "msaa-color",
        32,
        32,
        4,
        TextureFormat::Rgba8UnormSrgb,
    );

    let mut resolve_reuses_color = device
        .create_command_list(RenderQueueClass::Graphics, "resolve-reuses-color")
        .unwrap();
    resolve_reuses_color.begin_render_pass(
        "resolve-reuses-color",
        vec![color_attachment(msaa_color).with_resolve_target(msaa_color)],
        None,
    );
    assert_eq!(
        device.submit(resolve_reuses_color).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: format!(
                "texture `{}` mip 0 layer 0 is bound more than once in the render pass",
                msaa_color.diagnostic_id()
            ),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_pipeline_sample_count() {
    let device = DeterministicRhiContractDevice::new_headless();
    let msaa_color = create_render_attachment(
        &device,
        "msaa-color",
        32,
        32,
        4,
        TextureFormat::Rgba8UnormSrgb,
    );
    let pipeline = create_raster_pipeline(
        &device,
        "single-sample-pipeline",
        TextureFormat::Rgba8UnormSrgb,
        1,
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "pipeline-sample-mismatch")
        .unwrap();
    command_list.begin_render_pass(
        "pipeline-sample-mismatch",
        vec![color_attachment(msaa_color)],
        None,
    );
    command_list.set_pipeline(pipeline);
    command_list.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(command_list).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "render pass sample_count 4 does not match raster pipeline sample_count 1"
                .to_string(),
        }
    );
}
