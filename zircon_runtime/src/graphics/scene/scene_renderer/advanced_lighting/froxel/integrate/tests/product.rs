use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;

use image::{ImageBuffer, ImageFormat, Rgba};
use wgpu::util::DeviceExt;

use crate::core::framework::render::{
    FroxelGridParams, ViewportCameraSnapshot, VolumetricFogSettings,
};
use crate::core::math::{Mat4, UVec2, Vec3, Vec4};

use super::super::super::light_scatter::{FroxelLightScatterPipeline, FroxelLightScatterRequest};
use super::super::super::media_inject::{FroxelMediaInjectPipeline, FroxelMediaInjectRequest};
use super::super::super::GpuFroxelTemporalReprojection;
use super::super::{FroxelIntegratePipeline, FroxelIntegrateRequest};
use super::fixture::{
    clear_shadow_atlas, create_lighting_resources, create_rgba16f_3d_texture,
    create_shadow_resources, d3_view_descriptor, test_froxel_view, write_shadow_occluder_depth,
    READBACK_BYTES_PER_ROW, TEST_GRID, TEST_OUTPUT, TEST_SHADOWED_RECEIVER_DEPTH,
    TEST_SHADOW_OCCLUDER_DEPTH,
};
use super::support::{f16_bits_to_f32, render_test_output_dir, test_device, write_output_png};

const PRODUCT_PNG: &str = "plan18_volumetric_light_scatter_integrate_shadow_wgpu_20260711.png";
const PRODUCT_REPORT: &str = "plan18_volumetric_light_scatter_integrate_shadow_wgpu_20260711.txt";
#[test]
fn render_volumetric_light_scatter_integrate_consumes_light_grid_and_shadow_atlas() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let result = run_volumetric_chain(&device, &queue, TEST_SHADOWED_RECEIVER_DEPTH);
    assert_eq!(result.media_dispatch, [4, 2, 2]);
    assert_eq!(result.scatter_dispatch, [4, 2, 2]);
    assert_eq!(result.integrate_dispatch, [2, 1]);
    assert_eq!(result.apply_dispatch, [2, 1]);
    assert!(
        result.right_average[0] > result.left_average[0] + 0.2,
        "unshadowed half should contain the directional light shaft: left={:?}, right={:?}",
        result.left_average,
        result.right_average,
    );
    assert!(
        result.right_average[1] > result.left_average[1] + 0.15,
        "light-grid selected scattering should survive integration: left={:?}, right={:?}",
        result.left_average,
        result.right_average,
    );
}

#[test]
fn render_volumetric_shadow_equality_depth_remains_visible_with_less_equal_compare() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let shadowed = run_volumetric_chain(&device, &queue, TEST_SHADOWED_RECEIVER_DEPTH);
    let equality = run_volumetric_chain(&device, &queue, TEST_SHADOW_OCCLUDER_DEPTH);

    assert!(
        equality.left_average[0] > shadowed.left_average[0] + 0.2,
        "an equality-depth receiver must remain lit by LessEqual: shadowed={:?}, equality={:?}",
        shadowed.left_average,
        equality.left_average,
    );
    assert!(
        (equality.left_average[0] - shadowed.right_average[0]).abs() < 0.05,
        "equality-depth receiver should match the unshadowed directional shaft: equality={:?}, unshadowed={:?}",
        equality.left_average,
        shadowed.right_average,
    );
}

#[test]
#[ignore]
fn export_volumetric_light_scatter_integrate_shadow_wgpu_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping volumetric light-shaft product because no adapter is available");
        return;
    };
    let result = run_volumetric_chain(&device, &queue, TEST_SHADOWED_RECEIVER_DEPTH);
    assert!(result.right_average[0] > result.left_average[0] + 0.2);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_output_png(output_dir.join(PRODUCT_PNG), &result.texels);
    fs::write(
        output_dir.join(PRODUCT_REPORT),
        format!(
            "png={PRODUCT_PNG}\nwidth=256\nheight=128\ngpu_froxel_dimensions=16x8x8\nintegrated_product=rgba16float_3d_radiance_transmittance\nshading_apply_bindings=group1_26_texture3d_group1_27_sampler\noutput_dimensions=16x8\nmedia_dispatch={},{},{}\nlight_scatter_dispatch={},{},{}\nintegrate_dispatch={},{}\napply_dispatch={},{}\nlight_grid_words_per_tile=1\nlight_grid_selected_directional_lights=1\nshadow_atlas_format=depth32float\nshadow_atlas_compare=less_equal\nshadow_projection=left_half_shadowed_right_half_outside_slot\nphase_g=0\nstep_length=0.25\nleft_shadowed_average_rgb={:.6},{:.6},{:.6}\nright_unshadowed_average_rgb={:.6},{:.6},{:.6}\nreference=UE_VolumetricFog_LightScatteringCS_plus_front_to_back_integrate\n",
            result.media_dispatch[0],
            result.media_dispatch[1],
            result.media_dispatch[2],
            result.scatter_dispatch[0],
            result.scatter_dispatch[1],
            result.scatter_dispatch[2],
            result.integrate_dispatch[0],
            result.integrate_dispatch[1],
            result.apply_dispatch[0],
            result.apply_dispatch[1],
            result.left_average[0],
            result.left_average[1],
            result.left_average[2],
            result.right_average[0],
            result.right_average[1],
            result.right_average[2],
        ),
    )
    .unwrap();
}

#[derive(Debug)]
struct VolumetricChainResult {
    texels: Vec<[f32; 4]>,
    media_dispatch: [u32; 3],
    scatter_dispatch: [u32; 3],
    integrate_dispatch: [u32; 2],
    apply_dispatch: [u32; 2],
    left_average: [f32; 3],
    right_average: [f32; 3],
}

fn run_volumetric_chain(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shadow_receiver_depth: f32,
) -> VolumetricChainResult {
    let media = create_rgba16f_3d_texture(device, "volumetric-chain-media");
    let media_view = media.create_view(&d3_view_descriptor("volumetric-chain-media-view"));
    let scattering = create_rgba16f_3d_texture(device, "volumetric-chain-scattering");
    let scattering_view =
        scattering.create_view(&d3_view_descriptor("volumetric-chain-scattering-view"));
    let integrated = create_rgba16f_3d_texture(device, "volumetric-chain-integrated");
    let integrated_view =
        integrated.create_view(&d3_view_descriptor("volumetric-chain-integrated-view"));
    let scene_color = create_scene_color(device, queue);
    let scene_color_view = scene_color.create_view(&wgpu::TextureViewDescriptor::default());
    let output = create_output_texture(device);
    let output_view = output.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("volumetric-chain-readback"),
        size: u64::from(READBACK_BYTES_PER_ROW * TEST_OUTPUT[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let lighting = create_lighting_resources(device);
    let (shadow_atlas, shadow_atlas_view, shadow_sampler, shadow_slots, shadow_globals) =
        create_shadow_resources(device, shadow_receiver_depth);

    let grid = FroxelGridParams {
        dimensions: TEST_GRID,
        near_depth: 0.1,
        far_depth: 20.0,
        depth_distribution_exp: 2.0,
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("volumetric-chain-encoder"),
    });
    clear_shadow_atlas(&mut encoder, &shadow_atlas_view);
    write_shadow_occluder_depth(&mut encoder, &shadow_atlas_view);

    let media_pipeline = FroxelMediaInjectPipeline::new(device);
    let media_dispatch = media_pipeline
        .encode(
            device,
            &mut encoder,
            &media_view,
            FroxelMediaInjectRequest {
                settings: VolumetricFogSettings {
                    density: 0.12,
                    albedo: Vec3::new(1.0, 0.72, 0.28),
                    phase_g: 0.0,
                    height_falloff: 0.0,
                    scattering_intensity: 1.0,
                    depth_distribution_exp: 2.0,
                    temporal: false,
                },
                grid,
                view: test_froxel_view(),
                local_volumes: &[],
                include_local_volumes: false,
            },
        )
        .unwrap();

    let scatter_pipeline = FroxelLightScatterPipeline::new(device);
    let scatter_dispatch = scatter_pipeline
        .encode(
            device,
            &mut encoder,
            FroxelLightScatterRequest {
                grid,
                view: test_froxel_view(),
                phase_g: 0.0,
                ambient_radiance: Vec3::ZERO,
                viewport_size: TEST_OUTPUT,
                media_view: &media_view,
                history_view: &media_view,
                temporal: GpuFroxelTemporalReprojection::new(
                    &ViewportCameraSnapshot::default(),
                    None,
                    UVec2::from_array(TEST_OUTPUT),
                    grid,
                    false,
                    false,
                ),
                light_buffer: &lighting.light_buffer,
                light_count: 1,
                light_grid_params_buffer: &lighting.params_buffer,
                light_zbins_buffer: &lighting.zbins_buffer,
                light_tile_masks_buffer: &lighting.tile_masks_buffer,
                shadow_atlas_view: &shadow_atlas_view,
                shadow_sampler: &shadow_sampler,
                shadow_slots_buffer: &shadow_slots,
                shadow_globals_buffer: &shadow_globals,
                output_view: &scattering_view,
            },
        )
        .unwrap();

    let integrate_pipeline = FroxelIntegratePipeline::new(device);
    let integrate_dispatch = integrate_pipeline
        .encode(
            device,
            &mut encoder,
            FroxelIntegrateRequest {
                grid,
                view: test_froxel_view(),
                scattering_view: &scattering_view,
                output_view: &integrated_view,
            },
        )
        .unwrap();
    let apply_dispatch = encode_volumetric_apply(
        device,
        &mut encoder,
        &scene_color_view,
        &integrated_view,
        &output_view,
    );
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(READBACK_BYTES_PER_ROW),
                rows_per_image: Some(TEST_OUTPUT[1]),
            },
        },
        wgpu::Extent3d {
            width: TEST_OUTPUT[0],
            height: TEST_OUTPUT[1],
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let texels = read_rgba16f_2d(device, &readback);
    let (left_average, right_average) = split_average(&texels);
    drop(shadow_atlas);
    VolumetricChainResult {
        texels,
        media_dispatch,
        scatter_dispatch,
        integrate_dispatch,
        apply_dispatch,
        left_average,
        right_average,
    }
}

fn encode_volumetric_apply(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    scene_color_view: &wgpu::TextureView,
    integrated_view: &wgpu::TextureView,
    output_view: &wgpu::TextureView,
) -> [u32; 2] {
    let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("volumetric-apply-test-scene-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });
    let volumetric_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("volumetric-apply-test-volume-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 25,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 26,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 27,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("volumetric-apply-test-shader"),
        source: wgpu::ShaderSource::Wgsl(super::VOLUMETRIC_APPLY_TEST_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("volumetric-apply-test-pipeline-layout"),
        bind_group_layouts: &[Some(&scene_layout), Some(&volumetric_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("volumetric-apply-test-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let scene_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("volumetric-apply-test-scene-uniform"),
        contents: bytemuck::cast_slice(&[
            1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0,
        ]),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let scene_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("volumetric-apply-test-scene-group"),
        layout: &scene_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(scene_color_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(output_view),
            },
        ],
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("volumetric-apply-test-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    });
    let volumetric_params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("volumetric-apply-test-params"),
        contents: bytemuck::cast_slice(&[
            0.1_f32,
            1.0,
            2.0,
            1.0,
            0.0,
            0.0,
            1.0 / TEST_OUTPUT[0] as f32,
            1.0 / TEST_OUTPUT[1] as f32,
        ]),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let volumetric_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("volumetric-apply-test-volume-group"),
        layout: &volumetric_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 25,
                resource: volumetric_params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 26,
                resource: wgpu::BindingResource::TextureView(integrated_view),
            },
            wgpu::BindGroupEntry {
                binding: 27,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let dispatch = [TEST_OUTPUT[0].div_ceil(8), TEST_OUTPUT[1].div_ceil(8)];
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("VolumetricApplyTestPass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &scene_group, &[]);
    pass.set_bind_group(1, &volumetric_group, &[]);
    pass.dispatch_workgroups(dispatch[0], dispatch[1], 1);
    dispatch
}

fn create_scene_color(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("volumetric-chain-scene-color"),
        size: wgpu::Extent3d {
            width: TEST_OUTPUT[0],
            height: TEST_OUTPUT[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texels = vec![12_u8, 14, 20, 255]
        .into_iter()
        .cycle()
        .take((TEST_OUTPUT[0] * TEST_OUTPUT[1] * 4) as usize)
        .collect::<Vec<_>>();
    queue.write_texture(
        texture.as_image_copy(),
        &texels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TEST_OUTPUT[0] * 4),
            rows_per_image: Some(TEST_OUTPUT[1]),
        },
        wgpu::Extent3d {
            width: TEST_OUTPUT[0],
            height: TEST_OUTPUT[1],
            depth_or_array_layers: 1,
        },
    );
    texture
}

fn create_output_texture(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("volumetric-chain-output"),
        size: wgpu::Extent3d {
            width: TEST_OUTPUT[0],
            height: TEST_OUTPUT[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn read_rgba16f_2d(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<[f32; 4]> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll should complete volumetric chain readback");
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let mut texels = Vec::with_capacity((TEST_OUTPUT[0] * TEST_OUTPUT[1]) as usize);
    for y in 0..TEST_OUTPUT[1] as usize {
        let row_offset = y * READBACK_BYTES_PER_ROW as usize;
        for x in 0..TEST_OUTPUT[0] as usize {
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
    drop(mapped);
    buffer.unmap();
    texels
}

fn split_average(texels: &[[f32; 4]]) -> ([f32; 3], [f32; 3]) {
    let mut left = [0.0; 3];
    let mut right = [0.0; 3];
    let mut counts = [0_u32; 2];
    for y in 0..TEST_OUTPUT[1] {
        for x in 0..TEST_OUTPUT[0] {
            let sample = texels[(y * TEST_OUTPUT[0] + x) as usize];
            let side = usize::from(x >= TEST_OUTPUT[0] / 2);
            let sum = if side == 0 { &mut left } else { &mut right };
            for channel in 0..3 {
                sum[channel] += sample[channel];
            }
            counts[side] += 1;
        }
    }
    for channel in 0..3 {
        left[channel] /= counts[0] as f32;
        right[channel] /= counts[1] as f32;
    }
    (left, right)
}
