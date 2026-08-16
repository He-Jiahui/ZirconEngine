use std::sync::mpsc;

use wgpu::util::DeviceExt;

use super::super::super::super::{
    gpu_pending_probe_input::GpuPendingProbeInput, gpu_resident_probe_input::GpuResidentProbeInput,
    seed_quantization::quantized_signed,
};
use super::*;

mod global_sdf;
mod multi_ray_quality;
mod surface_cache_hzb;
mod voxel_lookup;

const PROBE_TRACE_TILE_STORAGE_BUFFER_BINDING_COUNT: u32 = 9;

#[test]
fn trace_probe_tiles_shader_writes_trace_lighting_buffer_from_tile_schedule() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping trace_probe_tiles Wgpu test because no adapter is available");
        return;
    };
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        &device,
        1,
        0,
        2,
        ProbeTraceTileSurfaceCacheParams::unavailable(),
        0,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-test-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-test-tile-schedule",
        &[0_u32, 7, 40, 12, 1, 7, 41, 6],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(&device);
    let fallback_surface_cache = create_probe_trace_tile_fallback_surface_cache_textures(&device);
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
        &fallback_surface_cache.atlas_view,
        &fallback_surface_cache.depth_view,
        &scene_prepare_descriptor_buffer,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-test-encoder"),
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
    assert_ne!(
        words[2], 0,
        "expected trace_probe_tiles.wgsl to write a nonzero packed RGB result"
    );
}

#[test]
fn trace_probe_tiles_shader_samples_surface_cache_atlas_and_depth_textures() {
    let Some((device, queue)) = test_device() else {
        eprintln!(
            "skipping trace_probe_tiles surface-cache Wgpu test because no adapter is available"
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
            atlas_width: 1,
            atlas_height: 1,
            atlas_columns: 1,
            tile_extent: 1,
        },
        0,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-surface-cache-test-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-surface-cache-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-surface-cache-test-tile-schedule",
        &[0_u32, 7, 0, 12],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(&device);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-surface-cache-test-atlas",
        [200, 80, 20, 255],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-surface-cache-test-depth",
        [128, 128, 128, 255],
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
        label: Some("zircon-hybrid-gi-trace-probe-tiles-surface-cache-test-encoder"),
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
        pack_rgb8([150, 60, 15]),
        "expected trace_probe_tiles.wgsl to use surface-cache atlas/depth textureLoad"
    );
}

#[test]
fn trace_probe_tiles_shader_preserves_locally_lit_surface_cache_radiance() {
    let shader = include_str!("../../../../shaders/trace_probe_tiles.wgsl");

    assert!(
        !shader.contains("scene_light_seed"),
        "trace_probe_tiles.wgsl must consume authoritative Surface Cache and Voxel radiance without scene-wide relighting"
    );
}

#[test]
fn trace_probe_tiles_shader_marches_surface_cache_depth_before_voxel_fallback() {
    let Some((device, queue)) = test_device() else {
        eprintln!(
            "skipping trace_probe_tiles surface-cache ray-march Wgpu test because no adapter is available"
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
            atlas_width: 3,
            atlas_height: 1,
            atlas_columns: 3,
            tile_extent: 1,
        },
        0,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-surface-march-test-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-surface-march-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-surface-march-test-tile-schedule",
        &[0_u32, 7, 0, 12],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(&device);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-surface-march-test-atlas",
        3,
        1,
        &[[200, 80, 20, 255], [80, 160, 40, 255], [250, 0, 0, 255]],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-surface-march-test-depth",
        3,
        1,
        &[
            [128, 128, 128, 255],
            [136, 136, 136, 255],
            [240, 240, 240, 255],
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
        label: Some("zircon-hybrid-gi-trace-probe-tiles-surface-march-test-encoder"),
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
        pack_rgb8([127, 74, 19]),
        "expected multi-direction surface-cache traces to include the near depth texel and reject the far depth texel before voxel fallback"
    );
}

#[test]
fn trace_probe_tiles_shader_distributes_surface_cache_ray_steps_by_sample_id() {
    let Some((device, queue)) = test_device() else {
        eprintln!(
            "skipping trace_probe_tiles surface-cache ray-direction Wgpu test because no adapter is available"
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
            atlas_width: 3,
            atlas_height: 3,
            atlas_columns: 3,
            tile_extent: 1,
        },
        0,
    );
    let resident_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-ray-direction-test-resident-probe",
        &[resident_probe_input(7)],
    );
    let pending_probe_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-ray-direction-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        &device,
        "zircon-hybrid-gi-trace-probe-tiles-ray-direction-test-tile-schedule",
        &[0_u32, 7, 4, 8],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(&device, 3);
    let readback_buffer = create_readback_buffer(&device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(&device);
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-ray-direction-test-atlas",
        3,
        3,
        &[
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [100, 40, 20, 255],
            [20, 200, 20, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [200, 80, 40, 255],
        ],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-trace-probe-tiles-ray-direction-test-depth",
        3,
        3,
        &[
            [255, 255, 255, 0],
            [255, 255, 255, 0],
            [255, 255, 255, 0],
            [255, 255, 255, 0],
            [128, 128, 128, 255],
            [132, 132, 132, 255],
            [255, 255, 255, 0],
            [255, 255, 255, 0],
            [132, 132, 132, 255],
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
        label: Some("zircon-hybrid-gi-trace-probe-tiles-ray-direction-test-encoder"),
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
        pack_rgb8([96, 38, 19]),
        "expected the eight-ray surface-cache trace to rotate from the sample-id direction across the deterministic direction set"
    );
}

fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    test_device_with_backends(wgpu::Backends::PRIMARY)
}

fn test_device_with_backends(backends: wgpu::Backends) -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = backends;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    if adapter.limits().max_storage_buffers_per_shader_stage
        < PROBE_TRACE_TILE_STORAGE_BUFFER_BINDING_COUNT
    {
        return None;
    }
    let required_limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: PROBE_TRACE_TILE_STORAGE_BUFFER_BINDING_COUNT,
        ..wgpu::Limits::default()
    };
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits,
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}

fn resident_probe_input(probe_id: u32) -> GpuResidentProbeInput {
    GpuResidentProbeInput {
        probe_id,
        slot: 0,
        ray_budget: 64,
        lineage_trace_support_q: 0,
        position_x_q: 2_064,
        position_y_q: 2_048,
        position_z_q: 2_032,
        radius_q: 96,
        previous_irradiance_rgb: 0,
        runtime_hierarchy_irradiance_rgb: 0,
        runtime_hierarchy_irradiance_weight_q: 0,
        skip_scene_prepare_for_irradiance_q: 0,
        lineage_trace_lighting_rgb: 0,
        skip_scene_prepare_for_trace_q: 0,
        parent_probe_id: u32::MAX,
        resident_ancestor_probe_id: u32::MAX,
        resident_ancestor_depth: 0,
        resident_secondary_ancestor_probe_id: u32::MAX,
        resident_secondary_ancestor_depth: 0,
        resident_tertiary_ancestor_probe_id: u32::MAX,
        resident_tertiary_ancestor_depth: 0,
        resident_quaternary_ancestor_probe_id: u32::MAX,
        resident_quaternary_ancestor_depth: 0,
    }
}

fn create_storage_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[T],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(contents),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn create_trace_lighting_buffer(device: &wgpu::Device, word_count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-test-trace-lighting"),
        size: (word_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

fn create_zeroed_scene_prepare_descriptor_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    create_storage_buffer(
        device,
        "zircon-hybrid-gi-trace-probe-tiles-test-empty-scene-prepare-descriptor",
        &[[0_u32; 12]],
    )
}

fn voxel_cell_descriptor_words(
    clipmap_id: u32,
    cell_index: u32,
    occupancy_count: u32,
    rgb: [u32; 3],
) -> [u32; 12] {
    let zero_position_q = quantized_signed(0.0);
    [
        3,
        clipmap_id,
        cell_index,
        occupancy_count,
        pack_rgb8(rgb),
        zero_position_q,
        zero_position_q,
        zero_position_q,
        0,
        0,
        1,
        0,
    ]
}

fn voxel_cell_descriptor_words_with_half_extent(
    clipmap_id: u32,
    cell_index: u32,
    occupancy_count: u32,
    rgb: [u32; 3],
    cell_half_extent_q: u32,
) -> [u32; 12] {
    let mut words = voxel_cell_descriptor_words(clipmap_id, cell_index, occupancy_count, rgb);
    words[8] = cell_half_extent_q;
    words
}

fn voxel_cell_lookup_words(clipmap_id: u32, descriptor_cells: &[(u32, u32)]) -> Vec<u32> {
    const VOXEL_CELL_COUNT: usize = 64;
    const LOOKUP_WORDS_PER_CLIPMAP: usize = 1 + VOXEL_CELL_COUNT;
    const LOOKUP_CLIPMAP_CAPACITY: usize = 8;

    let mut words = vec![u32::MAX; LOOKUP_WORDS_PER_CLIPMAP * LOOKUP_CLIPMAP_CAPACITY];
    words[0] = clipmap_id;
    for &(cell_index, descriptor_index) in descriptor_cells {
        words[1 + cell_index as usize] = descriptor_index;
    }
    words
}

fn voxel_cell_lookup_words_for_descriptors(
    scene_prepare_descriptors: &[[u32; 12]],
) -> (u32, Vec<u32>) {
    const VOXEL_CELL_COUNT: usize = 64;
    const LOOKUP_WORDS_PER_CLIPMAP: usize = 1 + VOXEL_CELL_COUNT;
    const LOOKUP_CLIPMAP_CAPACITY: usize = 8;
    const INVALID_DESCRIPTOR_INDEX: u32 = u32::MAX;

    let mut words =
        vec![INVALID_DESCRIPTOR_INDEX; LOOKUP_WORDS_PER_CLIPMAP * LOOKUP_CLIPMAP_CAPACITY];
    let mut clipmap_ids = Vec::new();
    for (descriptor_index, descriptor) in scene_prepare_descriptors.iter().enumerate() {
        if descriptor[0] != 3 {
            continue;
        }
        let cell_index = descriptor[2] as usize;
        if cell_index >= VOXEL_CELL_COUNT {
            return (0, vec![INVALID_DESCRIPTOR_INDEX; words.len()]);
        }
        let clipmap_id = descriptor[1];
        let lookup_index = match clipmap_ids.iter().position(|id| *id == clipmap_id) {
            Some(index) => index,
            None if clipmap_id != INVALID_DESCRIPTOR_INDEX
                && clipmap_ids.len() < LOOKUP_CLIPMAP_CAPACITY =>
            {
                words[clipmap_ids.len() * LOOKUP_WORDS_PER_CLIPMAP] = clipmap_id;
                clipmap_ids.push(clipmap_id);
                clipmap_ids.len() - 1
            }
            None => return (0, vec![INVALID_DESCRIPTOR_INDEX; words.len()]),
        };
        let word_index = lookup_index * LOOKUP_WORDS_PER_CLIPMAP + 1 + cell_index;
        if words[word_index] != INVALID_DESCRIPTOR_INDEX {
            return (0, vec![INVALID_DESCRIPTOR_INDEX; words.len()]);
        }
        let Ok(descriptor_index) = u32::try_from(descriptor_index) else {
            return (0, vec![INVALID_DESCRIPTOR_INDEX; words.len()]);
        };
        words[word_index] = descriptor_index;
    }

    (clipmap_ids.len() as u32, words)
}

#[test]
fn voxel_lookup_fixture_maps_voxel_descriptors_and_rejects_duplicate_cells() {
    let (clipmap_count, words) = voxel_cell_lookup_words_for_descriptors(&[
        [0_u32; 12],
        voxel_cell_descriptor_words(5, 42, 4, [32, 64, 96]),
        voxel_cell_descriptor_words(5, 58, 2, [96, 64, 32]),
    ]);

    assert_eq!(clipmap_count, 1);
    assert_eq!(words[0], 5);
    assert_eq!(words[1 + 42], 1);
    assert_eq!(words[1 + 58], 2);

    let (clipmap_count, words) = voxel_cell_lookup_words_for_descriptors(&[
        voxel_cell_descriptor_words(5, 42, 4, [32, 64, 96]),
        voxel_cell_descriptor_words(5, 42, 2, [96, 64, 32]),
    ]);
    assert_eq!(clipmap_count, 0);
    assert!(words.iter().all(|word| *word == u32::MAX));
}

#[test]
fn voxel_descriptor_fixture_uses_runtime_signed_position_encoding() {
    let words = voxel_cell_descriptor_words(7, 2, 4, [24, 96, 160]);
    let zero_position_q = quantized_signed(0.0);

    assert_eq!(&words[5..8], &[zero_position_q; 3]);
}

fn create_readback_buffer(device: &wgpu::Device, word_count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-test-readback"),
        size: (word_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn create_test_surface_cache_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    rgba: [u8; 4],
) -> (wgpu::Texture, wgpu::TextureView) {
    create_test_surface_cache_texture_with_pixels(device, queue, label, 1, 1, &[rgba])
}

fn create_test_surface_cache_texture_with_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    width: u32,
    height: u32,
    rgba_pixels: &[[u8; 4]],
) -> (wgpu::Texture, wgpu::TextureView) {
    assert_eq!(rgba_pixels.len(), (width * height) as usize);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(rgba_pixels),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-hybrid-gi-trace-probe-tiles-test-surface-cache-view"),
        ..Default::default()
    });
    (texture, view)
}

fn pack_rgb8(rgb: [u32; 3]) -> u32 {
    rgb[0].min(255) | (rgb[1].min(255) << 8) | (rgb[2].min(255) << 16)
}

fn readback_u32s(device: &wgpu::Device, buffer: &wgpu::Buffer, word_count: usize) -> Vec<u32> {
    let slice = buffer.slice(..(word_count * std::mem::size_of::<u32>()) as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let data = bytemuck::cast_slice(&mapped[..]).to_vec();
    drop(mapped);
    buffer.unmap();
    data
}
