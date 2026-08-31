use zr_rhi::{
    BufferDesc, BufferUsage, DiagnosticReadbackAdmission, DiagnosticReadbackTerminal, IndexFormat,
    PipelineDesc, PipelineKind, PipelineLayoutDesc, RasterPipelineStateDesc, RenderClearColor,
    RenderDevice, RenderOperation, RenderOperationSupport, RenderPassColorAttachmentDesc,
    RenderPassColorLoadOp, RenderPassStoreOp, RenderPassTextureViewDesc, RenderQueueClass,
    RhiError, ShaderModuleDesc, ShaderStage, SurfaceAcquireOutcome, SurfaceFrameTerminal,
    SurfaceSessionCreateOutcome, TextureCopyRegion, TextureDesc, TextureDimension, TextureFormat,
    TextureUsage, TextureViewDesc, TextureViewDimension,
};

use super::super::{
    WgpuDiagnosticReadbackDelivery, WgpuMvpOffscreenTriangle, WgpuMvpSurfaceTriangle,
    WgpuRenderDevice,
};
use super::{production_test_device, wait_for_submission};

fn deterministic_surface_descriptor(width: u32, height: u32) -> zr_rhi::RenderSurfaceDescriptor {
    zr_rhi::RenderSurfaceDescriptor::new(
        "production-mvp-surface",
        zr_rhi::RenderNativeSurfaceTarget::Win32 {
            hwnd: 1,
            hinstance: None,
        },
        zr_rhi::SwapchainDesc {
            width,
            height,
            present_mode: zr_rhi::PresentMode::Fifo,
            format: TextureFormat::Bgra8UnormSrgb,
        },
    )
}

#[test]
fn production_mvp_surface_triangle_submits_the_acquired_target_then_presents_it() {
    let device = crate::DeterministicRhiContractDevice::new_headless();
    let SurfaceSessionCreateOutcome::Renderable(surface) = device
        .create_surface_session(&deterministic_surface_descriptor(64, 64))
        .unwrap()
    else {
        panic!("nonzero deterministic surface must be renderable");
    };
    let SurfaceAcquireOutcome::Acquired(frame) =
        device.acquire_surface_frame(surface.session()).unwrap()
    else {
        panic!("deterministic surface must acquire one frame");
    };
    let target = frame.target();
    let renderer = WgpuMvpSurfaceTriangle::new_for_device(&device, &surface.swapchain).unwrap();

    let receipt = renderer
        .render_and_present_for_device(&device, frame)
        .unwrap();

    assert_eq!(receipt.terminal, SurfaceFrameTerminal::Presented);
    assert!(device.texture_desc(target).is_err());
    renderer.destroy_for_device(&device).unwrap();
}

#[test]
fn production_mvp_surface_triangle_rejects_foreign_device_before_recording_and_discards_lease() {
    let surface_device = crate::DeterministicRhiContractDevice::new_headless();
    let pipeline_device = crate::DeterministicRhiContractDevice::new_headless_with_identity(
        zr_rhi::DeviceId::new(2),
        zr_rhi::DeviceGeneration::initial(),
    );
    let SurfaceSessionCreateOutcome::Renderable(surface) = surface_device
        .create_surface_session(&deterministic_surface_descriptor(64, 64))
        .unwrap()
    else {
        panic!("nonzero deterministic surface must be renderable");
    };
    let SurfaceAcquireOutcome::Acquired(frame) = surface_device
        .acquire_surface_frame(surface.session())
        .unwrap()
    else {
        panic!("deterministic surface must acquire one frame");
    };
    let target = frame.target();
    let renderer =
        WgpuMvpSurfaceTriangle::new_for_device(&pipeline_device, &surface.swapchain).unwrap();

    assert_eq!(
        renderer
            .render_and_present_for_device(&surface_device, frame)
            .unwrap_err(),
        RhiError::SurfaceUnavailable(
            "direct surface triangle resources belong to another WGPU device generation"
                .to_string(),
        ),
    );
    assert!(surface_device.texture_desc(target).is_err());
    renderer.destroy_for_device(&pipeline_device).unwrap();
}

#[test]
fn production_mvp_triangle_owner_records_only_neutral_rhi_work() {
    let Some(device) = production_test_device() else {
        return;
    };
    let frame = WgpuMvpOffscreenTriangle::new(&device, 64, 64).unwrap();
    let depth = device.texture_desc(frame.depth_target()).unwrap();
    assert_eq!(depth.format, TextureFormat::Depth24Plus);
    assert_eq!(depth.usage, TextureUsage::RENDER_ATTACHMENT);
    let ticket = frame.submit(&device).unwrap();
    wait_for_submission(&device, ticket);

    device.begin_diagnostic_readback_frame(62).unwrap();
    let request = match device
        .enqueue_diagnostic_texture_readback(frame.target(), TextureCopyRegion::new(64, 64))
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("MVP triangle readback request unexpectedly rejected: {receipt:?}")
        }
    };
    let submitted = device
        .submit_diagnostic_readback_frame("production-mvp-triangle-readback")
        .unwrap()
        .expect("one MVP triangle readback request must create a diagnostic frame");
    assert_eq!(device.flush_submissions().unwrap(), 1);
    let delivery = wait_for_diagnostic_delivery(&device);
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(submitted));
    assert_eq!(
        delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Succeeded
    );
    let pixels = delivery
        .bytes()
        .expect("completed MVP triangle diagnostic readback must carry pixels");
    let center = ((32 * 64 + 32) * 4) as usize;
    assert_rgba8_unorm_near(&pixels[center..center + 4], [26, 204, 77, 255]);

    frame.destroy(&device).unwrap();
}

#[test]
fn production_owner_encodes_mip_and_array_layer_render_attachments() {
    let Some(device) = production_test_device() else {
        return;
    };
    assert_render_attachment_subresource(
        &device,
        "production-mipped-array-attachment",
        TextureDimension::D2Array,
        2,
        1,
        63,
    );
}

#[test]
fn production_owner_encodes_mip_and_cube_face_render_attachments() {
    let Some(device) = production_test_device() else {
        return;
    };
    assert_render_attachment_subresource(
        &device,
        "production-mipped-cube-face-attachment",
        TextureDimension::Cube,
        6,
        5,
        64,
    );
}

fn assert_render_attachment_subresource(
    device: &WgpuRenderDevice,
    label: &str,
    dimension: TextureDimension,
    array_layers: u32,
    array_layer: u32,
    diagnostic_frame_index: u64,
) {
    let target = device
        .create_texture(
            &TextureDesc::new(
                format!("{label}-target"),
                64,
                64,
                TextureFormat::Rgba8Unorm,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
            )
            .with_dimension(dimension)
            .with_array_layers(array_layers)
            .with_mip_levels(2),
        )
        .unwrap();
    let registered_view = device
        .create_texture_view(
            &TextureViewDesc::new(
                format!("{label}-registered-view"),
                target,
                TextureViewDimension::D2,
            )
            .with_mip_range(1, 1)
            .with_array_layer_range(array_layer, 1),
        )
        .unwrap();
    let view = RenderPassTextureViewDesc::new(target)
        .with_mip_level(1)
        .with_array_layer(array_layer)
        .with_registered_view(registered_view);
    let mut commands = device
        .create_command_list(RenderQueueClass::Graphics, label)
        .unwrap();
    commands.begin_render_pass(
        label,
        vec![RenderPassColorAttachmentDesc::new(
            target,
            RenderPassColorLoadOp::Clear(RenderClearColor::new(0.0, 1.0, 0.0, 1.0)),
            RenderPassStoreOp::Store,
        )
        .with_view(view)],
        None,
    );
    commands.end_render_pass();
    wait_for_submission(device, device.submit(commands).unwrap());

    device
        .begin_diagnostic_readback_frame(diagnostic_frame_index)
        .unwrap();
    let request = match device
        .enqueue_diagnostic_texture_readback(
            target,
            TextureCopyRegion::new(32, 32)
                .with_mip_level(1)
                .with_origin(0, 0, array_layer),
        )
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("mipped array attachment readback unexpectedly rejected: {receipt:?}")
        }
    };
    let frame = device
        .submit_diagnostic_readback_frame(&format!("{label}-readback"))
        .unwrap()
        .expect(
            "one render attachment subresource readback request must create a diagnostic frame",
        );
    assert_eq!(device.flush_submissions().unwrap(), 1);
    let delivery = wait_for_diagnostic_delivery(&device);
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(frame));
    let pixels = delivery
        .bytes()
        .expect("render attachment subresource readback must carry pixels");
    assert_eq!(&pixels[..4], &[0, 255, 0, 255]);

    device.destroy_texture_view(registered_view).unwrap();
    device.destroy_texture(target).unwrap();
}

#[test]
fn production_owner_encodes_native_copy_compute_and_offscreen_triangle_and_indexed_draw() {
    let Some(device) = production_test_device() else {
        return;
    };

    assert_eq!(
        device
            .caps()
            .operation_support(RenderOperation::BufferToBufferCopy),
        RenderOperationSupport::Native
    );
    assert_eq!(
        device
            .caps()
            .operation_support(RenderOperation::BufferToTextureCopy),
        RenderOperationSupport::Native
    );
    assert_eq!(
        device
            .caps()
            .operation_support(RenderOperation::TextureToBufferCopy),
        RenderOperationSupport::Native
    );
    assert_eq!(
        device
            .caps()
            .operation_support(RenderOperation::ComputeDispatch),
        RenderOperationSupport::Native
    );
    assert_eq!(
        device.caps().operation_support(RenderOperation::DirectDraw),
        RenderOperationSupport::Native
    );
    assert!(!device.caps().supports_buffer_readback);

    let source = device
        .create_buffer(&BufferDesc::new(
            "production-copy-source",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "production-copy-destination",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    device
        .write_buffer(
            source,
            0,
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        )
        .unwrap();
    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "production-copy")
        .unwrap();
    copy.push_debug_group("production-copy-submit");
    copy.push_debug_marker("production-copy-buffers");
    copy.copy_buffer_to_buffer(source, destination, 0, 0, 16);
    copy.pop_debug_group();
    wait_for_submission(&device, device.submit(copy).unwrap());
    assert!(matches!(
        device.read_buffer(destination, 0, 16),
        Err(RhiError::ReadbackUnavailable { .. })
    ));

    let compute_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "production-compute-layout",
            Vec::new(),
        ))
        .unwrap();
    let compute_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "production-compute-shader",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let compute_pipeline = device
        .create_pipeline(
            &PipelineDesc::new("production-compute", PipelineKind::Compute)
                .with_layout(compute_layout)
                .with_compute_shader(compute_shader),
        )
        .unwrap();
    let mut compute = device
        .create_command_list(RenderQueueClass::Compute, "production-compute")
        .unwrap();
    compute.begin_compute_pass("production-compute-pass");
    compute.set_pipeline(compute_pipeline);
    compute.dispatch_compute(1, 1, 1);
    compute.end_compute_pass();
    wait_for_submission(&device, device.submit(compute).unwrap());

    let target = device
        .create_texture(&TextureDesc::new(
            "production-triangle-target",
            64,
            64,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let triangle_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "production-triangle-layout",
            Vec::new(),
        ))
        .unwrap();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "production-triangle-vertex",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {\n  let positions = array<vec2<f32>, 3>(vec2<f32>(0.0, 0.75), vec2<f32>(-0.75, -0.75), vec2<f32>(0.75, -0.75));\n  return vec4<f32>(positions[index], 0.0, 1.0);\n}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "production-triangle-fragment",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.1, 0.8, 0.3, 1.0); }",
        ))
        .unwrap();
    let triangle_pipeline = device
        .create_pipeline(
            &PipelineDesc::new("production-triangle", PipelineKind::Raster)
                .with_layout(triangle_layout)
                .with_vertex_shader(vertex_shader)
                .with_fragment_shader(fragment_shader)
                .with_raster_state(RasterPipelineStateDesc::single_color(
                    TextureFormat::Rgba8Unorm,
                )),
        )
        .unwrap();
    let mut triangle = device
        .create_command_list(RenderQueueClass::Graphics, "production-triangle")
        .unwrap();
    triangle.push_debug_group("production-triangle-submit");
    triangle.begin_render_pass(
        "production-triangle-pass",
        vec![RenderPassColorAttachmentDesc::new(
            target,
            RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
            RenderPassStoreOp::Store,
        )],
        None,
    );
    triangle.push_debug_group("production-triangle-pass");
    triangle.push_debug_marker("production-draw-triangle");
    triangle.set_pipeline(triangle_pipeline);
    triangle.draw(0, 3, 0, 1);
    triangle.pop_debug_group();
    triangle.end_render_pass();
    triangle.pop_debug_group();
    wait_for_submission(&device, device.submit(triangle).unwrap());

    let index_buffer = device
        .create_buffer(&BufferDesc::new(
            "production-triangle-indices",
            8,
            BufferUsage::INDEX | BufferUsage::COPY_DST,
        ))
        .unwrap();
    device
        .write_buffer(index_buffer, 0, &[0, 0, 1, 0, 2, 0, 0, 0])
        .unwrap();
    let mut indexed_triangle = device
        .create_command_list(RenderQueueClass::Graphics, "production-indexed-triangle")
        .unwrap();
    indexed_triangle.begin_render_pass(
        "production-indexed-triangle-pass",
        vec![RenderPassColorAttachmentDesc::new(
            target,
            RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
            RenderPassStoreOp::Store,
        )],
        None,
    );
    indexed_triangle.set_pipeline(triangle_pipeline);
    indexed_triangle.set_index_buffer(index_buffer, 0, 6, IndexFormat::Uint16);
    indexed_triangle.draw_indexed(0, 3, 0, 0, 1);
    indexed_triangle.end_render_pass();
    wait_for_submission(&device, device.submit(indexed_triangle).unwrap());

    device.begin_diagnostic_readback_frame(61).unwrap();
    let request = match device
        .enqueue_diagnostic_texture_readback(target, TextureCopyRegion::new(64, 64))
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("triangle readback request unexpectedly rejected: {receipt:?}")
        }
    };
    let frame = device
        .submit_diagnostic_readback_frame("production-triangle-readback")
        .unwrap()
        .expect("one triangle readback request must create a diagnostic frame");
    assert_eq!(device.flush_submissions().unwrap(), 1);
    let delivery = wait_for_diagnostic_delivery(&device);
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(frame));
    assert_eq!(
        delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Succeeded
    );
    let pixels = delivery
        .bytes()
        .expect("completed triangle diagnostic readback must carry pixels");
    let center = ((32 * 64 + 32) * 4) as usize;
    assert_rgba8_unorm_near(&pixels[center..center + 4], [26, 204, 77, 255]);

    assert_eq!(device.first_fault(), None);
}

fn wait_for_diagnostic_delivery(device: &WgpuRenderDevice) -> WgpuDiagnosticReadbackDelivery {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        device.poll_submissions().unwrap();
        if let Some(delivery) = device.take_diagnostic_readback_delivery() {
            return delivery;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "triangle diagnostic map timed out"
        );
        std::thread::yield_now();
    }
}

fn assert_rgba8_unorm_near(actual: &[u8], expected: [u8; 4]) {
    assert_eq!(actual.len(), expected.len());
    for (channel, (&actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            actual.abs_diff(expected) <= 1,
            "RGBA8 channel {channel} differed by more than one UNORM quantization step: actual={actual}, expected={expected}"
        );
    }
}
