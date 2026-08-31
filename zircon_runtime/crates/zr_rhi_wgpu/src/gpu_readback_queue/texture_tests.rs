use std::sync::{Arc, Mutex};

use super::{tests::offscreen_test_device, GpuReadbackQueue, ReadbackError};

#[test]
fn r32_uint_texel_readback_returns_only_the_requested_coordinate() {
    let Some((device, submission_queue)) = offscreen_test_device() else {
        return;
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-readback-queue-r32-uint-source"),
        size: wgpu::Extent3d {
            width: 3,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let source_tokens = [3_u32, 5, 7, 11, 13, 17];
    submission_queue.write_texture(
        texture.as_image_copy(),
        bytemuck::cast_slice(&source_tokens),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(3 * size_of::<u32>() as u32),
            rows_per_image: Some(2),
        },
        texture.size(),
    );

    let delivered = Arc::new(Mutex::new(None));
    let callback_delivered = Arc::clone(&delivered);
    let mut readback_queue = GpuReadbackQueue::new(&device);
    readback_queue.prepare_frame(31).unwrap();
    readback_queue
        .request_texture_r32_uint_texel(
            "test-r32-uint-texel",
            &texture,
            [2, 1],
            Box::new(move |result| {
                *callback_delivered.lock().unwrap() = Some(result.unwrap());
            }),
        )
        .unwrap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-queue-r32-uint-encoder"),
    });
    assert_eq!(readback_queue.encode_copies(&mut encoder, 31).unwrap(), 256);
    submission_queue.submit([encoder.finish()]);
    readback_queue.begin_map(31).unwrap();

    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    readback_queue.poll_completed();

    assert_eq!(*delivered.lock().unwrap(), Some(17));
}

#[test]
fn r32_uint_texel_readback_rejects_wrong_format_and_out_of_bounds_pixel() {
    let Some((device, _submission_queue)) = offscreen_test_device() else {
        return;
    };
    let wrong_format = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-readback-queue-wrong-identity-format"),
        size: wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let identity = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-readback-queue-bounded-identity-source"),
        size: wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let mut readback_queue = GpuReadbackQueue::new(&device);
    readback_queue.prepare_frame(32).unwrap();

    assert!(matches!(
        readback_queue.request_texture_r32_uint_texel(
            "wrong-format",
            &wrong_format,
            [0, 0],
            Box::new(|_| {}),
        ),
        Err(ReadbackError::TextureFormatMismatch {
            expected: wgpu::TextureFormat::R32Uint,
            actual: wgpu::TextureFormat::Rgba8Unorm,
        })
    ));
    assert!(matches!(
        readback_queue.request_texture_r32_uint_texel(
            "out-of-bounds",
            &identity,
            [2, 1],
            Box::new(|_| {}),
        ),
        Err(ReadbackError::TextureCoordinateOutOfBounds {
            x: 2,
            y: 1,
            width: 2,
            height: 2,
        })
    ));
}
