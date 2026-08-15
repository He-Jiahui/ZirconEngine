use super::*;

#[test]
fn voxel_fallback_uses_fixed_lookup_and_three_dimensional_cell_distance() {
    let shader_source = include_str!("../../../../../shaders/trace_probe_tiles.wgsl");
    let voxel_source = include_str!("../../../../../shaders/trace_probe_tiles_voxel.wgsl");
    let bind_group_source = include_str!("../bind_group.rs");
    let dispatch_source = include_str!("../../dispatch_probe_trace_tiles.rs");

    assert!(shader_source.contains("const VOXEL_CELL_LOOKUP_MAX_CLIPMAPS: u32 = 8u;"));
    assert!(shader_source.contains("const VOXEL_CLIPMAP_CELL_COUNT: u32 = 64u;"));
    assert!(shader_source.contains("voxel_cell_lookup_clipmap_count: u32,"));
    assert!(voxel_source.contains("fn voxel_cell_manhattan_distance("));
    assert!(
        voxel_source.contains("for (var cell_index = 0u; cell_index < VOXEL_CLIPMAP_CELL_COUNT;")
    );
    assert!(!voxel_source
        .contains("for (var voxel_index = 0u; voxel_index < params.voxel_cell_descriptor_count;"));
    assert!(bind_group_source.contains("binding: 11"));
    assert!(dispatch_source.contains("voxel_cell_lookup_clipmap_count: u32,"));
    assert!(dispatch_source.contains("trace_probe_tiles_voxel.wgsl"));
}

#[test]
fn voxel_fallback_accepts_a_neighbor_across_the_z_plane() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping voxel lookup Wgpu test because no adapter is available");
        return;
    };
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        &device,
        1,
        0,
        1,
        ProbeTraceTileSurfaceCacheParams {
            texture_available: 1,
            atlas_width: 1,
            atlas_height: 1,
            atlas_columns: 1,
            tile_extent: 1,
        },
        2,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-tile-schedule",
        &[0_u32, 5, 42, 12],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-atlas",
        [0, 0, 0, 255],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-depth",
        [255, 255, 255, 0],
    );
    let scene_prepare_descriptor_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-descriptors",
        &[
            voxel_cell_descriptor_words_with_half_extent(5, 42, 4, [32, 64, 96], 96),
            voxel_cell_descriptor_words_with_half_extent(5, 58, 2, [96, 64, 32], 96),
        ],
    );
    let voxel_cell_lookup_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-lookup-z-neighbor-index",
        &voxel_cell_lookup_words(5, &[(42, 0), (58, 1)]),
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group_with_voxel_lookup(
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
        &voxel_cell_lookup_buffer,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-voxel-lookup-z-neighbor-encoder"),
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
        pack_rgb8([39, 64, 89]),
        "the z-plane neighbor must contribute through three-dimensional cell distance",
    );
}

#[test]
fn trace_probe_tiles_shader_uses_voxel_cell_descriptor_when_surface_cache_sample_is_invalid() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping voxel fallback Wgpu test because no adapter is available");
        return;
    };
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        &device,
        1,
        0,
        1,
        ProbeTraceTileSurfaceCacheParams {
            texture_available: 1,
            atlas_width: 1,
            atlas_height: 1,
            atlas_columns: 1,
            tile_extent: 1,
        },
        1,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-fallback-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-fallback-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-fallback-tile-schedule",
        &[0_u32, 5, 42, 12],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-voxel-fallback-atlas",
        [0, 0, 0, 255],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-voxel-fallback-depth",
        [255, 255, 255, 0],
    );
    let scene_prepare_descriptor_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-fallback-descriptor",
        &[voxel_cell_descriptor_words(5, 42, 4, [32, 64, 96])],
    );
    let voxel_cell_lookup_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-fallback-lookup",
        &voxel_cell_lookup_words(5, &[(42, 0)]),
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group_with_voxel_lookup(
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
        &voxel_cell_lookup_buffer,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-voxel-fallback-encoder"),
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
    assert_eq!(words.as_slice(), &[1, 7, pack_rgb8([32, 64, 96])]);
}

#[test]
fn trace_probe_tiles_shader_cone_traces_multiple_voxel_cells_when_surface_cache_misses() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping voxel cone-trace Wgpu test because no adapter is available");
        return;
    };
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        &device,
        1,
        0,
        1,
        ProbeTraceTileSurfaceCacheParams {
            texture_available: 1,
            atlas_width: 1,
            atlas_height: 1,
            atlas_columns: 1,
            tile_extent: 1,
        },
        4,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-cone-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-cone-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-cone-tile-schedule",
        &[0_u32, 5, 42, 12],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-voxel-cone-atlas",
        [0, 0, 0, 255],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-voxel-cone-depth",
        [255, 255, 255, 0],
    );
    let scene_prepare_descriptor_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-cone-descriptors",
        &[
            voxel_cell_descriptor_words_with_half_extent(5, 42, 4, [32, 64, 96], 96),
            voxel_cell_descriptor_words_with_half_extent(5, 43, 2, [96, 64, 32], 96),
            voxel_cell_descriptor_words_with_half_extent(5, 48, 8, [240, 0, 0], 96),
            voxel_cell_descriptor_words_with_half_extent(6, 42, 8, [0, 240, 0], 96),
        ],
    );
    let voxel_cell_lookup_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-voxel-cone-lookup",
        &voxel_cell_lookup_words(5, &[(42, 0), (43, 1), (48, 2)]),
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group_with_voxel_lookup(
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
        &voxel_cell_lookup_buffer,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-voxel-cone-encoder"),
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
    assert_eq!(words.as_slice(), &[1, 7, pack_rgb8([39, 64, 89])]);
}
