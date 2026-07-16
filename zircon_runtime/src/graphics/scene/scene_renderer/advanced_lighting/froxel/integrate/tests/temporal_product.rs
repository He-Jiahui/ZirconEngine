use super::super::super::GpuFroxelTemporalReprojection;
use super::*;

const TEMPORAL_PRODUCT_PNG: &str = "plan18_volumetric_temporal_reprojection_wgpu_20260711.png";
const TEMPORAL_PRODUCT_REPORT: &str = "plan18_volumetric_temporal_reprojection_wgpu_20260711.txt";
const MATCHED_HISTORY: [u16; 4] = [0x3c00, 0x3400, 0x3000, 0x3000];
const REJECTED_HISTORY: [u16; 4] = [0x3c00, 0x3400, 0x3000, 0x3c00];
const MEDIA: [u16; 4] = [0x3000, 0x2c00, 0x2800, 0x3000];

#[test]
fn render_volumetric_temporal_reprojects_history_and_rejects_extinction_changes() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let result = run_temporal_matrix(&device, &queue);

    assert!(result.accumulated_changed_count > result.raw.len() / 2);
    assert!(result.accumulated_history_distance < result.jittered_history_distance * 0.35);
    assert!(result.rejection_max_error <= 0.002);
}

#[test]
#[ignore]
fn export_volumetric_temporal_reprojection_wgpu_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping volumetric temporal product because no adapter is available");
        return;
    };
    let result = run_temporal_matrix(&device, &queue);
    assert!(result.accumulated_changed_count > result.raw.len() / 2);
    assert!(result.accumulated_history_distance < result.jittered_history_distance * 0.35);
    assert!(result.rejection_max_error <= 0.002);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_temporal_png(
        output_dir.join(TEMPORAL_PRODUCT_PNG),
        &result.raw,
        &result.accumulated,
        &result.rejected,
    );
    fs::write(
        output_dir.join(TEMPORAL_PRODUCT_REPORT),
        format!(
            "png={TEMPORAL_PRODUCT_PNG}\nwidth=770\nheight=128\ngpu_froxel_dimensions=16x8x8\npanels=raw_temporal_off|history_accumulated|extinction_history_rejected\nworkgroup_size=4x4x4\ndispatch=4,2,2\nhistory_weight=0.9\njitter_sequence_index=3\njitter_xy_pixels=0.25,-0.125\njitter_z_halton_base5=0.1\nmatched_history_rgba=1.0,0.25,0.125,0.125\nrejected_history_rgba=1.0,0.25,0.125,1.0\naccumulated_changed_froxels={}\nraw_history_distance={:.6}\naccumulated_history_distance={:.6}\nextinction_rejection_max_error={:.6}\nreference=UE_VolumetricFog_LightScatteringSampleJitterMultiplier_history_reprojection\n",
            result.accumulated_changed_count,
            result.jittered_history_distance,
            result.accumulated_history_distance,
            result.rejection_max_error,
        ),
    )
    .unwrap();
}

struct TemporalMatrixResult {
    raw: Vec<[f32; 4]>,
    accumulated: Vec<[f32; 4]>,
    rejected: Vec<[f32; 4]>,
    accumulated_changed_count: usize,
    jittered_history_distance: f32,
    accumulated_history_distance: f32,
    rejection_max_error: f32,
}

fn run_temporal_matrix(device: &wgpu::Device, queue: &wgpu::Queue) -> TemporalMatrixResult {
    let media = create_test_volume(device, "volumetric-temporal-media", false);
    write_constant_volume(queue, &media, MEDIA);
    let matched_history = create_test_volume(device, "volumetric-temporal-history", false);
    write_constant_volume(queue, &matched_history, MATCHED_HISTORY);
    let rejected_history =
        create_test_volume(device, "volumetric-temporal-rejected-history", false);
    write_constant_volume(queue, &rejected_history, REJECTED_HISTORY);
    let raw = create_test_volume(device, "volumetric-temporal-raw", true);
    let jittered = create_test_volume(device, "volumetric-temporal-jittered", true);
    let accumulated = create_test_volume(device, "volumetric-temporal-accumulated", true);
    let rejected = create_test_volume(device, "volumetric-temporal-rejected", true);
    let views = [
        media.create_view(&d3_view_descriptor("volumetric-temporal-media-view")),
        matched_history.create_view(&d3_view_descriptor("volumetric-temporal-history-view")),
        rejected_history.create_view(&d3_view_descriptor(
            "volumetric-temporal-rejected-history-view",
        )),
        raw.create_view(&d3_view_descriptor("volumetric-temporal-raw-view")),
        jittered.create_view(&d3_view_descriptor("volumetric-temporal-jittered-view")),
        accumulated.create_view(&d3_view_descriptor("volumetric-temporal-accumulated-view")),
        rejected.create_view(&d3_view_descriptor("volumetric-temporal-rejected-view")),
    ];
    let lighting = create_lighting_resources(device);
    let (_shadow, shadow_view, shadow_sampler, shadow_slots, shadow_globals) =
        create_shadow_resources(device);
    let grid = FroxelGridParams {
        dimensions: TEST_GRID,
        near_depth: 0.1,
        far_depth: 20.0,
        depth_distribution_exp: 2.0,
    };
    let mut camera = ViewportCameraSnapshot::default();
    camera.z_far = 20.0;
    camera.aspect_ratio = 2.0;
    camera.temporal_jitter = crate::core::framework::render::TemporalJitterSample {
        offset_pixels: crate::core::math::Vec2::new(0.25, -0.125),
        sequence_index: 3,
    };
    let view = FroxelViewReconstruction::from_camera(&camera, UVec2::new(16, 8));
    let pipeline = FroxelLightScatterPipeline::new(device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("volumetric-temporal-matrix-encoder"),
    });
    clear_shadow_atlas(&mut encoder, &shadow_view);
    encode_scatter(
        &pipeline,
        device,
        &mut encoder,
        grid,
        view,
        &camera,
        false,
        false,
        &views[0],
        &views[0],
        &views[3],
        &lighting,
        &shadow_view,
        &shadow_sampler,
        &shadow_slots,
        &shadow_globals,
    );
    encode_scatter(
        &pipeline,
        device,
        &mut encoder,
        grid,
        view,
        &camera,
        true,
        false,
        &views[0],
        &views[0],
        &views[4],
        &lighting,
        &shadow_view,
        &shadow_sampler,
        &shadow_slots,
        &shadow_globals,
    );
    encode_scatter(
        &pipeline,
        device,
        &mut encoder,
        grid,
        view,
        &camera,
        true,
        true,
        &views[0],
        &views[1],
        &views[5],
        &lighting,
        &shadow_view,
        &shadow_sampler,
        &shadow_slots,
        &shadow_globals,
    );
    encode_scatter(
        &pipeline,
        device,
        &mut encoder,
        grid,
        view,
        &camera,
        true,
        true,
        &views[0],
        &views[2],
        &views[6],
        &lighting,
        &shadow_view,
        &shadow_sampler,
        &shadow_slots,
        &shadow_globals,
    );
    let readbacks = [raw, jittered, accumulated, rejected]
        .iter()
        .enumerate()
        .map(|(index, texture)| create_volume_readback(device, &mut encoder, texture, index))
        .collect::<Vec<_>>();
    queue.submit([encoder.finish()]);
    let raw = read_volume(device, &readbacks[0]);
    let jittered = read_volume(device, &readbacks[1]);
    let accumulated = read_volume(device, &readbacks[2]);
    let rejected = read_volume(device, &readbacks[3]);
    let history_rgb = [1.0, 0.25, 0.125];
    TemporalMatrixResult {
        accumulated_changed_count: accumulated
            .iter()
            .zip(&jittered)
            .filter(|(left, right)| rgb_distance(left, right) > 0.02)
            .count(),
        jittered_history_distance: average_history_distance(&jittered, history_rgb),
        accumulated_history_distance: average_history_distance(&accumulated, history_rgb),
        rejection_max_error: rejected
            .iter()
            .zip(&jittered)
            .map(|(left, right)| rgb_distance(left, right))
            .fold(0.0, f32::max),
        raw,
        accumulated,
        rejected,
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_scatter(
    pipeline: &FroxelLightScatterPipeline,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    grid: FroxelGridParams,
    view: FroxelViewReconstruction,
    camera: &ViewportCameraSnapshot,
    jitter_enabled: bool,
    history_available: bool,
    media_view: &wgpu::TextureView,
    history_view: &wgpu::TextureView,
    output_view: &wgpu::TextureView,
    lighting: &LightingResources,
    shadow_view: &wgpu::TextureView,
    shadow_sampler: &wgpu::Sampler,
    shadow_slots: &wgpu::Buffer,
    shadow_globals: &wgpu::Buffer,
) {
    pipeline
        .encode(
            device,
            encoder,
            FroxelLightScatterRequest {
                grid,
                view,
                phase_g: 0.0,
                ambient_radiance: Vec3::ZERO,
                viewport_size: TEST_OUTPUT,
                media_view,
                history_view,
                temporal: GpuFroxelTemporalReprojection::new(
                    camera,
                    Some(camera),
                    UVec2::new(16, 8),
                    grid,
                    jitter_enabled,
                    history_available,
                ),
                light_buffer: &lighting.light_buffer,
                light_count: 1,
                light_grid_params_buffer: &lighting.params_buffer,
                light_zbins_buffer: &lighting.zbins_buffer,
                light_tile_masks_buffer: &lighting.tile_masks_buffer,
                shadow_atlas_view: shadow_view,
                shadow_sampler,
                shadow_slots_buffer: shadow_slots,
                shadow_globals_buffer: shadow_globals,
                output_view,
            },
        )
        .unwrap();
}

fn create_test_volume(device: &wgpu::Device, label: &str, output: bool) -> wgpu::Texture {
    let usage = if output {
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC
    } else {
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
    };
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: test_volume_extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba16Float,
        usage,
        view_formats: &[],
    })
}

fn write_constant_volume(queue: &wgpu::Queue, texture: &wgpu::Texture, value: [u16; 4]) {
    let texel = bytemuck::cast_slice(&value);
    let mut data = Vec::with_capacity((TEST_GRID[0] * TEST_GRID[1] * TEST_GRID[2] * 8) as usize);
    for _ in 0..TEST_GRID[0] * TEST_GRID[1] * TEST_GRID[2] {
        data.extend_from_slice(texel);
    }
    queue.write_texture(
        texture.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TEST_GRID[0] * 8),
            rows_per_image: Some(TEST_GRID[1]),
        },
        test_volume_extent(),
    );
}

fn create_volume_readback(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    index: usize,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("volumetric-temporal-readback-{index}")),
        size: u64::from(READBACK_BYTES_PER_ROW * TEST_GRID[1] * TEST_GRID[2]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(READBACK_BYTES_PER_ROW),
                rows_per_image: Some(TEST_GRID[1]),
            },
        },
        test_volume_extent(),
    );
    buffer
}

fn read_volume(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<[f32; 4]> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let mut texels = Vec::with_capacity((TEST_GRID[0] * TEST_GRID[1] * TEST_GRID[2]) as usize);
    let image_stride = READBACK_BYTES_PER_ROW as usize * TEST_GRID[1] as usize;
    for z in 0..TEST_GRID[2] as usize {
        for y in 0..TEST_GRID[1] as usize {
            let row_offset = z * image_stride + y * READBACK_BYTES_PER_ROW as usize;
            for x in 0..TEST_GRID[0] as usize {
                let offset = row_offset + x * 8;
                let words = bytemuck::cast_slice::<u8, u16>(&mapped[offset..offset + 8]);
                texels.push([
                    f16_bits_to_f32(words[0]),
                    f16_bits_to_f32(words[1]),
                    f16_bits_to_f32(words[2]),
                    f16_bits_to_f32(words[3]),
                ]);
            }
        }
    }
    drop(mapped);
    buffer.unmap();
    texels
}

fn test_volume_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: TEST_GRID[0],
        height: TEST_GRID[1],
        depth_or_array_layers: TEST_GRID[2],
    }
}

fn rgb_distance(left: &[f32; 4], right: &[f32; 4]) -> f32 {
    (0..3)
        .map(|channel| (left[channel] - right[channel]).abs())
        .sum()
}

fn average_history_distance(texels: &[[f32; 4]], history: [f32; 3]) -> f32 {
    texels
        .iter()
        .map(|sample| {
            (0..3)
                .map(|channel| (sample[channel] - history[channel]).abs())
                .sum::<f32>()
        })
        .sum::<f32>()
        / texels.len() as f32
}

fn write_temporal_png(
    path: PathBuf,
    raw: &[[f32; 4]],
    accumulated: &[[f32; 4]],
    rejected: &[[f32; 4]],
) {
    const SCALE: u32 = 16;
    const GAP: u32 = 1;
    let panel_width = TEST_GRID[0] * SCALE;
    let mut image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::new(panel_width * 3 + GAP * 2, TEST_GRID[2] * SCALE);
    for (panel, texels) in [raw, accumulated, rejected].into_iter().enumerate() {
        for z in 0..TEST_GRID[2] {
            for x in 0..TEST_GRID[0] {
                let mut color = [0.0; 3];
                for y in 0..TEST_GRID[1] {
                    let sample = texels[((z * TEST_GRID[1] + y) * TEST_GRID[0] + x) as usize];
                    for channel in 0..3 {
                        color[channel] += sample[channel] / TEST_GRID[1] as f32;
                    }
                }
                let mapped = color.map(|value| {
                    let exposed = value.max(0.0) * 2.0;
                    let reinhard = exposed / (1.0 + exposed);
                    (reinhard.powf(1.0 / 2.2) * 255.0 + 0.5) as u8
                });
                let panel_origin = panel as u32 * (panel_width + GAP);
                for py in 0..SCALE {
                    for px in 0..SCALE {
                        image.put_pixel(
                            panel_origin + x * SCALE + px,
                            z * SCALE + py,
                            Rgba(if px == 0 || py == 0 {
                                [5, 7, 9, 255]
                            } else {
                                [mapped[0], mapped[1], mapped[2], 255]
                            }),
                        );
                    }
                }
            }
        }
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}
