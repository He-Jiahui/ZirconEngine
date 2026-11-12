//! Behavioral adaptations of the local Unreal RHITests, rooted at
//! E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Tests/RHITests/Source/RHITests/:
//! - Public/RHIBufferTests.h: FRHIBufferTests::Test_RHICreateBuffer_Buffer (409-499),
//!   with Private/RHIBufferTests.cpp: VerifyBufferContents (68-107).
//! - Public/RHITextureTests.h: FRHITextureTests::Test_RHICopyTexture (876-1023),
//!   specifically the whole-texture case and the four CopyTextureSrgbLambda
//!   UNorm/sRGB pairings, whose destination stays bitwise identical to the source.
//!
//! R07/F02: exercise resource creation and recording through the existing neutral
//! interface and the transitional native recorder, with the same device owner
//! submitting and delivering the bytes. These are not architecture-convergence
//! assertions or render-thread/graph tests. Unreal's zero-byte null buffer is
//! rejected by the neutral descriptor contract and its one-byte buffer cannot use
//! the four-byte-aligned readback, so both stay outside this subset, as do the
//! mip and slice copies.

use std::time::{Duration, Instant};

use zr_rhi::{
    BufferDesc, BufferUsage, CommandList, DiagnosticReadbackAdmission, DiagnosticReadbackTerminal,
    RenderDevice, RenderQueueClass, RhiError, SubmissionStatus, SubmissionTicket,
    TextureCopyRegion, TextureDesc, TextureFormat, TextureUsage,
};

use super::super::WgpuRenderDevice;
use super::{production_test_device, wait_for_submission};

#[test]
fn unreal_create_buffer_preserves_zero_and_initialized_bytes_through_neutral_rhi() {
    let Some(device) = production_test_device() else {
        eprintln!("SKIP Unreal buffer contract: production GPU context unavailable");
        return;
    };

    for size in [4_usize, 2048] {
        for initialized in [false, true] {
            let expected = buffer_pattern(size, initialized);
            let buffer = device
                .create_buffer(&BufferDesc::new(
                    "unreal-neutral-buffer",
                    size as u64,
                    BufferUsage::VERTEX | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
                ))
                .unwrap();
            if initialized {
                // ZE uses its existing ticketed upload rather than UE's initializer API.
                let upload = device.write_buffer(buffer, 0, &expected).unwrap();
                assert_eq!(
                    device.submission_status(upload).unwrap(),
                    SubmissionStatus::Accepted
                );
            }
            device.begin_diagnostic_readback_frame(1).unwrap();
            let admission = device
                .enqueue_diagnostic_buffer_readback(buffer, 0, size as u64)
                .unwrap();
            let frame = device
                .submit_diagnostic_readback_frame("unreal-neutral-buffer-readback")
                .unwrap()
                .expect("one buffer readback must produce a frame");

            assert_submitted_bytes(&device, admission, frame.submission(), &expected);
            device.destroy_buffer(buffer).unwrap();
        }
    }
}

#[test]
fn unreal_create_buffer_preserves_zero_and_initialized_bytes_through_native_recorder() {
    let Some(device) = production_test_device() else {
        eprintln!("SKIP Unreal native buffer contract: production GPU context unavailable");
        return;
    };

    for size in [4_usize, 2048] {
        for initialized in [false, true] {
            let expected = buffer_pattern(size, initialized);
            device.begin_diagnostic_readback_frame(1).unwrap();
            let mut admission = None;
            let mut diagnostic_frame = None;
            let mut recording = device
                .begin_native_recording(RenderQueueClass::Copy)
                .unwrap();
            recording
                .record_command_buffer("unreal-native-buffer", |native, encoder| {
                    let buffer = native.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("unreal-native-buffer"),
                        size: size as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: initialized,
                    });
                    if initialized {
                        buffer
                            .slice(..)
                            .get_mapped_range_mut()
                            .copy_from_slice(&expected);
                        buffer.unmap();
                    }
                    admission = Some(device.enqueue_native_diagnostic_buffer_readback(
                        &buffer,
                        0,
                        size as u64,
                    )?);
                    diagnostic_frame = device.prepare_native_diagnostic_readback_frame(
                        "unreal-native-buffer-readback",
                        encoder,
                    )?;
                    // The local resource reference drops before enqueue; recorded work and
                    // the existing diagnostic batch must retain what completion needs.
                    Ok::<(), RhiError>(())
                })
                .unwrap();
            let ticket = device
                .enqueue_native_recording_packet_with_diagnostics(
                    recording.finish().unwrap(),
                    Some(diagnostic_frame.expect("one native buffer readback frame")),
                )
                .unwrap();

            assert_submitted_bytes(&device, admission.unwrap(), ticket, &expected);
        }
    }
}

#[test]
fn unreal_copy_texture_preserves_unorm_and_srgb_bytes_through_neutral_rhi() {
    let Some(device) = production_test_device() else {
        eprintln!("SKIP Unreal texture contract: production GPU context unavailable");
        return;
    };
    let expected = texture_pattern();
    let region = TextureCopyRegion::new(16, 16);

    // UE's sRGB cases use PF_B8G8R8A8; the RGBA8 pairs keep the whole-copy coverage.
    for (source_format, destination_format) in [
        (TextureFormat::Rgba8Unorm, TextureFormat::Rgba8Unorm),
        (TextureFormat::Rgba8UnormSrgb, TextureFormat::Rgba8UnormSrgb),
        (TextureFormat::Bgra8Unorm, TextureFormat::Bgra8Unorm),
        (TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb),
        (TextureFormat::Bgra8UnormSrgb, TextureFormat::Bgra8Unorm),
        (TextureFormat::Bgra8UnormSrgb, TextureFormat::Bgra8UnormSrgb),
    ] {
        let source = device
            .create_texture(&TextureDesc::new(
                "unreal-neutral-copy-source",
                16,
                16,
                source_format,
                TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
            ))
            .unwrap();
        let destination = device
            .create_texture(&TextureDesc::new(
                "unreal-neutral-copy-destination",
                16,
                16,
                destination_format,
                TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
            ))
            .unwrap();
        device
            .write_texture(source, region, 16 * 4, &expected)
            .unwrap();
        let mut copy = device
            .create_command_list(RenderQueueClass::Copy, "unreal-neutral-texture-copy")
            .unwrap();
        copy.copy_texture_to_texture(source, destination, region, region);
        let copy_ticket = device.enqueue_command_list(copy).unwrap();
        assert_eq!(
            device.submission_status(copy_ticket).unwrap(),
            SubmissionStatus::Accepted
        );
        device.begin_diagnostic_readback_frame(1).unwrap();
        let admission = device
            .enqueue_diagnostic_texture_readback(destination, region)
            .unwrap();
        let frame = device
            .submit_diagnostic_readback_frame("unreal-neutral-texture-readback")
            .unwrap()
            .expect("one texture readback must produce a frame");

        // Upload, copy and readback are ordered by the existing device timeline.
        assert_submitted_bytes(&device, admission, frame.submission(), &expected);
        wait_for_submission(&device, copy_ticket);
        device.destroy_texture(source).unwrap();
        device.destroy_texture(destination).unwrap();
    }
}

#[test]
fn unreal_copy_texture_preserves_unorm_and_srgb_bytes_through_native_recorder() {
    let Some(device) = production_test_device() else {
        eprintln!("SKIP Unreal native texture contract: production GPU context unavailable");
        return;
    };
    let expected = texture_pattern();
    let mut padded = vec![0; 16 * 256];
    for row in 0..16 {
        padded[row * 256..row * 256 + 64].copy_from_slice(&expected[row * 64..(row + 1) * 64]);
    }

    // The public native readback bridge admits RGBA8 only, so the four UE sRGB
    // pairings run on RGBA8 here; the neutral test covers UE's BGRA8 layout.
    for (source_format, destination_format) in [
        (
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ),
        (
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ),
        (
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
        ),
        (
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ),
    ] {
        device.begin_diagnostic_readback_frame(1).unwrap();
        let mut admission = None;
        let mut diagnostic_frame = None;
        let mut recording = device
            .begin_native_recording(RenderQueueClass::Copy)
            .unwrap();
        recording
            .record_command_buffer("unreal-native-texture-copy", |native, encoder| {
                let extent = wgpu::Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: 1,
                };
                let texture_desc = |format: wgpu::TextureFormat| wgpu::TextureDescriptor {
                    label: Some("unreal-native-copy-texture"),
                    size: extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                };
                let source = native.create_texture(&texture_desc(source_format));
                let destination = native.create_texture(&texture_desc(destination_format));
                let upload = native.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("unreal-native-texture-upload"),
                    size: padded.len() as u64,
                    usage: wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                });
                upload
                    .slice(..)
                    .get_mapped_range_mut()
                    .copy_from_slice(&padded);
                upload.unmap();
                encoder.copy_buffer_to_texture(
                    wgpu::TexelCopyBufferInfo {
                        buffer: &upload,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(256),
                            rows_per_image: Some(16),
                        },
                    },
                    source.as_image_copy(),
                    extent,
                );
                encoder.copy_texture_to_texture(
                    source.as_image_copy(),
                    destination.as_image_copy(),
                    extent,
                );
                admission = Some(device.enqueue_native_diagnostic_texture_rgba8_readback(
                    &destination,
                    16,
                    16,
                )?);
                diagnostic_frame = device.prepare_native_diagnostic_readback_frame(
                    "unreal-native-texture-readback",
                    encoder,
                )?;
                Ok::<(), RhiError>(())
            })
            .unwrap();
        let ticket = device
            .enqueue_native_recording_packet_with_diagnostics(
                recording.finish().unwrap(),
                Some(diagnostic_frame.expect("one native texture readback frame")),
            )
            .unwrap();

        assert_submitted_bytes(&device, admission.unwrap(), ticket, &expected);
    }
}

fn buffer_pattern(size: usize, initialized: bool) -> Vec<u8> {
    (0..size)
        .map(|index| if initialized { index as u8 } else { 0 })
        .collect()
}

fn texture_pattern() -> Vec<u8> {
    // UE's 16x16 whole-copy case writes consecutive 32-bit values starting at 1.
    (1_u32..=256).flat_map(u32::to_le_bytes).collect()
}

fn assert_submitted_bytes(
    device: &WgpuRenderDevice,
    admission: DiagnosticReadbackAdmission,
    ticket: SubmissionTicket,
    expected: &[u8],
) {
    let request = match admission {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("Unreal-equivalent readback unexpectedly rejected: {receipt:?}")
        }
    };
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    // Do not prescribe a physical submit count; the owner may batch logical work.
    device.flush_submissions().unwrap();
    wait_for_submission(device, ticket);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        device.poll_submissions().unwrap();
        if let Some(delivery) = device.take_diagnostic_readback_delivery() {
            assert_eq!(delivery.receipt().request(), request);
            assert_eq!(delivery.receipt().frame_key().unwrap().submission(), ticket);
            assert_eq!(
                delivery.receipt().terminal(),
                DiagnosticReadbackTerminal::Succeeded
            );
            assert_eq!(delivery.bytes(), Some(expected));
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Unreal-equivalent GPU readback timed out"
        );
        std::thread::yield_now();
    }
}
