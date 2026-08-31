use std::time::Duration;

use zr_rhi::{
    BufferDesc, BufferUsage, CommandList, DiagnosticReadbackAdmission, DiagnosticReadbackKind,
    DiagnosticReadbackTerminal, RenderDevice, RenderQueueClass, RhiError, TextureCopyAspect,
    TextureCopyRegion, TextureDesc, TextureFormat, TextureUsage,
};

use super::super::{WgpuDiagnosticReadbackDelivery, WgpuRenderDevice};
use super::{production_test_device, wait_for_submission};

#[test]
fn production_texture_copies_preserve_padded_rows_and_async_readback_payloads() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(&TextureDesc::new(
            "production-diagnostic-texture-source",
            3,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
        ))
        .unwrap();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "production-diagnostic-texture-upload",
            512,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let copied = device
        .create_buffer(&BufferDesc::new(
            "production-diagnostic-texture-copied",
            512,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let roundtrip_texture = device
        .create_texture(&TextureDesc::new(
            "production-diagnostic-texture-roundtrip",
            3,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
        ))
        .unwrap();
    let expected = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ];
    let mut expected_padded = vec![0; 512];
    expected_padded[..12].copy_from_slice(&expected[..12]);
    expected_padded[256..268].copy_from_slice(&expected[12..]);
    let upload_ticket = device.write_buffer(upload, 0, &expected_padded).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, upload_ticket);

    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "production-texture-copy-roundtrip")
        .unwrap();
    copy.copy_buffer_to_texture(upload, texture, 0, 256, TextureCopyRegion::new(3, 2));
    copy.copy_texture_to_buffer(texture, copied, 0, 256, TextureCopyRegion::new(3, 2));
    copy.copy_buffer_to_texture(
        copied,
        roundtrip_texture,
        0,
        256,
        TextureCopyRegion::new(3, 2),
    );
    let copy_ticket = device.enqueue_command_list(copy).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, copy_ticket);

    device.begin_diagnostic_readback_frame(43).unwrap();
    let texture_request = match device
        .enqueue_diagnostic_texture_readback(roundtrip_texture, TextureCopyRegion::new(3, 2))
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("texture readback request unexpectedly rejected: {receipt:?}")
        }
    };
    let texture_frame = device
        .submit_diagnostic_readback_frame("production-diagnostic-texture-map")
        .unwrap()
        .expect("one admitted texture request must produce a submission-qualified frame");
    assert_eq!(device.flush_submissions().unwrap(), 1);
    let texture_delivery = wait_for_diagnostic_delivery(&device);
    assert_eq!(texture_delivery.receipt().request(), texture_request);
    assert_eq!(
        texture_delivery.receipt().kind(),
        DiagnosticReadbackKind::Texture
    );
    assert_eq!(texture_delivery.receipt().frame_key(), Some(texture_frame));
    assert_eq!(
        texture_delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Succeeded
    );
    assert_eq!(texture_delivery.bytes(), Some(&expected[..]));
}

#[test]
fn production_texture_to_texture_copy_preserves_color_subresource_bytes() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_texture(&TextureDesc::new(
            "production-texture-to-texture-source",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
        ))
        .unwrap();
    let destination = device
        .create_texture(&TextureDesc::new(
            "production-texture-to-texture-destination",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
        ))
        .unwrap();
    let source_pixels: Vec<u8> = (0_u8..16)
        .flat_map(|value| [value, value.wrapping_add(1), value.wrapping_add(2), 255])
        .collect();
    let source_region = TextureCopyRegion::new(2, 2).with_origin(1, 1, 0);
    let destination_region = TextureCopyRegion::new(2, 2).with_origin(0, 2, 0);

    let upload = device
        .write_texture(source, TextureCopyRegion::new(4, 4), 16, &source_pixels)
        .unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, upload);

    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "production-texture-to-texture-copy")
        .unwrap();
    copy.copy_texture_to_texture(source, destination, source_region, destination_region);
    let copy_ticket = device.enqueue_command_list(copy).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, copy_ticket);

    device.begin_diagnostic_readback_frame(45).unwrap();
    let request = match device
        .enqueue_diagnostic_texture_readback(destination, destination_region)
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("texture-to-texture readback unexpectedly rejected: {receipt:?}")
        }
    };
    let frame = device
        .submit_diagnostic_readback_frame("production-texture-to-texture-map")
        .unwrap()
        .expect("one admitted texture request must produce a submission-qualified frame");
    assert_eq!(device.flush_submissions().unwrap(), 1);
    let delivery = wait_for_diagnostic_delivery(&device);
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(frame));
    assert_eq!(
        delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Succeeded
    );
    assert_eq!(
        delivery.bytes(),
        Some(&[5, 6, 7, 255, 6, 7, 8, 255, 9, 10, 11, 255, 10, 11, 12, 255][..])
    );
}

#[test]
fn production_texture_upload_accepts_tight_rows_and_delivers_one_copy_ticket() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(&TextureDesc::new(
            "production-tight-texture-upload",
            3,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
        ))
        .unwrap();
    let expected = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ];

    let upload = device
        .write_texture(texture, TextureCopyRegion::new(3, 2), 12, &expected)
        .unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, upload);

    device.begin_diagnostic_readback_frame(44).unwrap();
    let request = match device
        .enqueue_diagnostic_texture_readback(texture, TextureCopyRegion::new(3, 2))
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("texture upload readback unexpectedly rejected: {receipt:?}")
        }
    };
    let frame = device
        .submit_diagnostic_readback_frame("production-tight-texture-upload-map")
        .unwrap()
        .expect("one admitted texture request must produce a submission-qualified frame");
    assert_eq!(device.flush_submissions().unwrap(), 1);
    let delivery = wait_for_diagnostic_delivery(&device);
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(frame));
    assert_eq!(
        delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Succeeded
    );
    assert_eq!(delivery.bytes(), Some(&expected[..]));
}

#[test]
fn production_texture_copy_rejects_unpadded_multiline_rows_before_native_encoding() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_buffer(&BufferDesc::new(
            "production-texture-copy-tight-rows",
            24,
            BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_texture(&TextureDesc::new(
            "production-texture-copy-tight-rows-target",
            3,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST,
        ))
        .unwrap();
    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "production-texture-copy-tight-rows")
        .unwrap();
    copy.copy_buffer_to_texture(source, destination, 0, 12, TextureCopyRegion::new(3, 2));

    assert!(matches!(
        device.enqueue_command_list(copy),
        Err(RhiError::BufferToTextureCopyOutOfRange { .. })
    ));
    assert_eq!(device.first_fault(), None);
}

#[test]
fn production_texture_copy_rejects_depth_stencil_without_a_neutral_aspect() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_buffer(&BufferDesc::new(
            "production-depth-copy-source",
            256,
            BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_texture(&TextureDesc::new(
            "production-depth-copy-target",
            1,
            1,
            TextureFormat::Depth24PlusStencil8,
            TextureUsage::COPY_DST | TextureUsage::RENDER_ATTACHMENT,
        ))
        .unwrap();
    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "production-depth-copy")
        .unwrap();
    copy.copy_buffer_to_texture(source, destination, 0, 4, TextureCopyRegion::new(1, 1));

    assert!(matches!(
        device.enqueue_command_list(copy),
        Err(RhiError::InvalidCopy { .. })
    ));
    assert_eq!(device.first_fault(), None);
}

#[test]
fn production_depth32_texture_to_buffer_uses_the_explicit_depth_aspect() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_texture(&TextureDesc::new(
            "production-depth32-copy-source",
            1,
            1,
            TextureFormat::Depth32Float,
            TextureUsage::COPY_SRC | TextureUsage::RENDER_ATTACHMENT,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "production-depth32-copy-destination",
            256,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let region = TextureCopyRegion::new(1, 1).with_aspect(TextureCopyAspect::DepthOnly);

    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "production-depth32-to-buffer")
        .unwrap();
    copy.copy_texture_to_buffer(source, destination, 0, 256, region);
    let ticket = device.enqueue_command_list(copy).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, ticket);
    assert_eq!(device.first_fault(), None);
}

fn wait_for_diagnostic_delivery(device: &WgpuRenderDevice) -> WgpuDiagnosticReadbackDelivery {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        device.poll_submissions().unwrap();
        if let Some(delivery) = device.take_diagnostic_readback_delivery() {
            return delivery;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "diagnostic map timed out"
        );
        std::thread::yield_now();
    }
}
