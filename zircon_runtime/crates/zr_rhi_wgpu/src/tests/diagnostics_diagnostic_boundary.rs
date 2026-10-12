use super::*;
use zr_rhi::{DeviceGeneration, DeviceId, RenderResourceHandleAllocator, TextureFormat};

#[test]
fn diagnostic_boundary_texture_row_narrowing_returns_invalid_copy() {
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let texture = handles.allocate_texture().unwrap();
    let desc = TextureDesc::new(
        "diagnostic row narrowing",
        u32::MAX,
        1,
        TextureFormat::Rgba8Unorm,
        TextureUsage::COPY_SRC,
    );

    assert!(matches!(
        ensure_diagnostic_texture_readback_region(
            texture,
            &desc,
            TextureCopyRegion::new(u32::MAX, 1),
        ),
        Err(RhiError::InvalidCopy { reason }) if reason.contains("staging layout overflowed")
    ));
}

#[test]
fn diagnostic_boundary_depth_direct_copy_returns_invalid_copy() {
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let texture = handles.allocate_texture().unwrap();
    let desc = TextureDesc::new(
        "diagnostic depth",
        1,
        1,
        TextureFormat::Depth32Float,
        TextureUsage::COPY_SRC,
    );

    assert!(matches!(
        ensure_diagnostic_texture_readback_region(texture, &desc, TextureCopyRegion::new(1, 1)),
        Err(RhiError::InvalidCopy { reason }) if reason.contains("depth/stencil conversion")
    ));
}

#[test]
fn diagnostic_boundary_buffer_invalid_ranges_return_typed_errors() {
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let buffer = handles.allocate_buffer().unwrap();
    let desc = BufferDesc::new("diagnostic buffer", 16, BufferUsage::COPY_SRC);

    assert!(matches!(
        ensure_diagnostic_readback_range(buffer, &desc, 0, 0),
        Err(RhiError::InvalidCopy { .. })
    ));
    assert!(matches!(
        ensure_diagnostic_readback_range(buffer, &desc, 12, 8),
        Err(RhiError::ReadbackOutOfRange { .. })
    ));
}

#[test]
fn normal_color8_budget_checks_padded_staging_not_tight_payload() {
    assert!(u64::from(4097_u32) * 4 * 1515 < 24 * 1024 * 1024);
    assert!(
        matches!(normal_color8_staging_layout(4097,1515),Err(RhiError::InvalidCopy { reason }) if reason.contains("padded staging"))
    );
    assert!(normal_color8_staging_layout(1920, 1200).is_ok());
    assert!(normal_color8_staging_layout(0, 1200).is_err());
}

#[test]
fn normal_color8_readback_preserves_actual_format_and_the_paint_submission_ticket() {
    let Some((context, profile)) = crate::production::tests::production_test_context_with_policy(
        zr_rhi::GpuMemoryBudget::reference_1080p_mid(),
        &zr_rhi::RenderDeviceRequestPolicy::mvp_baseline(),
    ) else {
        return;
    };
    let gpu = context.device.clone();
    let device = WgpuRenderDevice::new(context, profile).unwrap();
    for (index, format) in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    ]
    .into_iter()
    .enumerate()
    {
        let source = gpu.create_texture(&wgpu::TextureDescriptor {
            label: Some("normal-color8-real-owner-test"),
            size: wgpu::Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = source.create_view(&Default::default());
        let mut encoder = gpu.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("actual-color8-paint"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.25,
                            g: 0.5,
                            b: 0.75,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        device
            .begin_diagnostic_readback_frame(900 + index as u64)
            .unwrap();
        assert!(device
            .enqueue_native_diagnostic_texture_color8_readback(&source, 1, 2)
            .is_err());
        let DiagnosticReadbackAdmission::Admitted(request) = device
            .enqueue_native_diagnostic_texture_color8_readback(&source, 2, 2)
            .unwrap()
        else {
            panic!("small actual copy must be admitted");
        };
        let diagnostic = device
            .prepare_native_diagnostic_readback_frame("same-paint-encoder", &mut encoder)
            .unwrap()
            .unwrap();
        let mut recording = device
            .begin_native_recording(RenderQueueClass::Graphics)
            .unwrap();
        recording.extend_recorded_command_buffers([encoder.finish()]);
        let ticket = device
            .submit_native_recording_packet_with_diagnostics(
                recording.finish().unwrap(),
                Some(diagnostic),
            )
            .unwrap();
        crate::production::tests::wait_for_submission(&device, ticket);
        device.poll_submissions().unwrap();
        let delivery = device
            .take_diagnostic_readback_delivery_for(request)
            .expect("existing owner must deliver the actual copied bytes");
        assert_eq!(
            delivery.receipt().terminal(),
            DiagnosticReadbackTerminal::Succeeded
        );
        assert_eq!(delivery.receipt().frame_key().unwrap().submission(), ticket);
        let bytes = delivery.into_bytes().unwrap();
        assert_eq!(
            bytes.len(),
            16,
            "row padding is removed by the existing delivery owner"
        );
        assert_eq!(bytes[3], 255);
        if matches!(
            format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        ) {
            assert!(bytes[0] > bytes[2]);
        } else {
            assert!(bytes[0] < bytes[2]);
        }
    }
}
