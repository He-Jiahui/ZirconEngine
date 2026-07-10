use super::*;

#[test]
fn trace_probe_tiles_shader_uses_surface_cache_hzb_to_skip_depth_disjoint_blocks() {
    let Some((device, queue)) = test_device() else {
        eprintln!(
            "skipping trace_probe_tiles surface-cache HZB Wgpu test because no adapter is available"
        );
        return;
    };
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        &device,
        1,
        0,
        1,
        ProbeTraceTileSurfaceCacheParams {
            texture_available: 1,
            atlas_width: 8,
            atlas_height: 1,
            atlas_columns: 8,
            tile_extent: 1,
        },
        0,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-hzb-test-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-hzb-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-hzb-test-tile-schedule",
        &[0_u32, 7, 0, 32],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(&device);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-hzb-test-atlas",
        8,
        1,
        &[
            [100, 40, 20, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [20, 200, 40, 255],
            [0, 0, 0, 255],
        ],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_hzb_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-hzb-test-depth",
        8,
        1,
        &[
            &[
                [128, 128, 0, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [132, 132, 0, 255],
                [220, 220, 0, 255],
            ],
            &[
                [128, 220, 92, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [132, 220, 88, 255],
            ],
            &[[128, 220, 92, 255], [132, 220, 88, 255]],
            &[[128, 220, 92, 255]],
        ],
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group(
        &device,
        &bind_group_layout,
        &params_buffer,
        &resident_probe_buffer,
        &pending_probe_buffer,
        &probe_trace_tile_buffer,
        &trace_lighting_buffer,
        &atlas_view,
        &depth_view,
        &scene_prepare_descriptor_buffer,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-hzb-test-encoder"),
    });
    encode_probe_trace_tile_dispatch(&mut encoder, &pipeline, &bind_group, 1);
    encoder.copy_buffer_to_buffer(
        &trace_lighting_buffer,
        0,
        &readback_buffer,
        0,
        (3 * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));

    let words = readback_u32s(&device, &readback_buffer, 3);
    assert_eq!(words[0], 1);
    assert_eq!(words[1], 7);
    assert_eq!(
        words[2],
        pack_rgb8([55, 70, 20]),
        "expected multi-direction HZB traces to skip depth-disjoint middle blocks and refine the far near-depth texel"
    );

    let shader_source = include_str!("../../../../../shaders/trace_probe_tiles.wgsl");
    assert!(shader_source.contains("textureNumLevels(surface_cache_depth)"));
    assert!(shader_source.contains("surface_cache_hzb_depth_range"));
}

#[test]
fn trace_probe_tiles_shader_keeps_negative_direction_hzb_skips_inside_the_tested_block() {
    let Some((device, queue)) = test_device() else {
        eprintln!(
            "skipping negative-direction surface-cache HZB Wgpu test because no adapter is available"
        );
        return;
    };
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        &device,
        1,
        0,
        1,
        ProbeTraceTileSurfaceCacheParams {
            texture_available: 1,
            atlas_width: 16,
            atlas_height: 1,
            atlas_columns: 8,
            tile_extent: 4,
        },
        0,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-negative-hzb-test-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-negative-hzb-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-negative-hzb-test-tile-schedule",
        &[0_u32, 7, 2, 32],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(&device);
    let mut atlas_pixels = [[0, 0, 0, 255]; 16];
    atlas_pixels[10] = [100, 40, 20, 255];
    atlas_pixels[7] = [20, 200, 40, 255];
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-negative-hzb-test-atlas",
        16,
        1,
        &atlas_pixels,
    );
    let mut depth_mip0 = [[220, 220, 0, 255]; 16];
    depth_mip0[10] = [128, 128, 0, 255];
    depth_mip0[7] = [132, 132, 0, 255];
    let (_depth_texture, depth_view) = create_test_surface_cache_hzb_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-negative-hzb-test-depth",
        16,
        1,
        &[
            &depth_mip0,
            &[
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
                [132, 220, 88, 255],
                [220, 220, 0, 255],
                [128, 220, 92, 255],
                [220, 220, 0, 255],
                [220, 220, 0, 255],
            ],
            &[
                [220, 220, 0, 255],
                [132, 220, 88, 255],
                [128, 220, 92, 255],
                [220, 220, 0, 255],
            ],
            &[[132, 220, 88, 255], [128, 220, 92, 255]],
            &[[128, 220, 92, 255]],
        ],
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group(
        &device,
        &bind_group_layout,
        &params_buffer,
        &resident_probe_buffer,
        &pending_probe_buffer,
        &probe_trace_tile_buffer,
        &trace_lighting_buffer,
        &atlas_view,
        &depth_view,
        &scene_prepare_descriptor_buffer,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-negative-hzb-test-encoder"),
    });
    encode_probe_trace_tile_dispatch(&mut encoder, &pipeline, &bind_group, 1);
    encoder.copy_buffer_to_buffer(
        &trace_lighting_buffer,
        0,
        &readback_buffer,
        0,
        (3 * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));

    let words = readback_u32s(&device, &readback_buffer, 3);
    assert_eq!(words[0], 1);
    assert_eq!(words[1], 7);
    assert_eq!(
        words[2],
        pack_rgb8([59, 62, 19]),
        "expected the multi-direction HZB trace to retain the negative ray that visits the near-depth texel inside the skipped distance range"
    );

    let shader_source = include_str!("../../../../../shaders/trace_probe_tiles.wgsl");
    assert!(shader_source.contains("surface_cache_hzb_mip_for_step_coord"));
}

fn create_test_surface_cache_hzb_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    width: u32,
    height: u32,
    mip_levels: &[&[[u8; 4]]],
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_levels.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    for (mip_level, pixels) in mip_levels.iter().enumerate() {
        let mip_width = (width >> mip_level).max(1);
        let mip_height = (height >> mip_level).max(1);
        assert_eq!(pixels.len(), (mip_width * mip_height) as usize);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: mip_level as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(*pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(mip_width * 4),
                rows_per_image: Some(mip_height),
            },
            wgpu::Extent3d {
                width: mip_width,
                height: mip_height,
                depth_or_array_layers: 1,
            },
        );
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-test-hzb-view"),
        ..Default::default()
    });
    (texture, view)
}
