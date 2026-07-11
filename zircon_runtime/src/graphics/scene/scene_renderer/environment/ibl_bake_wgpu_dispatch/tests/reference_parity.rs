use crate::core::framework::render::{
    build_source_cubemap_from_source_mips_with_quality, cubemap_texel_direction,
    decode_rgba16f_texels, encode_rgba16f_texels, source_cubemap_face_mip_offset,
    source_cubemap_mip_size, source_cubemap_sample_count, CubemapFace, IblBakeArtifactContents,
    SourceCubemapPrefilterQuality, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::graphics::backend::RenderBackend;

use super::super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::super::super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_bind_group, create_ibl_bake_wgpu_params_buffer,
    create_ibl_bake_wgpu_source_sampler, IblBakeWgpuBindGroupLayouts,
    IblBakeWgpuOutputBindingResource,
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

#[test]
fn render_env_prefilter_cpu_gpu_match_16() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let device = &backend.device;
    let queue = &backend.queue;
    let source_texels = rgba16f_quantized_direction_source();
    let cpu = build_source_cubemap_from_source_mips_with_quality(
        SOURCE_FACE_SIZE,
        SOURCE_MIP_COUNT,
        source_texels.clone(),
        SourceCubemapPrefilterQuality::Normal,
    );

    let source_texture = create_rgba16f_source_cubemap(device, queue, &source_texels);
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
                        let error = (gpu[gpu_offset + local][channel]
                            - cpu.pmrem_texels()[cpu_offset + local][channel])
                            .abs();
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
