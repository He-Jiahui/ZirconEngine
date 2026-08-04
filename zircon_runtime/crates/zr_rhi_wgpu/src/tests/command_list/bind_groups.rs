use super::*;

#[test]
fn command_list_records_bind_groups_and_submit_validates_raster_pipeline_layout() {
    let device = DeterministicRhiContractDevice::new_headless();
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
            zr_rhi::CommandListCommand::BeginRenderPass {
                label: "test-render-pass".to_string(),
                color_attachments: vec![color_attachment(color)],
                depth_stencil_attachment: Some(depth_attachment(depth)),
            },
            zr_rhi::CommandListCommand::SetPipeline { pipeline },
            zr_rhi::CommandListCommand::SetBindGroup {
                slot: 0,
                bind_group,
            },
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
    let device = DeterministicRhiContractDevice::new_headless();
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
    let device = DeterministicRhiContractDevice::new_headless();
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
