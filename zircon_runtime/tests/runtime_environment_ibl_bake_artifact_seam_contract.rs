use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, cubemap_direction_from_scaled_uv,
    cubemap_face_scaled_uv_from_direction, cubemap_scaled_uv_for_texel,
    source_cubemap_mip_chain_with_bake_artifact, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, CubemapFace, IblBakeArtifactBlob,
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactPayload,
    IblBakeArtifactRequest, ProceduralSkyParams, SourceCubemapMipChain,
};

#[test]
fn runtime_environment_ibl_bake_artifact_pmrem_roundtrip_preserves_seam_metrics() {
    let bake_key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let offline_pmrem = build_source_cubemap_from_equirect(64, synthetic_seam_stress_environment);
    let request = IblBakeArtifactRequest::new(
        bake_key,
        offline_pmrem.source_face_size(),
        offline_pmrem.source_mip_count(),
    );
    let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &offline_pmrem, None)
        .expect("offline PMREM artifact payload should encode");
    let blob = IblBakeArtifactBlob::from_payload(payload);

    let decoded_blob = IblBakeArtifactBlob::decode_current_for_request(&request, &blob.encode())
        .expect("encoded artifact should decode for the current request");
    let applied =
        source_cubemap_mip_chain_with_bake_artifact(&offline_pmrem, decoded_blob.payload())
            .expect("decoded artifact should apply to the matching source cubemap");

    let mid_mip =
        source_cubemap_pmrem_mip_from_roughness(0.5, applied.pmrem_mip_count()).round() as u32;
    let rough_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, applied.pmrem_mip_count()).round() as u32;
    let expected_mid = pmrem_seam_luma_stats(&offline_pmrem, mid_mip);
    let expected_rough = pmrem_seam_luma_stats(&offline_pmrem, rough_mip);
    let applied_base = pmrem_seam_luma_stats(&applied, 0);
    let applied_mid = pmrem_seam_luma_stats(&applied, mid_mip);
    let applied_rough = pmrem_seam_luma_stats(&applied, rough_mip);

    assert_stats_close(expected_mid, applied_mid, 0.003);
    assert_stats_close(expected_rough, applied_rough, 0.003);
    assert!(
        applied_mid.mean < applied_base.mean * 0.9,
        "artifact-applied PMREM mid mip should still reduce seam energy, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
    );
    assert!(
        applied_rough.mean <= applied_mid.mean * 0.9 + 0.001,
        "artifact-applied PMREM rough mip should keep reducing seam energy, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
    );
    assert!(
        applied_rough.max < applied_base.max * 0.75,
        "artifact-applied PMREM rough mip should reduce worst seam delta, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
    );
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
        "mean seam delta changed across artifact roundtrip: expected={expected:?} actual={actual:?}"
    );
    assert!(
        (expected.max - actual.max).abs() <= tolerance,
        "max seam delta changed across artifact roundtrip: expected={expected:?} actual={actual:?}"
    );
}
