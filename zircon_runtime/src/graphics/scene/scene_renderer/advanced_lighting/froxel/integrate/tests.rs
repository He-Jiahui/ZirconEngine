use super::*;

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;

use image::{ImageBuffer, ImageFormat, Rgba};
use wgpu::util::DeviceExt;

use crate::core::framework::render::{
    GpuLightData, GpuLightType, ViewportCameraSnapshot, VolumetricFogSettings,
};
use crate::core::math::{Mat4, UVec2, Vec3, Vec4};
use crate::graphics::scene::scene_renderer::lighting::light_grid_builder::{
    build_light_grid, LightGridProjection, LightGridViewInfo,
};
use crate::graphics::scene::scene_renderer::shadow::slot::{
    GpuShadowGlobals, GpuShadowSlot, GPU_SHADOW_SLOT_FLAG_VALID,
};

use super::super::light_scatter::{FroxelLightScatterPipeline, FroxelLightScatterRequest};
use super::super::media_inject::{FroxelMediaInjectPipeline, FroxelMediaInjectRequest};
use super::super::GpuFroxelTemporalReprojection;

mod temporal_product;

const TEST_GRID: [u32; 3] = [16, 8, 8];
const TEST_OUTPUT: [u32; 2] = [16, 8];
const READBACK_BYTES_PER_ROW: u32 = 256;
const PRODUCT_PNG: &str = "plan18_volumetric_light_scatter_integrate_shadow_wgpu_20260711.png";
const PRODUCT_REPORT: &str = "plan18_volumetric_light_scatter_integrate_shadow_wgpu_20260711.txt";
const VOLUMETRIC_APPLY_INCLUDE: &str =
    include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl");
const VOLUMETRIC_APPLY_TEST_SHADER: &str = concat!(
    r#"
struct SceneUniform {
    inverse_view_proj: mat4x4<f32>,
    camera_world_position: vec4<f32>,
    camera_view_direction: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
"#,
    include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl"),
    r#"
@group(0) @binding(1) var scene_color: texture_2d<f32>;
@group(0) @binding(2) var output_color: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let dimensions = textureDimensions(scene_color);
    if (any(invocation.xy >= dimensions)) {
        return;
    }
    let source = textureLoad(scene_color, vec2<i32>(invocation.xy), 0);
    let fragment_position = vec2<f32>(invocation.xy) + vec2<f32>(0.5);
    textureStore(output_color, invocation.xy, vec4<f32>(zr_volumetric_apply(source.rgb, fragment_position, 1.0), source.a));
}
"#,
);

#[test]
fn render_volumetric_integrate_upload_bytes_match_uniform_abi() {
    assert_eq!(FroxelIntegratePipeline::UPLOADED_BYTES_PER_DISPATCH, 128);
}

#[test]
fn render_volumetric_integrate_shader_writes_3d_radiance_transmittance_for_shading_apply() {
    assert!(INTEGRATE_SHADER.contains("zr_froxel_step_length"));
    assert!(INTEGRATE_SHADER.contains("exp(-extinction * step_length)"));
    assert!(INTEGRATE_SHADER.contains("(1.0 - step_transmittance) / extinction"));
    assert!(INTEGRATE_SHADER.contains("transmittance * max(sample.rgb"));
    assert!(INTEGRATE_SHADER.contains("texture_storage_3d<rgba16float, write>"));
    assert!(INTEGRATE_SHADER.contains("vec4<f32>(radiance, transmittance)"));
    assert!(!INTEGRATE_SHADER.contains("scene_color"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("@group(1) @binding(25)"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("@group(1) @binding(26)"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("@group(1) @binding(27)"));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("fn zr_volumetric_apply("));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("color * zr_volumetric_transmittance("));
    assert!(VOLUMETRIC_APPLY_INCLUDE.contains("+ zr_volumetric_scattering("));

    let module = naga::front::wgsl::parse_str(INTEGRATE_SHADER)
        .expect("volumetric integrate shader must parse");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("volumetric integrate shader must validate");

    let apply_module = naga::front::wgsl::parse_str(VOLUMETRIC_APPLY_TEST_SHADER)
        .expect("volumetric apply include must compose into a shader");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&apply_module)
    .expect("volumetric apply include must validate in a shading consumer");
}

#[test]
fn render_volumetric_light_scatter_integrate_consumes_light_grid_and_shadow_atlas() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let result = run_volumetric_chain(&device, &queue);
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
#[ignore]
fn export_volumetric_light_scatter_integrate_shadow_wgpu_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping volumetric light-shaft product because no adapter is available");
        return;
    };
    let result = run_volumetric_chain(&device, &queue);
    assert!(result.right_average[0] > result.left_average[0] + 0.2);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_output_png(output_dir.join(PRODUCT_PNG), &result.texels);
    fs::write(
        output_dir.join(PRODUCT_REPORT),
        format!(
            "png={PRODUCT_PNG}\nwidth=256\nheight=128\ngpu_froxel_dimensions=16x8x8\nintegrated_product=rgba16float_3d_radiance_transmittance\nshading_apply_bindings=group1_26_texture3d_group1_27_sampler\noutput_dimensions=16x8\nmedia_dispatch={},{},{}\nlight_scatter_dispatch={},{},{}\nintegrate_dispatch={},{}\napply_dispatch={},{}\nlight_grid_words_per_tile=1\nlight_grid_selected_directional_lights=1\nshadow_atlas_format=depth32float\nshadow_atlas_compare=greater_equal\nshadow_projection=left_half_shadowed_right_half_outside_slot\nphase_g=0\nstep_length=0.25\nleft_shadowed_average_rgb={:.6},{:.6},{:.6}\nright_unshadowed_average_rgb={:.6},{:.6},{:.6}\nreference=UE_VolumetricFog_LightScatteringCS_plus_front_to_back_integrate\n",
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

fn run_volumetric_chain(device: &wgpu::Device, queue: &wgpu::Queue) -> VolumetricChainResult {
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
        create_shadow_resources(device);

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

fn test_froxel_view() -> FroxelViewReconstruction {
    FroxelViewReconstruction::perspective(
        Mat4::perspective_rh(90.0_f32.to_radians(), 2.0, 0.1, 20.0).inverse(),
        Vec3::ZERO,
        Vec3::NEG_Z,
    )
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
        source: wgpu::ShaderSource::Wgsl(VOLUMETRIC_APPLY_TEST_SHADER.into()),
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

struct LightingResources {
    light_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    zbins_buffer: wgpu::Buffer,
    tile_masks_buffer: wgpu::Buffer,
}

fn create_lighting_resources(device: &wgpu::Device) -> LightingResources {
    let light = GpuLightData {
        color_intensity: [1.0, 0.82, 0.45, 80.0],
        direction_type: [0.0, 0.0, -1.0, GpuLightType::Directional.as_f32_bits()],
        shadow_slot_layer: [0, u32::MAX, 1, 1],
        shadow_params: [1.0, 0.0, 0.0, 1.0],
        cookie_misc: [0, 0, 1, 0],
        ..GpuLightData::default()
    };
    let projection = Mat4::perspective_rh(90.0_f32.to_radians(), 2.0, 0.1, 20.0);
    let light_grid = build_light_grid(
        &[light],
        &LightGridViewInfo {
            viewport_size: UVec2::from_array(TEST_OUTPUT),
            world_to_view: Mat4::IDENTITY,
            view_to_clip: projection,
            projection: LightGridProjection::Perspective,
            z_near: 0.1,
            z_far: 20.0,
        },
    );
    assert_eq!(light_grid.stats.light_count, 1);
    assert_eq!(light_grid.stats.non_empty_tile_count, 2);
    assert_eq!(
        light_grid.stats.non_empty_zbin_count,
        light_grid.stats.zbin_count
    );
    let light_buffer = create_buffer(
        device,
        "volumetric-chain-lights",
        &[light],
        wgpu::BufferUsages::STORAGE,
    );
    let params_buffer = create_buffer(
        device,
        "volumetric-chain-light-grid-params",
        &[light_grid.params],
        wgpu::BufferUsages::UNIFORM,
    );
    let zbins_buffer = create_buffer(
        device,
        "volumetric-chain-light-zbins",
        &light_grid.zbins,
        wgpu::BufferUsages::STORAGE,
    );
    let tile_masks_buffer = create_buffer(
        device,
        "volumetric-chain-light-tile-masks",
        &light_grid.tile_masks,
        wgpu::BufferUsages::STORAGE,
    );
    LightingResources {
        light_buffer,
        params_buffer,
        zbins_buffer,
        tile_masks_buffer,
    }
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
        label: Some("volumetric-chain-shadow-atlas"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("volumetric-chain-shadow-sampler"),
        compare: Some(wgpu::CompareFunction::GreaterEqual),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let left_half_projection = Mat4::from_cols(
        Vec4::new(2.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::ZERO,
        Vec4::new(1.0, 0.0, 0.5, 1.0),
    );
    let slot = GpuShadowSlot {
        view_proj: left_half_projection.to_cols_array_2d(),
        atlas_scale_bias: [1.0, 1.0, 0.0, 0.0],
        params: [
            0.0,
            0.0,
            1.0 / 16.0,
            f32::from_bits(GPU_SHADOW_SLOT_FLAG_VALID),
        ],
    };
    let slots = create_buffer(
        device,
        "volumetric-chain-shadow-slots",
        &[slot],
        wgpu::BufferUsages::STORAGE,
    );
    let globals = create_buffer(
        device,
        "volumetric-chain-shadow-globals",
        &[GpuShadowGlobals::disabled(16, 16)],
        wgpu::BufferUsages::UNIFORM,
    );
    (texture, view, sampler, slots, globals)
}

fn clear_shadow_atlas(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("volumetric-chain-clear-shadow-atlas"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view,
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
}

fn create_buffer<T: bytemuck::Pod>(
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

fn create_rgba16f_3d_texture(device: &wgpu::Device, label: &str) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: TEST_GRID[0],
            height: TEST_GRID[1],
            depth_or_array_layers: TEST_GRID[2],
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn d3_view_descriptor(label: &str) -> wgpu::TextureViewDescriptor<'_> {
    wgpu::TextureViewDescriptor {
        label: Some(label),
        dimension: Some(wgpu::TextureViewDimension::D3),
        ..Default::default()
    }
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

mod support;

use support::{f16_bits_to_f32, render_test_output_dir, test_device, write_output_png};
