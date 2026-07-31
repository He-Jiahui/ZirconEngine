use super::*;
use crate::graphics::backend::RenderBackend;

#[test]
fn realtime_capture_and_downsample_shaders_parse() {
    naga::front::wgsl::parse_str(CAPTURE_WGSL).expect("capture WGSL");
    naga::front::wgsl::parse_str(DOWNSAMPLE_WGSL).expect("downsample WGSL");
    assert!(CAPTURE_WGSL.contains("vec3<f32>(1.0, -uv.y, -uv.x)"));
    assert!(CAPTURE_WGSL.contains("dot(direction, params.sun_direction.xyz)"));
    assert!(!CAPTURE_WGSL.contains("length(params.sun_direction.xyz)"));
    assert!(!CAPTURE_WGSL.contains("cos("));
    assert!(!CAPTURE_WGSL.contains("rotate_y"));
    assert!(!CAPTURE_WGSL.contains("max(params.intensity"));
    assert!(DOWNSAMPLE_WGSL.contains("vec3<f32>(-uv.x, -uv.y, -1.0)"));
}

#[test]
fn capture_uniform_carries_directional_sun_without_final_sampling_parameters() {
    let mut params = ProceduralSkyParams::default_gradient();
    params.sun_direction = crate::core::math::Vec4::new(0.25, 0.5, 0.75, 0.0);
    params.sun_color = crate::core::math::Vec4::new(1.0, 0.8, 0.6, 1.0);
    params.sun_intensity = 12.0;
    params.sun_angular_radius_radians = 0.08;
    params.intensity = 4.0;
    params.rotation_radians = 1.5;

    let bytes: [u8; 112] = capture_params_bytes(&params, 128, 3);
    assert_eq!(bytes.len(), 112);
    let words = bytes
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
        .collect::<Vec<_>>();
    let normalized = params.sun_direction.truncate().normalize();
    assert_close(f32::from_bits(words[12]), normalized.x);
    assert_close(f32::from_bits(words[13]), normalized.y);
    assert_close(f32::from_bits(words[14]), normalized.z);
    assert_eq!(f32::from_bits(words[15]), 1.0);
    assert_eq!(f32::from_bits(words[20]), params.sun_intensity);
    assert_close(
        f32::from_bits(words[21]),
        params.sun_angular_radius_radians.cos(),
    );
    assert_close(
        f32::from_bits(words[22]),
        (params.sun_angular_radius_radians * 0.72).cos(),
    );
    assert_eq!(words[24], 128);
    assert_eq!(words[25], 3);
    assert!(!words.contains(&params.intensity.to_bits()));
    assert!(!words.contains(&params.rotation_radians.to_bits()));
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0e-6,
        "{actual} != {expected}"
    );
}

#[test]
fn capture_and_downsample_bindings_pass_wgpu_validation() {
    let Ok(RenderBackend { device, queue, .. }) = RenderBackend::new_offscreen() else {
        return;
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-realtime-ibl-kernel-test"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 6,
        },
        mip_level_count: 5,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let mip0_storage = texture.create_view(&storage_view(0));
    let mip0_sampled = texture.create_view(&sampled_view(0));
    let mip1_storage = texture.create_view(&storage_view(1));
    let pipelines = RealtimeIblCaptureWgpuPipelines::new(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-realtime-ibl-kernel-test"),
    });
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    pipelines.record_capture(
        &device,
        &mut encoder,
        &ProceduralSkyParams::default_gradient(),
        16,
        CubeFaceRange::ALL,
        &mip0_storage,
    );
    pipelines.record_downsample_mip(&device, &mut encoder, 16, 8, &mip0_sampled, &mip1_storage);
    queue.submit([encoder.finish()]);

    let validation_error = pollster::block_on(error_scope.pop());
    assert!(validation_error.is_none(), "{validation_error:?}");
}

fn sampled_view(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-realtime-ibl-test-sampled"),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}

fn storage_view(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-realtime-ibl-test-storage"),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        usage: Some(wgpu::TextureUsages::STORAGE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}
