use std::sync::mpsc;

use crate::core::framework::render::{
    build_source_cubemap_from_source_mips_with_quality, cubemap_texel_direction,
    decode_rgba16f_texels, encode_rgba16f_texels, source_cubemap_evaluate_irradiance_sh9,
    source_cubemap_face_mip_offset, source_cubemap_mip_size, source_cubemap_sample_count,
    source_cubemap_sample_irradiance_cube, CubemapFace, IblBakeArtifactContents,
    SourceCubemapIrradianceCube, SourceCubemapIrradianceSh9, SourceCubemapPrefilterQuality,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
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
    command_for_kind, create_ibl_bake_wgpu_compute_pipeline, create_sh9_output_buffer,
    create_storage_output_texture, encode_ibl_bake_wgpu_compute_dispatch,
    read_rgba16float_mip_faces, request, storage_texture_descriptor,
};

const SOURCE_FACE_SIZE: u32 = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE;
const SOURCE_MIP_COUNT: u32 = 6;

#[test]
fn render_env_sh9_matches_cpu_reference() {
    const COEFFICIENT_TOLERANCE: f32 = 0.004;

    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let source = direction_source();
    let cpu = build_source_cubemap_from_source_mips_with_quality(
        SOURCE_FACE_SIZE,
        SOURCE_MIP_COUNT,
        source.clone(),
        SourceCubemapPrefilterQuality::Normal,
    );
    let gpu = dispatch_sh9(&backend, &source);

    for coefficient in 0..9 {
        for channel in 0..3 {
            let error =
                (gpu[coefficient][channel] - cpu.irradiance_sh9()[coefficient][channel]).abs();
            assert!(
                error <= COEFFICIENT_TOLERANCE,
                "SH9 CPU/GPU mismatch: coefficient={coefficient} channel={channel} error={error} gpu={} cpu={}",
                gpu[coefficient][channel],
                cpu.irradiance_sh9()[coefficient][channel],
            );
        }
    }

    let constant = vec![
        [0.25, 0.5, 0.75, 1.0];
        source_cubemap_sample_count(SOURCE_FACE_SIZE, SOURCE_MIP_COUNT)
    ];
    let constant_gpu = dispatch_sh9(&backend, &constant);
    for (coefficient, value) in constant_gpu.iter().enumerate().skip(1) {
        for channel in 0..3 {
            assert!(
                value[channel].abs() <= 0.0005,
                "constant environment must only populate SH band zero: coefficient={coefficient} channel={channel} value={}",
                value[channel],
            );
        }
    }
}

#[test]
fn render_env_iem_matches_sh9_low_frequency() {
    const LOW_FREQUENCY_TOLERANCE: f32 = 0.055;

    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let source = direction_source();
    let sh9 = dispatch_sh9(&backend, &source);
    let iem = dispatch_iem(&backend, &source);

    for sample_index in 0..64 {
        let direction = fibonacci_sphere_direction(sample_index, 64);
        let sh = source_cubemap_evaluate_irradiance_sh9(&sh9, direction);
        let cube = source_cubemap_sample_irradiance_cube(&iem, direction);
        for channel in 0..3 {
            let error = (cube[channel] - sh[channel]).abs();
            assert!(
                error <= LOW_FREQUENCY_TOLERANCE,
                "IEM/SH9 low-frequency mismatch: sample={sample_index} channel={channel} error={error} iem={cube:?} sh9={sh:?}",
            );
        }
    }
}

fn direction_source() -> Vec<[f32; 4]> {
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

fn dispatch_sh9(backend: &RenderBackend, source: &[[f32; 4]]) -> SourceCubemapIrradianceSh9 {
    let device = &backend.device;
    let queue = &backend.queue;
    let source_texture = create_source_texture(device, queue, source);
    let source_view = source_texture.create_view(&source_view_descriptor());
    let request = request(
        SOURCE_FACE_SIZE,
        SOURCE_MIP_COUNT,
        IblBakeArtifactContents::SH9,
    );
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let command = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9);
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let output = create_sh9_output_buffer(device);
    let params = create_ibl_bake_wgpu_params_buffer(device, command);
    let bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        command,
        &params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageBuffer(&output),
    )
    .expect("reference SH9 bind group should be valid");
    let pipeline = create_ibl_bake_wgpu_compute_pipeline(
        device,
        command,
        layouts.layout(command.bind_group_layout_kind),
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-reference-sh9-encoder"),
    });
    encode_ibl_bake_wgpu_compute_dispatch(&mut encoder, command, &pipeline, &bind_group)
        .expect("reference SH9 dispatch should encode");
    queue.submit(std::iter::once(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("reference SH9 dispatch should finish");

    let bytes = read_buffer_bytes(
        device,
        queue,
        &output,
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
    );
    let mut coefficients = [[0.0; 4]; 9];
    for (coefficient, bytes) in coefficients.iter_mut().zip(bytes.chunks_exact(16)) {
        for channel in 0..4 {
            let offset = channel * 4;
            coefficient[channel] = f32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("SH9 channel must be four bytes"),
            );
        }
    }
    coefficients
}

fn dispatch_iem(backend: &RenderBackend, source: &[[f32; 4]]) -> SourceCubemapIrradianceCube {
    let device = &backend.device;
    let queue = &backend.queue;
    let source_texture = create_source_texture(device, queue, source);
    let source_view = source_texture.create_view(&source_view_descriptor());
    let request = request(
        SOURCE_FACE_SIZE,
        SOURCE_MIP_COUNT,
        IblBakeArtifactContents::PMREM_SH9_IEM,
    );
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let command = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceCube);
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let output = create_storage_output_texture(device, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, 1);
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
    .expect("reference IEM bind group should be valid");
    let pipeline = create_ibl_bake_wgpu_compute_pipeline(
        device,
        command,
        layouts.layout(command.bind_group_layout_kind),
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-reference-iem-encoder"),
    });
    encode_ibl_bake_wgpu_compute_dispatch(&mut encoder, command, &pipeline, &bind_group)
        .expect("reference IEM dispatch should encode");
    queue.submit(std::iter::once(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("reference IEM dispatch should finish");

    let rgba = decode_rgba16f_texels(&read_rgba16float_mip_faces(
        device,
        queue,
        &output,
        0,
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        6,
    ));
    SourceCubemapIrradianceCube::new(
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        rgba.into_iter()
            .map(|texel| [texel[0], texel[1], texel[2]])
            .collect(),
    )
}

fn create_source_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &[[f32; 4]],
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-irradiance-reference-source"),
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
            let bytes = encode_rgba16f_texels(&source[offset..offset + texel_count]);
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

fn source_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("ibl-bake-irradiance-reference-source-view"),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(SOURCE_MIP_COUNT),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}

fn read_buffer_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
    size: u64,
) -> Vec<u8> {
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ibl-bake-reference-buffer-readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-reference-buffer-readback-encoder"),
    });
    encoder.copy_buffer_to_buffer(source, 0, &readback, 0, size);
    queue.submit(std::iter::once(encoder.finish()));
    let slice = readback.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("reference buffer readback poll should finish");
    receiver
        .recv()
        .expect("reference buffer readback callback should run")
        .expect("reference buffer readback should map");
    let bytes = slice.get_mapped_range().to_vec();
    readback.unmap();
    bytes
}

fn fibonacci_sphere_direction(index: u32, count: u32) -> [f32; 3] {
    let y = 1.0 - 2.0 * (index as f32 + 0.5) / count as f32;
    let radius = (1.0 - y * y).max(0.0).sqrt();
    let phi = index as f32 * 2.399_963_1;
    [radius * phi.cos(), y, radius * phi.sin()]
}
