use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    PipelineDesc, PipelineHandle, PipelineKind, PipelineLayoutDesc, RasterPipelineStateDesc,
    RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassStoreOp, RenderPassTextureViewDesc, RenderQueueClass, RhiError, ShaderModuleDesc,
    ShaderModuleHandle, ShaderStage, TextureDesc, TextureDimension, TextureFormat, TextureHandle,
    TextureUsage, TextureViewDesc, TextureViewDimension,
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
                .with_raster_state(RasterPipelineStateDesc::single_color(color_format)),
        )
        .unwrap()
}

fn create_render_texture(
    device: &DeterministicRhiContractDevice,
    desc: TextureDesc,
) -> TextureHandle {
    device.create_texture(&desc).unwrap()
}

fn color_attachment(texture: TextureHandle) -> RenderPassColorAttachmentDesc {
    RenderPassColorAttachmentDesc::new(
        texture,
        RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
        RenderPassStoreOp::Store,
    )
}

#[test]
fn command_list_records_render_pass_texture_views() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_texture(
        &device,
        TextureDesc::new(
            "mipped-color",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        )
        .with_mip_levels(3),
    );
    let view = RenderPassTextureViewDesc::new(color)
        .with_mip_level(1)
        .with_array_layer(0);
    let attachment = color_attachment(color).with_view(view);

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "mip-view-recording")
        .unwrap();
    command_list.begin_render_pass("mip-view", vec![attachment], None);
    command_list.end_render_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            zr_rhi::CommandListCommand::BeginRenderPass {
                label: "mip-view".to_string(),
                color_attachments: vec![attachment],
                depth_stencil_attachment: None,
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
fn command_list_render_pass_submit_accepts_matching_registered_attachment_view() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_texture(
        &device,
        TextureDesc::new(
            "registered-view-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        ),
    );
    let view = device
        .create_texture_view(&TextureViewDesc::new(
            "registered-view-color-default",
            color,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let attachment = color_attachment(color)
        .with_view(RenderPassTextureViewDesc::new(color).with_registered_view(view));

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "registered-attachment-view")
        .unwrap();
    command_list.begin_render_pass("registered-attachment-view", vec![attachment], None);
    command_list.end_render_pass();

    assert_eq!(
        device
            .submission_status(device.submit(command_list).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );
}

#[test]
fn command_list_render_pass_submit_rejects_registered_attachment_view_from_other_texture() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_texture(
        &device,
        TextureDesc::new(
            "registered-view-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        ),
    );
    let other = create_render_texture(
        &device,
        TextureDesc::new(
            "registered-view-other",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        ),
    );
    let foreign_view = device
        .create_texture_view(&TextureViewDesc::new(
            "registered-view-other-default",
            other,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let attachment = color_attachment(color)
        .with_view(RenderPassTextureViewDesc::new(color).with_registered_view(foreign_view));

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "foreign-attachment-view")
        .unwrap();
    command_list.begin_render_pass("foreign-attachment-view", vec![attachment], None);

    assert!(matches!(
        device.submit(command_list),
        Err(RhiError::InvalidRenderPass { reason }) if reason.contains("registered view")
    ));
}

#[test]
fn command_list_render_pass_submit_rejects_non_matching_registered_attachment_subresources() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_texture(
        &device,
        TextureDesc::new(
            "registered-view-array-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(2)
        .with_mip_levels(2),
    );
    let cases = [
        (
            TextureViewDesc::new(
                "registered-view-array-dimension",
                color,
                TextureViewDimension::D2Array,
            )
            .with_mip_range(0, 1)
            .with_array_layer_range(0, 1),
            "must have D2 dimension",
        ),
        (
            TextureViewDesc::new(
                "registered-view-array-full-mip-range",
                color,
                TextureViewDimension::D2,
            )
            .with_array_layer_range(0, 1),
            "must select exactly one mip level and array layer",
        ),
        (
            TextureViewDesc::new(
                "registered-view-array-other-subresource",
                color,
                TextureViewDimension::D2,
            )
            .with_mip_range(1, 1)
            .with_array_layer_range(0, 1),
            "subresource does not match",
        ),
    ];

    for (index, (view_desc, expected_reason)) in cases.into_iter().enumerate() {
        let registered_view = device.create_texture_view(&view_desc).unwrap();
        let attachment = color_attachment(color)
            .with_view(RenderPassTextureViewDesc::new(color).with_registered_view(registered_view));
        let mut command_list = device
            .create_command_list(
                RenderQueueClass::Graphics,
                &format!("registered-attachment-shape-{index}"),
            )
            .unwrap();
        command_list.begin_render_pass(
            &format!("registered-attachment-shape-{index}"),
            vec![attachment],
            None,
        );

        assert!(matches!(
            device.submit(command_list),
            Err(RhiError::InvalidRenderPass { reason }) if reason.contains(expected_reason)
        ));
    }
}

#[test]
fn command_list_render_pass_submit_uses_mip_extent_for_view_compatibility() {
    let device = DeterministicRhiContractDevice::new_headless();
    let base = create_render_texture(
        &device,
        TextureDesc::new(
            "base-color",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_mip_levels(3),
    );
    let matching_mip = create_render_texture(
        &device,
        TextureDesc::new(
            "matching-mip-color",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_mip_levels(3),
    );
    let wrong_mip = create_render_texture(
        &device,
        TextureDesc::new(
            "wrong-mip-color",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_mip_levels(3),
    );

    let mut matching = device
        .create_command_list(RenderQueueClass::Graphics, "matching-mip-views")
        .unwrap();
    matching.begin_render_pass(
        "matching-mip-views",
        vec![
            color_attachment(base)
                .with_view(RenderPassTextureViewDesc::new(base).with_mip_level(1)),
            color_attachment(matching_mip)
                .with_view(RenderPassTextureViewDesc::new(matching_mip).with_mip_level(1)),
        ],
        None,
    );
    matching.end_render_pass();
    assert_eq!(
        device
            .submission_status(device.submit(matching).unwrap())
            .unwrap(),
        zr_rhi::SubmissionStatus::Completed
    );

    let mut mismatched = device
        .create_command_list(RenderQueueClass::Graphics, "mismatched-mip-views")
        .unwrap();
    mismatched.begin_render_pass(
        "mismatched-mip-views",
        vec![
            color_attachment(base)
                .with_view(RenderPassTextureViewDesc::new(base).with_mip_level(1)),
            color_attachment(wrong_mip)
                .with_view(RenderPassTextureViewDesc::new(wrong_mip).with_mip_level(2)),
        ],
        None,
    );
    assert_eq!(
        device.submit(mismatched).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 1 extent 16x8 does not match render pass extent 32x16"
                .to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_view_mip_and_array_layer_bounds() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color = create_render_texture(
        &device,
        TextureDesc::new(
            "bounded-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_mip_levels(2),
    );
    let array = create_render_texture(
        &device,
        TextureDesc::new(
            "bounded-array",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(2),
    );

    let mut invalid_mip = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-mip-view")
        .unwrap();
    invalid_mip.begin_render_pass(
        "invalid-mip-view",
        vec![color_attachment(color)
            .with_view(RenderPassTextureViewDesc::new(color).with_mip_level(2))],
        None,
    );
    assert_eq!(
        device.submit(invalid_mip).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 mip level 2 is outside texture mip_levels 2".to_string(),
        }
    );

    let mut invalid_layer = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-layer-view")
        .unwrap();
    invalid_layer.begin_render_pass(
        "invalid-layer-view",
        vec![color_attachment(array)
            .with_view(RenderPassTextureViewDesc::new(array).with_array_layer(2))],
        None,
    );
    assert_eq!(
        device.submit(invalid_layer).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 array layer 2 is outside texture layer count 2".to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_allows_distinct_array_layer_attachments() {
    let device = DeterministicRhiContractDevice::new_headless();
    let array = create_render_texture(
        &device,
        TextureDesc::new(
            "array-color",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(2),
    );
    let pipeline = create_raster_pipeline(
        &device,
        "array-layer-pipeline",
        TextureFormat::Rgba8UnormSrgb,
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "array-layer-attachments")
        .unwrap();
    command_list.begin_render_pass(
        "array-layer-attachments",
        vec![
            color_attachment(array)
                .with_view(RenderPassTextureViewDesc::new(array).with_array_layer(0)),
            color_attachment(array)
                .with_view(RenderPassTextureViewDesc::new(array).with_array_layer(1)),
        ],
        None,
    );
    command_list.set_pipeline(pipeline);
    command_list.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(command_list).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "render pass declares 2 color attachments but pipeline expects 1".to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_resolve_target_view_shape() {
    let device = DeterministicRhiContractDevice::new_headless();
    let msaa_color = create_render_texture(
        &device,
        TextureDesc::new(
            "msaa-color",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_sample_count(4),
    );
    let resolve = create_render_texture(
        &device,
        TextureDesc::new(
            "mipped-resolve",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        )
        .with_mip_levels(2),
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "resolve-view-shape")
        .unwrap();
    command_list.begin_render_pass(
        "resolve-view-shape",
        vec![color_attachment(msaa_color)
            .with_resolve_view(RenderPassTextureViewDesc::new(resolve).with_mip_level(1))],
        None,
    );
    assert_eq!(
        device.submit(command_list).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 resolve target extent 32x16 does not match render pass extent 64x32"
                .to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_rejects_non_2d_attachment_dimensions() {
    let device = DeterministicRhiContractDevice::new_headless();

    for (label, dimension) in [
        ("one-dimensional", TextureDimension::D1),
        ("three-dimensional", TextureDimension::D3),
    ] {
        let height = match dimension {
            TextureDimension::D1 => 1,
            _ => 32,
        };
        let texture = create_render_texture(
            &device,
            TextureDesc::new(
                format!("{label}-color"),
                32,
                height,
                TextureFormat::Rgba8UnormSrgb,
                TextureUsage::RENDER_ATTACHMENT,
            )
            .with_dimension(dimension),
        );
        let mut command_list = device
            .create_command_list(RenderQueueClass::Graphics, label)
            .unwrap();
        command_list.begin_render_pass(label, vec![color_attachment(texture)], None);

        assert_eq!(
            device.submit(command_list).unwrap_err(),
            RhiError::InvalidRenderPass {
                reason: "color attachment 0 must use a 2D, 2D array, or cube texture".to_string(),
            }
        );
    }
}
