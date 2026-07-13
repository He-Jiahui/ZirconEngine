use std::{fs, path::PathBuf, sync::mpsc};

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use image::{ImageBuffer, ImageFormat, Rgba};
use wgpu::util::DeviceExt;

const GRID: [u32; 3] = [16, 8, 8];
const READBACK_BYTES_PER_ROW: u32 = 256;
const OUTPUT_PNG: &str = "plan18_volumetric_temporal_reprojection_wgpu_20260711.png";
const OUTPUT_REPORT: &str = "plan18_volumetric_temporal_reprojection_wgpu_20260711.txt";
const MEDIA: [u16; 4] = [0x3000, 0x2c00, 0x2800, 0x3000];
const MATCHED_HISTORY: [u16; 4] = [0x3c00, 0x3400, 0x3000, 0x3000];
const REJECTED_HISTORY: [u16; 4] = [0x3c00, 0x3400, 0x3000, 0x3c00];

const LIGHT_SCATTER_SHADER: &str = concat!(
    include_str!(
        "../src/graphics/scene/scene_renderer/advanced_lighting/froxel/shaders/zr_froxel_reconstruct.wgsl"
    ),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/types.wgsl"
    ),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/main.wgsl"
    ),
);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FroxelViewParams {
    world_from_clip: [[f32; 4]; 4],
    camera_position_projection: [f32; 4],
    camera_forward: [f32; 4],
    depth: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TemporalReprojection {
    previous_clip_from_world: [[f32; 4]; 4],
    previous_camera_position: [f32; 4],
    previous_camera_forward: [f32; 4],
    previous_depth: [f32; 4],
    jitter_and_history: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightScatterParams {
    grid_and_light_count: [u32; 4],
    viewport_size: [u32; 4],
    phase_g: [f32; 4],
    view: FroxelViewParams,
    temporal: TemporalReprojection,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightGridParams {
    world_to_view: [[f32; 4]; 4],
    zbin_scale: f32,
    zbin_offset: f32,
    bin_count: u32,
    words_per_tile: u32,
    tile_resolution: [u32; 2],
    tile_size_px: u32,
    light_count: u32,
    projection_mode: u32,
    alignment_padding: [u32; 3],
    padding: [u32; 3],
    tail_padding: u32,
}

#[test]
fn runtime_volumetric_temporal_wgpu_reprojects_and_rejects_history() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping volumetric temporal WGPU contract because no adapter is available");
        return;
    };

    let result = run_temporal_matrix(&device, &queue);
    assert!(result.accumulated_changed_count > result.raw.len() / 2);
    assert!(result.accumulated_history_distance < result.raw_history_distance * 0.35);
    assert!(result.rejection_max_error <= 0.002);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_temporal_png(
        output_dir.join(OUTPUT_PNG),
        &result.raw,
        &result.accumulated,
        &result.rejected,
    );
    fs::write(
        output_dir.join(OUTPUT_REPORT),
        format!(
            "png={OUTPUT_PNG}\nwidth=770\nheight=128\ngpu_froxel_dimensions=16x8x8\nproduction_shader=FroxelLightScatter\npanels=temporal_off|matched_history_accumulated|extinction_history_rejected\nworkgroup_size=4x4x4\ndispatch=4,2,2\nhistory_weight=0.9\njitter_xy_froxels=0.25,-0.125\njitter_z_halton_base5=0.1\nmatched_history_rgba=1.0,0.25,0.125,0.125\nrejected_history_rgba=1.0,0.25,0.125,1.0\naccumulated_changed_froxels={}\nraw_history_distance={:.6}\naccumulated_history_distance={:.6}\nextinction_rejection_max_error={:.6}\nreference=UE_VolumetricFog_temporal_jitter_history_reprojection\n",
            result.accumulated_changed_count,
            result.raw_history_distance,
            result.accumulated_history_distance,
            result.rejection_max_error,
        ),
    )
    .unwrap();
}

struct TemporalResult {
    raw: Vec<[f32; 4]>,
    accumulated: Vec<[f32; 4]>,
    rejected: Vec<[f32; 4]>,
    accumulated_changed_count: usize,
    raw_history_distance: f32,
    accumulated_history_distance: f32,
    rejection_max_error: f32,
}

fn run_temporal_matrix(device: &wgpu::Device, queue: &wgpu::Queue) -> TemporalResult {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("volumetric-temporal-production-light-scatter"),
        source: wgpu::ShaderSource::Wgsl(LIGHT_SCATTER_SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("volumetric-temporal-production-pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let media = create_input_volume(device, "volumetric-temporal-media");
    let matched_history = create_input_volume(device, "volumetric-temporal-matched-history");
    let rejected_history = create_input_volume(device, "volumetric-temporal-rejected-history");
    write_constant_volume(queue, &media, MEDIA);
    write_constant_volume(queue, &matched_history, MATCHED_HISTORY);
    write_constant_volume(queue, &rejected_history, REJECTED_HISTORY);
    let media_view = d3_view(&media, "volumetric-temporal-media-view");
    let matched_history_view = d3_view(&matched_history, "volumetric-temporal-matched-view");
    let rejected_history_view = d3_view(&rejected_history, "volumetric-temporal-rejected-view");

    let raw = create_output_volume(device, "volumetric-temporal-raw");
    let accumulated = create_output_volume(device, "volumetric-temporal-accumulated");
    let rejected = create_output_volume(device, "volumetric-temporal-rejection");
    let raw_view = d3_view(&raw, "volumetric-temporal-raw-view");
    let accumulated_view = d3_view(&accumulated, "volumetric-temporal-accumulated-view");
    let rejected_view = d3_view(&rejected, "volumetric-temporal-rejection-view");

    let projection = Mat4::perspective_rh(60_f32.to_radians(), 2.0, 0.1, 20.0);
    let view = FroxelViewParams {
        world_from_clip: projection.inverse().to_cols_array_2d(),
        camera_position_projection: [0.0, 0.0, 0.0, 0.0],
        camera_forward: [0.0, 0.0, -1.0, 0.0],
        depth: [0.1, 20.0, 2.0, 0.0],
    };
    let temporal_base = TemporalReprojection {
        previous_clip_from_world: projection.to_cols_array_2d(),
        previous_camera_position: [0.0, 0.0, 0.0, 0.0],
        previous_camera_forward: [0.0, 0.0, -1.0, 0.0],
        previous_depth: [0.1, 20.0, 2.0, 0.0],
        jitter_and_history: [0.25, -0.125, 0.1, 0.0],
    };
    let light_grid = create_buffer(
        device,
        "volumetric-temporal-light-grid",
        &[LightGridParams {
            world_to_view: Mat4::IDENTITY.to_cols_array_2d(),
            zbin_scale: 0.0,
            zbin_offset: 0.0,
            bin_count: 1,
            words_per_tile: 1,
            tile_resolution: [1, 1],
            tile_size_px: 16,
            light_count: 0,
            projection_mode: 0,
            alignment_padding: [0; 3],
            padding: [0; 3],
            tail_padding: 0,
        }],
        wgpu::BufferUsages::UNIFORM,
    );
    let light_zbins = create_buffer(
        device,
        "volumetric-temporal-light-zbins",
        &[u32::MAX, 0, 0],
        wgpu::BufferUsages::STORAGE,
    );
    let light_tile_masks = create_buffer(
        device,
        "volumetric-temporal-light-masks",
        &[0_u32],
        wgpu::BufferUsages::STORAGE,
    );
    let light_data = create_buffer(
        device,
        "volumetric-temporal-light-data",
        &[0_u32; 24],
        wgpu::BufferUsages::STORAGE,
    );
    let (shadow_atlas, shadow_view, shadow_sampler, shadow_slots, shadow_globals) =
        create_shadow_resources(device);

    let group_one_layout = pipeline.get_bind_group_layout(1);
    let group_one = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("volumetric-temporal-scene-bind-group"),
        layout: &group_one_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::TextureView(&shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: wgpu::BindingResource::Sampler(&shadow_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: shadow_slots.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 11,
                resource: shadow_globals.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 20,
                resource: light_grid.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 21,
                resource: light_zbins.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 22,
                resource: light_tile_masks.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("volumetric-temporal-production-encoder"),
    });
    encode_case(
        device,
        &pipeline,
        &group_one,
        &mut encoder,
        view,
        TemporalReprojection {
            jitter_and_history: [0.25, -0.125, 0.1, 0.0],
            ..temporal_base
        },
        &media_view,
        &media_view,
        &light_data,
        &raw_view,
        "raw",
    );
    encode_case(
        device,
        &pipeline,
        &group_one,
        &mut encoder,
        view,
        TemporalReprojection {
            jitter_and_history: [0.25, -0.125, 0.1, 0.9],
            ..temporal_base
        },
        &media_view,
        &matched_history_view,
        &light_data,
        &accumulated_view,
        "accumulated",
    );
    encode_case(
        device,
        &pipeline,
        &group_one,
        &mut encoder,
        view,
        TemporalReprojection {
            jitter_and_history: [0.25, -0.125, 0.1, 0.9],
            ..temporal_base
        },
        &media_view,
        &rejected_history_view,
        &light_data,
        &rejected_view,
        "rejected",
    );

    let readbacks = [raw, accumulated, rejected]
        .iter()
        .enumerate()
        .map(|(index, texture)| create_readback(device, &mut encoder, texture, index))
        .collect::<Vec<_>>();
    queue.submit([encoder.finish()]);
    let raw = read_volume(device, &readbacks[0]);
    let accumulated = read_volume(device, &readbacks[1]);
    let rejected = read_volume(device, &readbacks[2]);
    drop(shadow_atlas);

    let history_rgb = [1.0, 0.25, 0.125];
    TemporalResult {
        accumulated_changed_count: accumulated
            .iter()
            .zip(&raw)
            .filter(|(left, right)| rgb_distance(left, right) > 0.02)
            .count(),
        raw_history_distance: average_history_distance(&raw, history_rgb),
        accumulated_history_distance: average_history_distance(&accumulated, history_rgb),
        rejection_max_error: rejected
            .iter()
            .zip(&raw)
            .map(|(left, right)| rgb_distance(left, right))
            .fold(0.0, f32::max),
        raw,
        accumulated,
        rejected,
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_case(
    device: &wgpu::Device,
    pipeline: &wgpu::ComputePipeline,
    group_one: &wgpu::BindGroup,
    encoder: &mut wgpu::CommandEncoder,
    view: FroxelViewParams,
    temporal: TemporalReprojection,
    media_view: &wgpu::TextureView,
    history_view: &wgpu::TextureView,
    light_data: &wgpu::Buffer,
    output_view: &wgpu::TextureView,
    label: &str,
) {
    let params = create_buffer(
        device,
        &format!("volumetric-temporal-{label}-params"),
        &[LightScatterParams {
            grid_and_light_count: [GRID[0], GRID[1], GRID[2], 0],
            viewport_size: [GRID[0], GRID[1], 0, 0],
            phase_g: [0.0; 4],
            view,
            temporal,
        }],
        wgpu::BufferUsages::UNIFORM,
    );
    let group_zero_layout = pipeline.get_bind_group_layout(0);
    let group_zero = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("volumetric-temporal-{label}-bind-group")),
        layout: &group_zero_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(media_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: light_data.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(output_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(history_view),
            },
        ],
    });
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(&format!("volumetric-temporal-{label}-pass")),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &group_zero, &[]);
    pass.set_bind_group(1, group_one, &[]);
    pass.dispatch_workgroups(4, 2, 2);
}

fn create_input_volume(device: &wgpu::Device, label: &str) -> wgpu::Texture {
    create_volume(
        device,
        label,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    )
}

fn create_output_volume(device: &wgpu::Device, label: &str) -> wgpu::Texture {
    create_volume(
        device,
        label,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    )
}

fn create_volume(device: &wgpu::Device, label: &str, usage: wgpu::TextureUsages) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: volume_extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba16Float,
        usage,
        view_formats: &[],
    })
}

fn d3_view(texture: &wgpu::Texture, label: &str) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(label),
        dimension: Some(wgpu::TextureViewDimension::D3),
        ..Default::default()
    })
}

fn write_constant_volume(queue: &wgpu::Queue, texture: &wgpu::Texture, value: [u16; 4]) {
    let texel = bytemuck::cast_slice(&value);
    let mut data = Vec::with_capacity((GRID[0] * GRID[1] * GRID[2] * 8) as usize);
    for _ in 0..GRID[0] * GRID[1] * GRID[2] {
        data.extend_from_slice(texel);
    }
    queue.write_texture(
        texture.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(GRID[0] * 8),
            rows_per_image: Some(GRID[1]),
        },
        volume_extent(),
    );
}

fn create_shadow_resources(
    device: &wgpu::Device,
) -> (
    wgpu::Texture,
    wgpu::TextureView,
    wgpu::Sampler,
    wgpu::Buffer,
    wgpu::Buffer,
) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("volumetric-temporal-shadow-atlas"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("volumetric-temporal-shadow-sampler"),
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });
    let slots = create_buffer(
        device,
        "volumetric-temporal-shadow-slots",
        &[0_u32; 24],
        wgpu::BufferUsages::STORAGE,
    );
    let globals = create_buffer(
        device,
        "volumetric-temporal-shadow-globals",
        &[0_u32; 12],
        wgpu::BufferUsages::UNIFORM,
    );
    (texture, view, sampler, slots, globals)
}

fn create_buffer<T: Pod>(
    device: &wgpu::Device,
    label: &str,
    values: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(values),
        usage,
    })
}

fn create_readback(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    index: usize,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("volumetric-temporal-readback-{index}")),
        size: u64::from(READBACK_BYTES_PER_ROW * GRID[1] * GRID[2]),
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
                rows_per_image: Some(GRID[1]),
            },
        },
        volume_extent(),
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
    let mut texels = Vec::with_capacity((GRID[0] * GRID[1] * GRID[2]) as usize);
    let image_stride = READBACK_BYTES_PER_ROW as usize * GRID[1] as usize;
    for z in 0..GRID[2] as usize {
        for y in 0..GRID[1] as usize {
            let row_offset = z * image_stride + y * READBACK_BYTES_PER_ROW as usize;
            for x in 0..GRID[0] as usize {
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

fn volume_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: GRID[0],
        height: GRID[1],
        depth_or_array_layers: GRID[2],
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

fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x03ff);
    let value = match exponent {
        0 => {
            if mantissa == 0 {
                sign
            } else {
                let mut mantissa = mantissa;
                let mut shift = 0_u32;
                while mantissa & 0x0400 == 0 {
                    mantissa <<= 1;
                    shift += 1;
                }
                let exponent = 127 - 15 - shift;
                sign | (exponent << 23) | ((mantissa & 0x03ff) << 13)
            }
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 127 - 15) << 23) | (mantissa << 13),
    };
    f32::from_bits(value)
}

fn write_temporal_png(
    path: PathBuf,
    raw: &[[f32; 4]],
    accumulated: &[[f32; 4]],
    rejected: &[[f32; 4]],
) {
    const SCALE: u32 = 16;
    const GAP: u32 = 1;
    let panel_width = GRID[0] * SCALE;
    let mut image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::new(panel_width * 3 + GAP * 2, GRID[2] * SCALE);
    for (panel, texels) in [raw, accumulated, rejected].into_iter().enumerate() {
        for z in 0..GRID[2] {
            for x in 0..GRID[0] {
                let mut color = [0.0; 3];
                for y in 0..GRID[1] {
                    let sample = texels[((z * GRID[1] + y) * GRID[0] + x) as usize];
                    for channel in 0..3 {
                        color[channel] += sample[channel] / GRID[1] as f32;
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

fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs/tests/runtime/render")
}

fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-volumetric-temporal-contract-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
