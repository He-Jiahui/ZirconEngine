use std::{fs, path::PathBuf, sync::mpsc};

use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, ImageFormat, Rgba};
use wgpu::util::DeviceExt;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 64;
const BYTES_PER_ROW: u32 = 256;
const OUTPUT_PNG: &str = "plan18_hybrid_gi_m4_source_ledger_wgpu_20260713.png";
const OUTPUT_REPORT: &str = "plan18_hybrid_gi_m4_source_ledger_wgpu_20260713.txt";

const POST_PROCESS_SHADER: &str = concat!(
    include_str!("../src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl"),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl"
    )
);

const SOURCE_FULL_DYNAMIC: u32 = 1;
const SOURCE_BAKED_BASELINE: u32 = 2;
const SOURCE_DYNAMIC_DELTA: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostProcessParams {
    viewport_and_clusters: [u32; 4],
    cluster_dimensions: [u32; 4],
    feature_flags: [u32; 4],
    lighting_flags: [u32; 4],
    hybrid_gi_counts: [u32; 4],
    hybrid_gi_source_ledger: [u32; 4],
    anti_alias: [u32; 4],
    blends: [f32; 4],
    grading: [f32; 4],
    tint_and_probe: [f32; 4],
    hybrid_gi_color_and_intensity: [f32; 4],
    baked_color_and_intensity: [f32; 4],
    effect_flags: [u32; 4],
    effect_tonemap_lut: [f32; 4],
    effect_blur_dof: [f32; 4],
    effect_dof_lens: [f32; 4],
    effect_vignette_grain: [f32; 4],
    effect_chromatic_fog: [f32; 4],
    effect_fog_color: [f32; 4],
    effect_dither_ssr: [f32; 4],
    effect_ssr_limits: [f32; 4],
    effect_depth: [f32; 4],
    effect_projection: [f32; 4],
    effect_view_x: [f32; 4],
    effect_view_y: [f32; 4],
    effect_view_z: [f32; 4],
    effect_motion_blur: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuHybridGiProbe {
    screen_uv_and_radius: [f32; 4],
    irradiance_and_intensity: [f32; 4],
    hierarchy_irradiance_rgb_and_weight: [f32; 4],
    hierarchy_rt_lighting_rgb_and_weight: [f32; 4],
    temporal_signature_and_source: [f32; 4],
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    scene_rgba: [u8; 4],
    global_source_mask: u32,
    global_baked_weight: u32,
    probe_source_mask: u32,
    probe_dynamic_weight: f32,
}

#[test]
fn hybrid_gi_m4_source_ledger_wgpu_exports_product_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping HybridGI M4 WGPU product because no adapter is available");
        return;
    };

    let scenarios = [
        Scenario {
            name: "full_dynamic",
            scene_rgba: [25, 25, 25, 255],
            global_source_mask: SOURCE_FULL_DYNAMIC,
            global_baked_weight: 0,
            probe_source_mask: SOURCE_FULL_DYNAMIC,
            probe_dynamic_weight: 1.0,
        },
        Scenario {
            name: "baked_baseline",
            scene_rgba: [70, 70, 70, 255],
            global_source_mask: SOURCE_BAKED_BASELINE | SOURCE_DYNAMIC_DELTA,
            global_baked_weight: 255,
            probe_source_mask: SOURCE_BAKED_BASELINE,
            probe_dynamic_weight: 0.0,
        },
        Scenario {
            name: "baked_plus_dynamic_delta",
            scene_rgba: [70, 70, 70, 255],
            global_source_mask: SOURCE_BAKED_BASELINE | SOURCE_DYNAMIC_DELTA,
            global_baked_weight: 255,
            probe_source_mask: SOURCE_BAKED_BASELINE | SOURCE_DYNAMIC_DELTA,
            probe_dynamic_weight: 1.0,
        },
        Scenario {
            name: "illegal_baked_plus_full_rejected",
            scene_rgba: [70, 70, 70, 255],
            global_source_mask: SOURCE_BAKED_BASELINE | SOURCE_DYNAMIC_DELTA,
            global_baked_weight: 255,
            probe_source_mask: SOURCE_BAKED_BASELINE | SOURCE_FULL_DYNAMIC,
            probe_dynamic_weight: 1.0,
        },
    ];
    let frames = scenarios
        .iter()
        .map(|scenario| render_scenario(&device, &queue, *scenario))
        .collect::<Vec<_>>();
    let centers = frames
        .iter()
        .map(|frame| frame[((HEIGHT / 2 * WIDTH + WIDTH / 2) * 4) as usize..][..4].to_vec())
        .collect::<Vec<_>>();

    assert!(rgb_sum(&centers[0]) > 0);
    assert!(rgb_sum(&centers[2]) > rgb_sum(&centers[1]) + 8);
    assert_eq!(
        centers[3], centers[1],
        "illegal baked + full-dynamic ownership must add no HGI energy"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(output_dir.join(OUTPUT_PNG), &frames);
    fs::write(
        output_dir.join(OUTPUT_REPORT),
        format!(
            "png={OUTPUT_PNG}\nwidth={}\nheight={HEIGHT}\npanels={}\nproduction_shader=post_process.wgsl+post_process_screen_space_reflection.wgsl\nsource_masks=full_dynamic:1,baked_baseline:2,dynamic_delta:4\nfull_dynamic_center_rgba={:?}\nbaked_baseline_center_rgba={:?}\nbaked_plus_dynamic_delta_center_rgba={:?}\nillegal_baked_plus_full_rejected_center_rgba={:?}\nillegal_matches_baked_baseline={}\nwgpu_product=true\n",
            WIDTH * 4 + 6,
            scenarios.iter().map(|scenario| scenario.name).collect::<Vec<_>>().join("|"),
            centers[0],
            centers[1],
            centers[2],
            centers[3],
            centers[3] == centers[1],
        ),
    )
    .unwrap();
}

fn render_scenario(device: &wgpu::Device, queue: &wgpu::Queue, scenario: Scenario) -> Vec<u8> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hybrid-gi-m4-source-ledger-post-process"),
        source: wgpu::ShaderSource::Wgsl(POST_PROCESS_SHADER.into()),
    });
    let bind_group_layout = create_post_process_bind_group_layout(device);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hybrid-gi-m4-source-ledger-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("hybrid-gi-m4-source-ledger-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
        }),
        multiview_mask: None,
        cache: None,
    });

    let params = create_buffer(
        device,
        "hybrid-gi-m4-post-params",
        &[post_process_params(scenario)],
        wgpu::BufferUsages::UNIFORM,
    );
    let cluster = create_buffer(
        device,
        "hybrid-gi-m4-cluster",
        &[[0.0_f32; 4]],
        wgpu::BufferUsages::STORAGE,
    );
    let reflection = create_buffer(
        device,
        "hybrid-gi-m4-reflection",
        &[[0.0_f32; 8]],
        wgpu::BufferUsages::STORAGE,
    );
    let probe = create_buffer(
        device,
        "hybrid-gi-m4-probe",
        &[GpuHybridGiProbe {
            screen_uv_and_radius: [0.5, 0.5, 1.5, 1.0],
            irradiance_and_intensity: [0.85, 0.22, 0.08, 1.0],
            hierarchy_irradiance_rgb_and_weight: [0.0; 4],
            hierarchy_rt_lighting_rgb_and_weight: [0.0; 4],
            temporal_signature_and_source: [
                0.25,
                1.0,
                scenario.probe_source_mask as f32,
                scenario.probe_dynamic_weight,
            ],
        }],
        wgpu::BufferUsages::STORAGE,
    );
    let trace = create_buffer(
        device,
        "hybrid-gi-m4-trace",
        &[[0.0_f32; 12]],
        wgpu::BufferUsages::STORAGE,
    );
    let exposure = create_buffer(
        device,
        "hybrid-gi-m4-exposure",
        &[[1.0_f32, 0.0, 0.0, 0.0]],
        wgpu::BufferUsages::STORAGE,
    );

    let scene = create_color_texture(device, queue, "hybrid-gi-m4-scene", scenario.scene_rgba);
    let neutral = create_color_texture(device, queue, "hybrid-gi-m4-neutral", [0, 0, 0, 255]);
    let scene_view = scene.create_view(&Default::default());
    let neutral_view = neutral.create_view(&Default::default());
    let depth = create_depth_texture(device);
    clear_depth(device, queue, &depth);
    let depth_view = depth.create_view(&Default::default());
    let lut3d = create_lut_3d(device, queue);
    let lut3d_view = lut3d.create_view(&Default::default());
    let filtering_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("hybrid-gi-m4-filtering-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    let non_filtering_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("hybrid-gi-m4-non-filtering-sampler"),
        ..Default::default()
    });

    let output = create_output_texture(device, "hybrid-gi-m4-output");
    let gi_output = create_output_texture(device, "hybrid-gi-m4-gi-output");
    let output_view = output.create_view(&Default::default());
    let gi_output_view = gi_output.create_view(&Default::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("hybrid-gi-m4-source-ledger-bind-group"),
        layout: &bind_group_layout,
        entries: &[
            texture_entry(0, &scene_view),
            texture_entry(1, &neutral_view),
            texture_entry(2, &neutral_view),
            texture_entry(3, &neutral_view),
            buffer_entry(4, &params),
            buffer_entry(5, &cluster),
            buffer_entry(6, &reflection),
            buffer_entry(7, &probe),
            buffer_entry(8, &trace),
            texture_entry(9, &neutral_view),
            texture_entry(10, &neutral_view),
            texture_entry(11, &depth_view),
            texture_entry(12, &lut3d_view),
            wgpu::BindGroupEntry {
                binding: 13,
                resource: wgpu::BindingResource::Sampler(&filtering_sampler),
            },
            texture_entry(14, &neutral_view),
            wgpu::BindGroupEntry {
                binding: 15,
                resource: wgpu::BindingResource::Sampler(&non_filtering_sampler),
            },
            texture_entry(16, &neutral_view),
            texture_entry(17, &neutral_view),
            texture_entry(18, &neutral_view),
            texture_entry(19, &neutral_view),
            texture_entry(20, &neutral_view),
            texture_entry(21, &neutral_view),
            texture_entry(22, &neutral_view),
            texture_entry(23, &neutral_view),
            texture_entry(24, &neutral_view),
            texture_entry(25, &neutral_view),
            texture_entry(26, &neutral_view),
            texture_entry(27, &neutral_view),
            buffer_entry(28, &exposure),
        ],
    });

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("hybrid-gi-m4-readback"),
        size: u64::from(BYTES_PER_ROW * HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hybrid-gi-m4-source-ledger-pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &gi_output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(BYTES_PER_ROW),
                rows_per_image: Some(HEIGHT),
            },
        },
        extent(),
    );
    queue.submit([encoder.finish()]);
    read_rgba8(device, &readback)
}

fn post_process_params(scenario: Scenario) -> PostProcessParams {
    PostProcessParams {
        viewport_and_clusters: [WIDTH, HEIGHT, 0, 0],
        cluster_dimensions: [1, 1, 0, 0],
        feature_flags: [0; 4],
        lighting_flags: [0; 4],
        hybrid_gi_counts: [1, 0, 0, 0],
        hybrid_gi_source_ledger: [
            scenario.global_source_mask,
            scenario.global_baked_weight,
            255,
            1,
        ],
        anti_alias: [0; 4],
        blends: [0.0; 4],
        grading: [1.0; 4],
        tint_and_probe: [1.0, 1.0, 1.0, 0.0],
        hybrid_gi_color_and_intensity: [1.0, 1.0, 1.0, 0.45],
        baked_color_and_intensity: [0.0; 4],
        effect_flags: [0; 4],
        effect_tonemap_lut: [0.0; 4],
        effect_blur_dof: [0.0; 4],
        effect_dof_lens: [0.0; 4],
        effect_vignette_grain: [0.0; 4],
        effect_chromatic_fog: [0.0; 4],
        effect_fog_color: [0.0; 4],
        effect_dither_ssr: [0.0; 4],
        effect_ssr_limits: [0.0; 4],
        effect_depth: [0.1, 100.0, 1.0 / 99.9, 1.0],
        effect_projection: [1.0, 1.0, 1.0, 1.0],
        effect_view_x: [1.0, 0.0, 0.0, 0.0],
        effect_view_y: [0.0, 1.0, 0.0, 0.0],
        effect_view_z: [0.0, 0.0, 1.0, 0.0],
        effect_motion_blur: [0.0; 4],
    }
}

fn create_post_process_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let mut entries = Vec::with_capacity(29);
    for binding in [0, 1, 2, 3] {
        entries.push(texture_layout_entry(
            binding,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureSampleType::Float { filterable: false },
        ));
    }
    entries.push(buffer_layout_entry(4, wgpu::BufferBindingType::Uniform));
    for binding in 5..=8 {
        entries.push(buffer_layout_entry(
            binding,
            wgpu::BufferBindingType::Storage { read_only: true },
        ));
    }
    for binding in 9..=10 {
        entries.push(texture_layout_entry(
            binding,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureSampleType::Float { filterable: false },
        ));
    }
    entries.push(texture_layout_entry(
        11,
        wgpu::TextureViewDimension::D2,
        wgpu::TextureSampleType::Depth,
    ));
    entries.push(texture_layout_entry(
        12,
        wgpu::TextureViewDimension::D3,
        wgpu::TextureSampleType::Float { filterable: true },
    ));
    entries.push(sampler_layout_entry(
        13,
        wgpu::SamplerBindingType::Filtering,
    ));
    entries.push(texture_layout_entry(
        14,
        wgpu::TextureViewDimension::D2,
        wgpu::TextureSampleType::Float { filterable: false },
    ));
    entries.push(sampler_layout_entry(
        15,
        wgpu::SamplerBindingType::NonFiltering,
    ));
    for binding in 16..=27 {
        entries.push(texture_layout_entry(
            binding,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureSampleType::Float { filterable: false },
        ));
    }
    entries.push(buffer_layout_entry(
        28,
        wgpu::BufferBindingType::Storage { read_only: true },
    ));
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("hybrid-gi-m4-source-ledger-bind-group-layout"),
        entries: &entries,
    })
}

fn texture_layout_entry(
    binding: u32,
    view_dimension: wgpu::TextureViewDimension,
    sample_type: wgpu::TextureSampleType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension,
            sample_type,
        },
        count: None,
    }
}

fn buffer_layout_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn sampler_layout_entry(binding: u32, ty: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(ty),
        count: None,
    }
}

fn create_color_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    rgba: [u8; 4],
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut bytes = vec![0_u8; (WIDTH * HEIGHT * 4) as usize];
    for pixel in bytes.chunks_exact_mut(4) {
        pixel.copy_from_slice(&rgba);
    }
    queue.write_texture(
        texture.as_image_copy(),
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(WIDTH * 4),
            rows_per_image: Some(HEIGHT),
        },
        extent(),
    );
    texture
}

fn create_output_texture(device: &wgpu::Device, label: &str) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn create_depth_texture(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hybrid-gi-m4-depth"),
        size: extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

fn clear_depth(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) {
    let view = texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("hybrid-gi-m4-clear-depth"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    queue.submit([encoder.finish()]);
}

fn create_lut_3d(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hybrid-gi-m4-lut3d"),
        size: wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 2,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &[0_u8; 32],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8),
            rows_per_image: Some(2),
        },
        wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 2,
        },
    );
    texture
}

fn create_buffer<T: Pod>(
    device: &wgpu::Device,
    label: &str,
    data: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(data),
        usage,
    })
}

fn texture_entry<'a>(binding: u32, view: &'a wgpu::TextureView) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

fn buffer_entry<'a>(binding: u32, buffer: &'a wgpu::Buffer) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn read_rgba8(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<u8> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let bytes = mapped.to_vec();
    drop(mapped);
    buffer.unmap();
    bytes
}

fn write_png(path: PathBuf, frames: &[Vec<u8>]) {
    const GAP: u32 = 2;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(WIDTH * 4 + GAP * 3, HEIGHT);
    for (panel, frame) in frames.iter().enumerate() {
        let origin = panel as u32 * (WIDTH + GAP);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let offset = (y * BYTES_PER_ROW + x * 4) as usize;
                image.put_pixel(
                    origin + x,
                    y,
                    Rgba(frame[offset..offset + 4].try_into().unwrap()),
                );
            }
        }
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn rgb_sum(rgba: &[u8]) -> u32 {
    rgba[..3].iter().map(|value| u32::from(*value)).sum()
}

fn extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    }
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
    if adapter.limits().max_sampled_textures_per_shader_stage < 21 {
        return None;
    }
    let required_limits = wgpu::Limits {
        max_sampled_textures_per_shader_stage: 21,
        ..wgpu::Limits::default()
    };
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-hybrid-gi-m4-source-ledger-product-device"),
        required_features: wgpu::Features::empty(),
        required_limits,
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
