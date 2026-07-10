use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::artifact::{
    resolve_ibl_bake_artifact_runtime_dispatch, IblBakeArtifactAssetDerivedRead,
    IblBakeArtifactCacheStore, IblSourceCubemapStagingRead, IblSourceCubemapStagingStore,
    IBL_BAKE_ASSET_DERIVED_DIRECTORY, IBL_BAKE_ASSET_DERIVED_EXTENSION,
    IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY, IBL_SOURCE_CUBEMAP_STAGING_EXTENSION,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, IblBakeArtifactBlob, IblBakeArtifactContents,
    IblBakeArtifactRequest, IblBakeArtifactSource, ProceduralSkyParams, SourceCubemapMipChain,
};
use zircon_runtime::core::math::Real;

#[test]
fn zcube_staged_ibl_bundle_keeps_source_cube_separate_from_derived_pmrem() {
    let root = unique_temp_root("zcube_staged_bundle");
    let source = high_frequency_source_cubemap();
    let request = request_for_source(&source);
    let store = IblSourceCubemapStagingStore::new(&root);

    let report = store
        .write_source_cubemap_staged_bundle(&request, test_uri(), &source, None)
        .expect("staged source cubemap and derived IBL artifact should write");

    assert!(report.source_zcube().path().is_file());
    assert!(report.asset_derived().path().is_file());
    assert_zcube_source_path(report.source_zcube().path());
    assert_asset_derived_path(report.asset_derived().path());
    assert_ne!(
        report.source_zcube().path(),
        report.asset_derived().path(),
        "source .zcube and derived .zribl artifacts must not share a path"
    );

    let staged_read = store
        .read_source_cubemap_zcube(&request, test_uri())
        .expect("staged source .zcube should read");
    let staged_cubemap = match staged_read {
        IblSourceCubemapStagingRead::Hit(cubemap) => cubemap,
        other => panic!("expected staged .zcube hit, got {other:?}"),
    };
    assert_eq!(staged_cubemap.face_size(), source.face_size());
    assert_eq!(staged_cubemap.mip_count(), source.mip_count());
    assert_rgba16f_close(staged_cubemap.texels(), source.source_texels());

    let asset_read = store
        .asset_derived_store()
        .read_asset_derived_artifact(&request)
        .expect("asset-derived .zribl should read");
    let asset_blob = match asset_read {
        IblBakeArtifactAssetDerivedRead::Hit(blob) => blob,
        other => panic!("expected asset-derived .zribl hit, got {other:?}"),
    };

    let staged_environment = store
        .read_source_cubemap_environment(&request, test_uri())
        .expect("staged source and derived artifacts should restore a render environment");
    assert_rgba16f_close(
        staged_environment.mip_chain.source_texels(),
        source.source_texels(),
    );
    assert_rgba16f_close(staged_environment.mip_chain.texels(), source.texels());
    assert_ne!(
        staged_environment.bake_artifact_hash, [0; 4],
        "restored environments must carry the derived artifact identity"
    );

    let runtime_cache = IblBakeArtifactCacheStore::new(&root);
    let dispatch =
        resolve_ibl_bake_artifact_runtime_dispatch(&runtime_cache, &request, &[asset_blob.clone()])
            .expect("staged asset-derived .zribl should resolve before runtime compute");
    assert_eq!(
        dispatch.source(),
        IblBakeArtifactSource::AssetDerivedArtifact
    );
    assert_eq!(dispatch.environment_compute_dispatch_count(), 0);
    assert!(!dispatch.requires_runtime_compute());

    let zcube_bytes = fs::read(report.source_zcube().path()).expect("source zcube bytes");
    assert!(
        IblBakeArtifactBlob::decode_current_for_request(&request, &zcube_bytes).is_err(),
        ".zcube source bytes must not decode as a reusable PMREM/SH9 .zribl artifact"
    );

    let _ = fs::remove_dir_all(root);
}

fn high_frequency_source_cubemap() -> SourceCubemapMipChain {
    build_source_cubemap_from_equirect(8, |u, v| {
        let stripe = if ((u * 29.0).floor() as i32 + (v * 19.0).floor() as i32) & 1 == 0 {
            0.08
        } else {
            2.2
        };
        [stripe, 0.18 + u * 0.7, 0.28 + (1.0 - v) * 0.9, 1.0]
    })
}

fn request_for_source(source: &SourceCubemapMipChain) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        source.face_size(),
        source.mip_count(),
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9)
}

fn test_uri() -> AssetUri {
    AssetUri::parse("res://textures/staged_hdri_environment.zcube").expect("valid texture uri")
}

fn assert_rgba16f_close(actual: &[[Real; 4]], expected: &[[Real; 4]]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        for channel in 0..4 {
            let delta = (actual[channel] - expected[channel]).abs();
            assert!(
                delta <= 0.0015,
                "texel {index} channel {channel} differs after staged .zcube roundtrip: actual={}, expected={}, delta={delta}",
                actual[channel],
                expected[channel]
            );
        }
    }
}

fn assert_zcube_source_path(path: &std::path::Path) {
    assert_path_contains_directory(path, IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY);
    assert_eq!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(IBL_SOURCE_CUBEMAP_STAGING_EXTENSION)
    );
}

fn assert_asset_derived_path(path: &std::path::Path) {
    assert_path_contains_directory(path, IBL_BAKE_ASSET_DERIVED_DIRECTORY);
    assert_eq!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(IBL_BAKE_ASSET_DERIVED_EXTENSION)
    );
}

fn assert_path_contains_directory(path: &std::path::Path, directory: &str) {
    let expected = directory.split('/').collect::<Vec<_>>();
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        components
            .windows(expected.len())
            .any(|window| window == expected.as_slice()),
        "expected path to contain {directory}, got {}",
        path.display()
    );
}

fn unique_temp_root(name: &str) -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_{name}_{}_{}",
        std::process::id(),
        timestamp
    ))
}
