use crate::core::framework::render::{
    CubemapFace, IblBakeArtifactContents, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT, SourceCubemapPrefilterQuality,
    build_source_cubemap_from_source_mips_with_quality, cubemap_texel_direction,
    decode_rgba16f_texels, encode_rgba16f_texels, source_cubemap_face_mip_offset,
    source_cubemap_mip_size, source_cubemap_sample_count,
};
use crate::graphics::backend::RenderBackend;

use super::super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::super::super::ibl_bake_wgpu_binding::{
    IblBakeWgpuBindGroupLayouts, IblBakeWgpuOutputBindingResource, create_ibl_bake_wgpu_bind_group,
    create_ibl_bake_wgpu_params_buffer, create_ibl_bake_wgpu_source_sampler,
};
use super::super::super::ibl_bake_wgpu_command_plan::ibl_bake_wgpu_command_plan_for_request;
use super::{
    command_for_kind, create_ibl_bake_wgpu_compute_pipeline, create_storage_output_texture,
    encode_ibl_bake_wgpu_compute_dispatch, read_rgba16float_mip_faces, request,
    storage_texture_descriptor,
};

const SOURCE_FACE_SIZE: u32 = 16;
const SOURCE_MIP_COUNT: u32 = 5;
const PMREM_TEXEL_TOLERANCE: f32 = 0.006;
const PMREM_CONSTANT_ABSOLUTE_TOLERANCE: f32 = 0.001;
const PMREM_CONSTANT_RELATIVE_TOLERANCE: f32 = 0.001;

#[test]
fn render_env_prefilter_cpu_gpu_match_16() {
    let backend = RenderBackend::new_offscreen()
        .expect("PMREM CPU/GPU parity requires an offscreen WGPU backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let source_texels = rgba16f_quantized_direction_source();
    let cpu = build_source_cubemap_from_source_mips_with_quality(
        SOURCE_FACE_SIZE,
        SOURCE_MIP_COUNT,
        source_texels.clone(),
        SourceCubemapPrefilterQuality::Normal,
    );

    let output = dispatch_pmrem(&backend, &source_texels);

    let mut worst = (
        0.0_f32,
        0_u32,
        CubemapFace::PositiveX,
        0_u32,
        0_u32,
        0_usize,
    );
    for mip_level in 0..SOURCE_CUBEMAP_PMREM_MIP_COUNT {
        let mip_size = source_cubemap_mip_size(SOURCE_CUBEMAP_PMREM_FACE_SIZE, mip_level);
        let gpu = decode_rgba16f_texels(&read_rgba16float_mip_faces(
            device, queue, &output, mip_level, mip_size, 6,
        ));
        for face in CubemapFace::ALL {
            let cpu_offset = source_cubemap_face_mip_offset(
                SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                SOURCE_CUBEMAP_PMREM_MIP_COUNT,
                face,
                mip_level,
            );
            let gpu_offset = face.index() * mip_size as usize * mip_size as usize;
            for y in 0..mip_size {
                for x in 0..mip_size {
                    let local = y as usize * mip_size as usize + x as usize;
                    for channel in 0..3 {
                        let gpu_value = gpu[gpu_offset + local][channel];
                        let cpu_value = cpu.pmrem_texels()[cpu_offset + local][channel];
                        assert!(
                            gpu_value.is_finite() && cpu_value.is_finite(),
                            "PMREM parity requires finite texels: mip={mip_level} face={face:?} x={x} y={y} channel={channel} gpu={gpu_value} cpu={cpu_value}"
                        );
                        let error = (gpu_value - cpu_value).abs();
                        if error > worst.0 {
                            worst = (error, mip_level, face, x, y, channel);
                        }
                    }
                }
            }
        }
    }

    assert!(
        worst.0 <= PMREM_TEXEL_TOLERANCE,
        "CPU/GPU PMREM mismatch: max_error={} tolerance={} mip={} face={:?} x={} y={} channel={}",
        worst.0,
        PMREM_TEXEL_TOLERANCE,
        worst.1,
        worst.2,
        worst.3,
        worst.4,
        worst.5,
    );
}

#[test]
fn render_env_prefilter_constant_env_is_identity() {
    let backend = RenderBackend::new_offscreen()
        .expect("PMREM constant-environment identity requires an offscreen WGPU backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let constant = decode_rgba16f_texels(&encode_rgba16f_texels(&[[0.25, 1.5, 3.0, 1.0]]))[0];
    let source_texels =
        vec![constant; source_cubemap_sample_count(SOURCE_FACE_SIZE, SOURCE_MIP_COUNT)];
    let output = dispatch_pmrem(&backend, &source_texels);

    let mut worst = (
        0.0_f32,
        0.0_f32,
        0.0_f32,
        0.0_f32,
        0_u32,
        CubemapFace::PositiveX,
        0_u32,
        0_u32,
        0_usize,
    );
    for mip_level in 0..SOURCE_CUBEMAP_PMREM_MIP_COUNT {
        let mip_size = source_cubemap_mip_size(SOURCE_CUBEMAP_PMREM_FACE_SIZE, mip_level);
        let gpu = decode_rgba16f_texels(&read_rgba16float_mip_faces(
            device, queue, &output, mip_level, mip_size, 6,
        ));
        for face in CubemapFace::ALL {
            let face_offset = face.index() * mip_size as usize * mip_size as usize;
            for y in 0..mip_size {
                for x in 0..mip_size {
                    let local = y as usize * mip_size as usize + x as usize;
                    for channel in 0..4 {
                        let gpu_value = gpu[face_offset + local][channel];
                        assert!(
                            gpu_value.is_finite(),
                            "constant PMREM requires finite texels: mip={mip_level} face={face:?} x={x} y={y} channel={channel} value={gpu_value}"
                        );
                        let expected = constant[channel];
                        let tolerance = PMREM_CONSTANT_ABSOLUTE_TOLERANCE
                            + PMREM_CONSTANT_RELATIVE_TOLERANCE * expected.abs();
                        let error = (gpu_value - expected).abs();
                        let normalized_error = error / tolerance;
                        if normalized_error > worst.0 {
                            worst = (
                                normalized_error,
                                error,
                                tolerance,
                                expected,
                                mip_level,
                                face,
                                x,
                                y,
                                channel,
                            );
                        }
                    }
                }
            }
        }
    }

    assert!(
        worst.0 <= 1.0,
        "constant PMREM must preserve RGBA: normalized_error={} error={} tolerance={} expected={} mip={} face={:?} x={} y={} channel={}",
        worst.0,
        worst.1,
        worst.2,
        worst.3,
        worst.4,
        worst.5,
        worst.6,
        worst.7,
        worst.8,
    );
}

fn dispatch_pmrem(backend: &RenderBackend, source_texels: &[[f32; 4]]) -> wgpu::Texture {
    let device = &backend.device;
    let queue = &backend.queue;
    let source_texture = create_rgba16f_source_cubemap(device, queue, source_texels);
    let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("ibl-bake-reference-source-view"),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(SOURCE_MIP_COUNT),
        base_array_layer: 0,
        array_layer_count: Some(6),
    });
    let output = create_storage_output_texture(
        device,
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let request = request(
        SOURCE_FACE_SIZE,
        SOURCE_MIP_COUNT,
        IblBakeArtifactContents::PMREM,
    );
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let first = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem { mip_level: 0 },
    );
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let pipeline = create_ibl_bake_wgpu_compute_pipeline(
        device,
        first,
        layouts.layout(first.bind_group_layout_kind),
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-reference-pmrem-encoder"),
    });

    for mip_level in 0..SOURCE_CUBEMAP_PMREM_MIP_COUNT {
        let command = command_for_kind(
            &plan.commands,
            IblBakeComputeKernelKind::Pmrem { mip_level },
        );
        let output_view = output.create_view(&storage_texture_descriptor(command));
        let params = create_ibl_bake_wgpu_params_buffer(device, command);
        let bind_group = create_ibl_bake_wgpu_bind_group(
            device,
            &layouts,
            command,
            &params,
            &source_view,
            &sampler,
            IblBakeWgpuOutputBindingResource::StorageTexture2DArray(&output_view),
        )
        .expect("reference PMREM bind group should be valid");
        encode_ibl_bake_wgpu_compute_dispatch(&mut encoder, command, &pipeline, &bind_group)
            .expect("reference PMREM dispatch should encode");
    }
    queue.submit(std::iter::once(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("reference PMREM dispatches should finish");

    output
}

fn rgba16f_quantized_direction_source() -> Vec<[f32; 4]> {
    let mut source =
        vec![[0.0; 4]; source_cubemap_sample_count(SOURCE_FACE_SIZE, SOURCE_MIP_COUNT)];
    for face in CubemapFace::ALL {
        for mip_level in 0..SOURCE_MIP_COUNT {
            let mip_size = source_cubemap_mip_size(SOURCE_FACE_SIZE, mip_level);
            let offset =
                source_cubemap_face_mip_offset(SOURCE_FACE_SIZE, SOURCE_MIP_COUNT, face, mip_level);
            for y in 0..mip_size {
                for x in 0..mip_size {
                    let direction = cubemap_texel_direction(face, x, y, mip_size);
                    source[offset + y as usize * mip_size as usize + x as usize] = [
                        0.4 + 0.3 * direction[0] + 0.1 * direction[1] * direction[2],
                        0.5 + 0.25 * direction[1] + 0.08 * direction[0] * direction[2],
                        0.6 + 0.2 * direction[2] + 0.06 * direction[0] * direction[1],
                        1.0,
                    ];
                }
            }
        }
    }
    decode_rgba16f_texels(&encode_rgba16f_texels(&source))
}

fn create_rgba16f_source_cubemap(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source_texels: &[[f32; 4]],
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-reference-source-cubemap"),
        size: wgpu::Extent3d {
            width: SOURCE_FACE_SIZE,
            height: SOURCE_FACE_SIZE,
            depth_or_array_layers: 6,
        },
        mip_level_count: SOURCE_MIP_COUNT,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for face in CubemapFace::ALL {
        for mip_level in 0..SOURCE_MIP_COUNT {
            let mip_size = source_cubemap_mip_size(SOURCE_FACE_SIZE, mip_level);
            let offset =
                source_cubemap_face_mip_offset(SOURCE_FACE_SIZE, SOURCE_MIP_COUNT, face, mip_level);
            let texel_count = mip_size as usize * mip_size as usize;
            let bytes = encode_rgba16f_texels(&source_texels[offset..offset + texel_count]);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face.index() as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(8 * mip_size),
                    rows_per_image: Some(mip_size),
                },
                wgpu::Extent3d {
                    width: mip_size,
                    height: mip_size,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    texture
}
