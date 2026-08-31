use super::pending::{IblBakeArtifactWgpuPendingReadback, IblBakeArtifactWgpuReadbackSection};
use super::resources::{
    required_irradiance_sh9_readback_resource, required_wgpu_readback_resource,
};
use super::staging::strip_padded_cube_mip_chain;
use crate::core::framework::render::{
    CubemapFace, IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactPayload,
    IblBakeArtifactRequest, ProceduralSkyParams, SOURCE_CUBEMAP_FACE_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, SourceCubemapIrradianceCube, SourceCubemapMipChain,
    build_source_cubemap_from_equirect, cubemap_direction_from_scaled_uv,
    cubemap_face_scaled_uv_from_direction, cubemap_scaled_uv_for_texel,
    source_cubemap_face_mip_offset, source_cubemap_mip_chain_with_bake_artifact,
    source_cubemap_mip_size, source_cubemap_pmrem_mip_from_roughness,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;
use wgpu::util::DeviceExt;

const RGBA16F_BYTES_PER_TEXEL: usize = 8;

#[test]
fn ibl_readback_root_remains_a_declarative_orchestration_owner() {
    let root = include_str!("../read_ibl_bake_artifact_sections.rs");
    let batch = include_str!("batch.rs");
    let pending = include_str!("pending.rs");
    let resources = include_str!("resources.rs");
    let staging = include_str!("staging.rs");

    for module in [
        "mod batch;",
        "mod pending;",
        "mod resources;",
        "mod staging;",
        "mod tests;",
    ] {
        assert!(root.contains(module), "missing readback owner: {module}");
    }
    assert!(root.lines().count() < 200);
    assert!(!root.contains("struct BufferReadback"));
    assert!(!root.contains("struct IblBakeArtifactWgpuPendingReadback"));
    assert!(batch.contains("struct IblBakeArtifactWgpuReadbackBatch"));
    assert!(pending.contains("struct IblBakeArtifactWgpuPendingReadback"));
    assert!(!pending.contains("wgpu::Buffer"));
    assert!(!pending.contains("map_async"));
    assert!(!pending.contains("device.poll"));
    assert!(!pending.contains("queue.submit"));
    assert!(resources.contains("struct IblBakeArtifactWgpuReadbackResources"));
    assert!(staging.contains("struct BufferReadback"));
    assert!(staging.contains("struct CubeMipChainReadback"));
}

#[test]
fn product_pending_sections_assemble_face_then_mip_order_without_native_map_ownership() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 2, 2, IblBakeArtifactContents::PMREM_SH9_IEM);
    let pending = IblBakeArtifactWgpuPendingReadback::new(descriptor)
        .expect("small descriptor should create bounded section slots");

    for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip in 0..descriptor.mip_count() as usize {
            let size = source_cubemap_mip_size(descriptor.face_size(), mip as u32) as usize;
            pending.record_delivery(
                IblBakeArtifactWgpuReadbackSection::Pmrem,
                face * descriptor.mip_count() as usize + mip,
                Ok(vec![
                    (face * 10 + mip) as u8;
                    size * size * RGBA16F_BYTES_PER_TEXEL
                ]),
            );
        }
    }
    pending.record_delivery(
        IblBakeArtifactWgpuReadbackSection::IrradianceSh9,
        0,
        Ok(vec![
            77;
            descriptor.expected_irradiance_sh9_size_bytes().unwrap()
        ]),
    );
    for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        let face_bytes = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * RGBA16F_BYTES_PER_TEXEL;
        pending.record_delivery(
            IblBakeArtifactWgpuReadbackSection::IrradianceCube,
            face,
            Ok(vec![(100 + face) as u8; face_bytes]),
        );
    }

    assert!(pending.poll_ready());
    let sections = pending
        .finish()
        .expect("all product sections should assemble");
    let pmrem = sections.pmrem_rgba16f_bytes().expect("PMREM section");
    assert_eq!(pmrem[0], 0);
    assert_eq!(pmrem[32], 1);
    assert_eq!(pmrem[40], 10);
    assert_eq!(sections.irradiance_sh9_bytes().unwrap()[0], 77);
    assert_eq!(sections.irradiance_cube_rgba16f_bytes().unwrap()[0], 100);
}

#[test]
fn product_pending_sections_fail_the_artifact_after_any_terminal_error() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor = IblBakeArtifactDescriptor::current(key, 2, 2, IblBakeArtifactContents::SH9);
    let pending = IblBakeArtifactWgpuPendingReadback::new(descriptor).unwrap();

    pending.record_delivery(
        IblBakeArtifactWgpuReadbackSection::IrradianceSh9,
        0,
        Err("over budget".to_string()),
    );

    assert!(pending.poll_ready());
    let error = pending
        .finish()
        .expect_err("partial artifacts must never be published");
    assert!(error.to_string().contains("over budget"));
}

#[test]
fn readback_resources_preserve_descriptor() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);

    let resources = super::IblBakeArtifactWgpuReadbackResources::new(descriptor);

    assert_eq!(resources.descriptor(), descriptor);
}

#[test]
fn readback_resources_report_required_wgpu_inputs_from_descriptor_contents() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let pmrem_sh9 =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let pmrem_sh9_iem =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);

    let resources = super::IblBakeArtifactWgpuReadbackResources::new(pmrem_sh9);

    assert!(resources.requires_pmrem_texture());
    assert!(resources.requires_irradiance_sh9_buffer());
    assert!(!resources.requires_irradiance_cube_texture());

    let resources = super::IblBakeArtifactWgpuReadbackResources::new(pmrem_sh9_iem);

    assert!(resources.requires_pmrem_texture());
    assert!(resources.requires_irradiance_sh9_buffer());
    assert!(resources.requires_irradiance_cube_texture());
}

#[test]
fn required_readback_resource_reports_missing_label() {
    let error = required_wgpu_readback_resource::<u32>(None, "SH9 buffer")
        .expect_err("missing resource should fail");

    assert!(matches!(
        error,
        GraphicsError::BufferMap(message) if message.contains("SH9 buffer")
    ));
}

#[test]
fn sh9_readback_resource_rejects_wrong_size_and_out_of_bounds_window() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor = IblBakeArtifactDescriptor::current(key, 2, 2, IblBakeArtifactContents::SH9);
    let expected = descriptor.expected_irradiance_sh9_size_bytes().unwrap() as u64;
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ibl-sh9-readback-window-validation"),
        size: expected + 16,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let wrong_size = super::IblBakeArtifactWgpuReadbackResources::new(descriptor)
        .with_irradiance_sh9_buffer_range(&buffer, 0, expected - 4);
    let error = match required_irradiance_sh9_readback_resource(&wrong_size) {
        Ok(_) => panic!("SH9 readback must reject a descriptor/window size mismatch"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("expected"));

    let out_of_bounds = super::IblBakeArtifactWgpuReadbackResources::new(descriptor)
        .with_irradiance_sh9_buffer_range(&buffer, 32, expected);
    let error = match required_irradiance_sh9_readback_resource(&out_of_bounds) {
        Ok(_) => panic!("SH9 readback must reject a physical buffer overrun"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("exceeds physical buffer size"));
}

#[test]
fn batched_cube_readback_strips_padding_in_face_then_mip_order() {
    let bytes_per_face = 768;
    let mut mapped = vec![0_u8; bytes_per_face * SOURCE_CUBEMAP_FACE_COUNT as usize];
    let mut staging_offset = 0;
    let mut value = 1_u8;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_size in [2_usize, 1] {
            for row in 0..mip_size {
                let row_offset = staging_offset + row * 256;
                mapped[row_offset..row_offset + mip_size * 8].fill(value);
            }
            staging_offset += mip_size * 256;
            value = value.wrapping_add(1);
        }
    }

    let bytes = strip_padded_cube_mip_chain(&mapped, 2, 2);

    assert_eq!(bytes.len(), 240);
    assert_eq!(&bytes[..32], &[1_u8; 32]);
    assert_eq!(&bytes[32..40], &[2_u8; 8]);
    assert_eq!(&bytes[200..232], &[11_u8; 32]);
    assert_eq!(&bytes[232..], &[12_u8; 8]);
}

#[test]
fn batched_readback_preserves_pmrem_sh9_and_iem_payload_bytes() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let pmrem = build_source_cubemap_from_equirect(32, synthetic_seam_stress_environment);
    let request =
        IblBakeArtifactRequest::new(key, pmrem.source_face_size(), pmrem.source_mip_count())
            .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);
    let irradiance_cube = SourceCubemapIrradianceCube::new(
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        vec![
            [0.125, 0.25, 0.5];
            CubemapFace::ALL.len()
                * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
                * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        ],
    );
    let payload =
        IblBakeArtifactPayload::from_source_cubemap(descriptor, &pmrem, Some(&irradiance_cube))
            .expect("PMREM/SH9/IEM payload should encode");
    let pmrem_range = payload.pmrem_rgba16f_byte_range().expect("pmrem range");
    let sh9_range = payload.irradiance_sh9_byte_range().expect("sh9 range");
    let iem_range = payload
        .irradiance_cube_rgba16f_byte_range()
        .expect("irradiance cube range");
    let pmrem_texture = create_pmrem_texture(&backend.device, descriptor);
    upload_cube_payload_to_texture(
        &backend.queue,
        &pmrem_texture,
        descriptor.face_size(),
        descriptor.mip_count(),
        &payload.bytes()[pmrem_range],
    );
    let irradiance_cube_texture = create_cube_texture(
        &backend.device,
        "ibl-readback-irradiance-cube-texture",
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        1,
    );
    upload_cube_payload_to_texture(
        &backend.queue,
        &irradiance_cube_texture,
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        1,
        &payload.bytes()[iem_range],
    );
    let sh9_buffer = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ibl-readback-seam-sh9-buffer"),
            contents: &payload.bytes()[sh9_range],
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        });

    let sections = super::read_ibl_bake_artifact_wgpu_sections(
        &backend.device,
        &backend.queue,
        super::IblBakeArtifactWgpuReadbackResources::new(descriptor)
            .with_pmrem_texture(&pmrem_texture)
            .with_irradiance_sh9_buffer(&sh9_buffer)
            .with_irradiance_cube_texture(&irradiance_cube_texture),
    )
    .expect("WGPU PMREM/SH9/IEM batch readback should produce artifact sections");
    let readback_payload = sections
        .into_payload()
        .expect("readback sections should assemble into a current payload");
    assert_eq!(readback_payload.bytes(), payload.bytes());

    let applied = source_cubemap_mip_chain_with_bake_artifact(&pmrem, &readback_payload)
        .expect("readback payload should apply to the matching source cubemap");
    let mid_mip =
        source_cubemap_pmrem_mip_from_roughness(0.5, applied.pmrem_mip_count()).round() as u32;
    let rough_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, applied.pmrem_mip_count()).round() as u32;
    let expected_mid = pmrem_seam_luma_stats(&pmrem, mid_mip);
    let expected_rough = pmrem_seam_luma_stats(&pmrem, rough_mip);
    let applied_base = pmrem_seam_luma_stats(&applied, 0);
    let applied_mid = pmrem_seam_luma_stats(&applied, mid_mip);
    let applied_rough = pmrem_seam_luma_stats(&applied, rough_mip);

    assert_stats_close(expected_mid, applied_mid, 0.003);
    assert_stats_close(expected_rough, applied_rough, 0.003);
    assert!(
        applied_mid.mean < applied_base.mean * 0.9,
        "WGPU-readback PMREM mid mip should still reduce seam energy, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
    );
    assert!(
        applied_rough.max < applied_base.max * 0.75,
        "WGPU-readback PMREM rough mip should reduce worst seam delta, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
    );
}

fn create_pmrem_texture(
    device: &wgpu::Device,
    descriptor: IblBakeArtifactDescriptor,
) -> wgpu::Texture {
    create_cube_texture(
        device,
        "ibl-readback-seam-pmrem-texture",
        descriptor.face_size(),
        descriptor.mip_count(),
    )
}

fn create_cube_texture(
    device: &wgpu::Device,
    label: &'static str,
    face_size: u32,
    mip_count: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: face_size,
            height: face_size,
            depth_or_array_layers: 6,
        },
        mip_level_count: mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn upload_cube_payload_to_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    face_size: u32,
    mip_count: u32,
    cube_bytes: &[u8],
) {
    for face in CubemapFace::ALL {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size as usize * RGBA16F_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
            let mut padded = vec![0; padded_bytes_per_row * mip_size as usize];
            let source_offset =
                source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level)
                    * RGBA16F_BYTES_PER_TEXEL;
            for row in 0..mip_size as usize {
                let source_row = source_offset + row * mip_size as usize * RGBA16F_BYTES_PER_TEXEL;
                let target_row = row * padded_bytes_per_row;
                padded[target_row..target_row + unpadded_bytes_per_row]
                    .copy_from_slice(&cube_bytes[source_row..source_row + unpadded_bytes_per_row]);
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face.index() as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &padded,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
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
}

fn synthetic_seam_stress_environment(u: f32, v: f32) -> [f32; 4] {
    let wave_a = (std::f32::consts::TAU * u * 17.0).sin();
    let wave_b = (std::f32::consts::TAU * (u * 11.0 + v * 7.0)).cos();
    let wave_c = (std::f32::consts::PI * v * 9.0).sin();
    let luma = 0.55 + wave_a * 0.22 + wave_b * 0.16 + wave_c * 0.12;
    [luma, luma * 0.85, luma * 0.7, 1.0]
}

#[derive(Clone, Copy, Debug)]
struct SeamLumaStats {
    mean: f32,
    max: f32,
}

fn pmrem_seam_luma_stats(cubemap: &SourceCubemapMipChain, mip_level: u32) -> SeamLumaStats {
    let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), mip_level);
    let mut sum = 0.0;
    let mut max = 0.0_f32;
    let mut count = 0.0;

    for face in CubemapFace::ALL {
        for side in CubeEdgeSide::ALL {
            let sample_start = if mip_size > 2 { 1 } else { 0 };
            let sample_end = if mip_size > 2 {
                mip_size.saturating_sub(1)
            } else {
                mip_size
            };
            for index in sample_start..sample_end {
                let (x, y) = side.edge_texel(index, mip_size);
                let current = cubemap.pmrem_texel(face, mip_level, x, y);
                let (neighbor_face, neighbor_x, neighbor_y) =
                    side.neighbor_texel(face, index, mip_size);
                let neighbor =
                    cubemap.pmrem_texel(neighbor_face, mip_level, neighbor_x, neighbor_y);
                let delta = (luma(current) - luma(neighbor)).abs();
                sum += delta;
                max = max.max(delta);
                count += 1.0;
            }
        }
    }

    SeamLumaStats {
        mean: sum / count,
        max,
    }
}

#[derive(Clone, Copy, Debug)]
enum CubeEdgeSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl CubeEdgeSide {
    const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    fn edge_texel(self, index: u32, size: u32) -> (u32, u32) {
        match self {
            Self::Left => (0, index),
            Self::Right => (size.saturating_sub(1), index),
            Self::Top => (index, 0),
            Self::Bottom => (index, size.saturating_sub(1)),
        }
    }

    fn neighbor_texel(self, face: CubemapFace, index: u32, size: u32) -> (CubemapFace, u32, u32) {
        let edge_uv = match self {
            Self::Left => [
                -1.0 - 1.0 / size as f32,
                cubemap_scaled_uv_for_texel(0, index, size)[1],
            ],
            Self::Right => [
                1.0 + 1.0 / size as f32,
                cubemap_scaled_uv_for_texel(size.saturating_sub(1), index, size)[1],
            ],
            Self::Top => [
                cubemap_scaled_uv_for_texel(index, 0, size)[0],
                -1.0 - 1.0 / size as f32,
            ],
            Self::Bottom => [
                cubemap_scaled_uv_for_texel(index, size.saturating_sub(1), size)[0],
                1.0 + 1.0 / size as f32,
            ],
        };
        let direction = cubemap_direction_from_scaled_uv(face, edge_uv);
        let (neighbor_face, neighbor_uv) = cubemap_face_scaled_uv_from_direction(direction);
        (
            neighbor_face,
            texel_coord_from_scaled_axis(neighbor_uv[0], size),
            texel_coord_from_scaled_axis(neighbor_uv[1], size),
        )
    }
}

fn texel_coord_from_scaled_axis(scaled_axis: f32, size: u32) -> u32 {
    (((scaled_axis * 0.5 + 0.5) * size as f32 - 0.5).round() as i32)
        .clamp(0, size.saturating_sub(1) as i32) as u32
}

fn luma(texel: [f32; 4]) -> f32 {
    0.2126 * texel[0] + 0.7152 * texel[1] + 0.0722 * texel[2]
}

fn assert_stats_close(expected: SeamLumaStats, actual: SeamLumaStats, tolerance: f32) {
    assert!(
        (expected.mean - actual.mean).abs() <= tolerance,
        "mean seam delta changed across WGPU readback: expected={expected:?} actual={actual:?}"
    );
    assert!(
        (expected.max - actual.max).abs() <= tolerance,
        "max seam delta changed across WGPU readback: expected={expected:?} actual={actual:?}"
    );
}
