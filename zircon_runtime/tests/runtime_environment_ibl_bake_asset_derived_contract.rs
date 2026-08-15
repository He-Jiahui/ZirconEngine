use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::artifact::{
    resolve_ibl_bake_artifact_runtime_dispatch, IblBakeArtifactAssetDerivedError,
    IblBakeArtifactAssetDerivedRead, IblBakeArtifactAssetDerivedStore, IblBakeArtifactCacheStore,
    IBL_BAKE_ASSET_DERIVED_DIRECTORY, IBL_BAKE_ASSET_DERIVED_EXTENSION,
};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, cubemap_direction_from_scaled_uv,
    cubemap_face_scaled_uv_from_direction, cubemap_scaled_uv_for_texel,
    source_cubemap_mip_chain_with_bake_artifact, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, CubemapFace, IblBakeArtifactBlob,
    IblBakeArtifactBlobError, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, IblBakeArtifactProducer, IblBakeArtifactRequest, IblBakeArtifactSource,
    ProceduralSkyParams, SourceCubemapMipChain, IBL_BAKE_ALGORITHM_VERSION,
};

#[test]
fn runtime_environment_ibl_bake_asset_derived_store_prebake_beats_runtime_cache() {
    let root = unique_temp_root("asset_derived_priority");
    let source = build_source_cubemap_from_equirect(16, synthetic_asset_derived_environment);
    let request = request_for_source(&source);
    let asset_store = IblBakeArtifactAssetDerivedStore::new(&root);

    let report = asset_store
        .write_source_cubemap_asset_derived_artifact(&request, &source, None)
        .expect("staged source-cubemap IBL artifact should write");
    assert!(report.path().is_file());
    assert_eq!(
        report.descriptor().contents(),
        IblBakeArtifactContents::PMREM_SH9
    );
    assert_eq!(
        report.payload_len(),
        report.descriptor().expected_payload_size_bytes()
    );
    assert_asset_derived_path(report.path());

    let read = asset_store
        .read_asset_derived_artifact(&request)
        .expect("current asset-derived IBL artifact should read");
    let asset_blob = match read {
        IblBakeArtifactAssetDerivedRead::Hit(blob) => blob,
        other => panic!("expected asset-derived hit, got {other:?}"),
    };

    let runtime_cache = IblBakeArtifactCacheStore::new(&root);
    let runtime_source =
        build_source_cubemap_from_equirect(16, synthetic_runtime_cache_environment);
    let runtime_blob = runtime_cache_blob_for_request(&request, &runtime_source);
    runtime_cache
        .write_runtime_cache(&runtime_blob)
        .expect("runtime cache blob should write");

    let dispatch =
        resolve_ibl_bake_artifact_runtime_dispatch(&runtime_cache, &request, &[asset_blob.clone()])
            .expect("asset-derived runtime dispatch should resolve");
    assert_eq!(
        dispatch.source(),
        IblBakeArtifactSource::AssetDerivedArtifact
    );
    assert_eq!(dispatch.environment_compute_dispatch_count(), 0);
    assert!(!dispatch.requires_runtime_compute());
    assert_eq!(
        dispatch.payload().expect("asset-derived payload").bytes(),
        asset_blob.payload().bytes(),
        "asset-derived blob must win over a same-request runtime cache blob"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_environment_ibl_bake_asset_derived_store_rejects_stale_blob() {
    let root = unique_temp_root("asset_derived_stale");
    let source = build_source_cubemap_from_equirect(8, synthetic_asset_derived_environment);
    let request = request_for_source(&source);
    let stale_descriptor = IblBakeArtifactDescriptor::current_for_request(&request)
        .with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let stale_payload =
        IblBakeArtifactPayload::from_source_cubemap(stale_descriptor, &source, None)
            .expect("stale payload layout should encode");
    let stale_blob = IblBakeArtifactBlob::from_payload(stale_payload);
    let asset_store = IblBakeArtifactAssetDerivedStore::new(&root);
    asset_store
        .write_asset_derived_blob(&stale_blob)
        .expect("stale blob fixture should write");

    let read = asset_store
        .read_asset_derived_artifact(&request)
        .expect("stale blob read should not be fatal");
    match read {
        IblBakeArtifactAssetDerivedRead::Rejected(
            IblBakeArtifactBlobError::DescriptorNotCurrent { descriptor },
        ) => assert_eq!(
            descriptor.algorithm_version(),
            IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1)
        ),
        other => panic!("expected stale descriptor rejection, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_environment_ibl_bake_asset_derived_store_rejects_gpu_runtime_blob() {
    let root = unique_temp_root("asset_derived_gpu_producer");
    let source = build_source_cubemap_from_equirect(8, synthetic_runtime_cache_environment);
    let request = request_for_source(&source);
    let runtime_blob = runtime_cache_blob_for_request(&request, &source);
    let asset_store = IblBakeArtifactAssetDerivedStore::new(&root);

    assert!(matches!(
        asset_store.write_asset_derived_blob(&runtime_blob),
        Err(IblBakeArtifactAssetDerivedError::InvalidProducer {
            producer: IblBakeArtifactProducer::RendererGpuRuntime
        })
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_environment_ibl_bake_asset_derived_store_preserves_pmrem_seams() {
    let root = unique_temp_root("asset_derived_seam");
    let source = build_source_cubemap_from_equirect(64, synthetic_seam_stress_environment);
    let request = request_for_source(&source);
    let asset_store = IblBakeArtifactAssetDerivedStore::new(&root);
    asset_store
        .write_source_cubemap_asset_derived_artifact(&request, &source, None)
        .expect("seam-stress asset-derived artifact should write");

    let read = asset_store
        .read_asset_derived_artifact(&request)
        .expect("seam-stress artifact should read");
    let blob = read.blob().expect("asset-derived artifact should hit");
    let applied = source_cubemap_mip_chain_with_bake_artifact(&source, blob.payload())
        .expect("asset-derived artifact should apply to the matching source cubemap");

    let mid_mip =
        source_cubemap_pmrem_mip_from_roughness(0.5, applied.pmrem_mip_count()).round() as u32;
    let rough_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, applied.pmrem_mip_count()).round() as u32;
    let expected_mid = pmrem_seam_luma_stats(&source, mid_mip);
    let expected_rough = pmrem_seam_luma_stats(&source, rough_mip);
    let applied_base = pmrem_seam_luma_stats(&applied, 0);
    let applied_mid = pmrem_seam_luma_stats(&applied, mid_mip);
    let applied_rough = pmrem_seam_luma_stats(&applied, rough_mip);

    assert_stats_close(expected_mid, applied_mid, 0.003);
    assert_stats_close(expected_rough, applied_rough, 0.003);
    assert!(
        applied_mid.mean < applied_base.mean * 0.9,
        "asset-derived PMREM mid mip should reduce seam energy, base={applied_base:?} mid={applied_mid:?}"
    );
    assert!(
        applied_rough.max < applied_base.max * 0.75,
        "asset-derived PMREM rough mip should reduce worst seam delta, base={applied_base:?} rough={applied_rough:?}"
    );

    let _ = fs::remove_dir_all(root);
}

fn request_for_source(source: &SourceCubemapMipChain) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        source.source_face_size(),
        source.source_mip_count(),
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9)
}

fn runtime_cache_blob_for_request(
    request: &IblBakeArtifactRequest,
    source: &SourceCubemapMipChain,
) -> IblBakeArtifactBlob {
    let descriptor = IblBakeArtifactDescriptor::current_for_runtime_cache_request(request);
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, source, None)
        .expect("fixture payload should encode");
    IblBakeArtifactBlob::from_payload(payload)
}

fn synthetic_asset_derived_environment(u: f32, v: f32) -> [f32; 4] {
    let horizon = (1.0 - (v - 0.52).abs() * 2.0).clamp(0.0, 1.0);
    let sun = (1.0 - ((u - 0.17).abs() * 24.0 + (v - 0.41).abs() * 18.0)).clamp(0.0, 1.0);
    [
        0.08 + horizon * 0.22 + sun * 1.4,
        0.12 + horizon * 0.28 + sun * 0.9,
        0.18 + (1.0 - v).max(0.0) * 0.46 + sun * 0.3,
        1.0,
    ]
}

fn synthetic_runtime_cache_environment(u: f32, v: f32) -> [f32; 4] {
    let stripe = (std::f32::consts::TAU * (u * 5.0 + v * 3.0)).sin() * 0.12;
    [0.25 + stripe, 0.18, 0.08, 1.0]
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
        "mean seam delta changed across asset-derived roundtrip: expected={expected:?} actual={actual:?}"
    );
    assert!(
        (expected.max - actual.max).abs() <= tolerance,
        "max seam delta changed across asset-derived roundtrip: expected={expected:?} actual={actual:?}"
    );
}

fn assert_asset_derived_path(path: &std::path::Path) {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        components
            .windows(2)
            .any(|window| window[0] == "render" && window[1] == "ibl-derived"),
        "asset-derived IBL artifact should live under {IBL_BAKE_ASSET_DERIVED_DIRECTORY}, got {}",
        path.display()
    );
    assert_eq!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(IBL_BAKE_ASSET_DERIVED_EXTENSION)
    );
}

fn unique_temp_root(name: &str) -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_ibl_bake_{name}_{}_{}",
        std::process::id(),
        timestamp
    ))
}
