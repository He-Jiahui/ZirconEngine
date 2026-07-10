use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use zircon_runtime::asset::artifact::{
    IblBakeArtifactAssetDerivedRead, IblSourceCubemapStagingRead, IblSourceCubemapStagingStore,
};
use zircon_runtime::asset::{
    decode_texture_source_image_rgba32f, stage_environment_ibl_source, AssetImportContext,
    AssetUri, EnvironmentIblSourceStagingStatus,
};
use zircon_runtime::core::framework::render::IblBakeArtifactContents;

#[test]
fn hdr_decode_preserves_linear_radiance_above_one() {
    let context = hdr_context("res://textures/radiance.hdr", 4, 2, "");

    let decoded = decode_texture_source_image_rgba32f(&context).expect("decode HDR as RGBA32F");

    assert_eq!((decoded.width, decoded.height), (4, 2));
    assert!(
        decoded
            .rgba
            .iter()
            .any(|texel| texel[0] > 4.0 || texel[1] > 4.0 || texel[2] > 4.0),
        "HDR decode must preserve radiance above the normalized LDR range"
    );
}

#[test]
fn hdr_equirect_import_stages_current_zcube_and_zribl_bundle() {
    let root = unique_temp_root("environment_ibl_source_import");
    let context = hdr_context("res://textures/studio.hdr", 128, 64, "");

    let staged = stage_environment_ibl_source(&context, &root)
        .expect("2:1 HDR should stage an environment IBL bundle");
    let request = *staged.request().expect("staged request");

    assert_eq!(staged.status(), EnvironmentIblSourceStagingStatus::Written);
    assert_eq!(request.face_size(), 64);
    assert_eq!(
        request.required_contents(),
        IblBakeArtifactContents::PMREM_SH9_IEM
    );
    assert!(staged.source_zcube_path().expect("zcube path").is_file());
    assert!(staged
        .asset_derived_path()
        .expect("asset-derived path")
        .is_file());

    let store = IblSourceCubemapStagingStore::new(&root);
    let source = store
        .read_source_cubemap_zcube(&request, context.uri.clone())
        .expect("read staged source cubemap");
    let source = match source {
        IblSourceCubemapStagingRead::Hit(source) => source,
        other => panic!("expected staged source hit, got {other:?}"),
    };
    assert!(
        source
            .texels()
            .iter()
            .any(|texel| texel[0] > 1.0 || texel[1] > 1.0 || texel[2] > 1.0),
        "staged RGBA16F source cubemap must retain HDR radiance"
    );

    let derived = store
        .asset_derived_store()
        .read_asset_derived_artifact(&request)
        .expect("read staged derived artifact");
    let blob = match derived {
        IblBakeArtifactAssetDerivedRead::Hit(blob) => blob,
        other => panic!("expected staged derived hit, got {other:?}"),
    };
    assert!(blob
        .descriptor()
        .contents()
        .contains(IblBakeArtifactContents::PMREM_SH9_IEM));

    let reused = stage_environment_ibl_source(&context, &root)
        .expect("current staged bundle should be reusable");
    assert_eq!(reused.status(), EnvironmentIblSourceStagingStatus::Reused);
    assert_eq!(reused.request(), Some(&request));

    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "stages the repository Poly Haven HDRI into docs validation artifacts"]
fn stage_polyhaven_lakes_2k_validation_bundle() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let source_path =
        workspace_root.join("docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr");
    let validation_root =
        workspace_root.join("docs/tests/runtime/shader/polyhaven_lakes_2k_staged_ibl_20260710");
    let context = AssetImportContext::new(
        source_path.clone(),
        AssetUri::parse("res://environment/polyhaven_lakes_2k.hdr").expect("valid environment URI"),
        fs::read(&source_path).expect("read Poly Haven HDRI"),
        "environment_ibl = true\nenvironment_ibl_face_size = 256"
            .parse()
            .expect("valid environment settings"),
    );

    let staged = stage_environment_ibl_source(&context, &validation_root)
        .expect("stage Poly Haven HDRI validation bundle");
    let request = *staged.request().expect("staged request");
    let store = IblSourceCubemapStagingStore::new(&validation_root);
    let environment = store
        .read_source_cubemap_environment(&request, context.uri.clone())
        .expect("restore staged Poly Haven environment");

    assert_eq!(request.face_size(), 256);
    assert_eq!(environment.mip_chain.face_size(), 256);
    assert!(environment
        .mip_chain
        .source_texels()
        .iter()
        .any(|texel| texel[0] > 1.0 || texel[1] > 1.0 || texel[2] > 1.0));
    assert_ne!(environment.bake_artifact_hash, [0; 4]);
    assert!(staged.source_zcube_path().expect("zcube path").is_file());
    assert!(staged
        .asset_derived_path()
        .expect("asset-derived path")
        .is_file());

    let report = format!(
        "status={:?}\nface_size={}\nmip_count={}\nsource_zcube={}\nasset_derived={}\nbake_artifact_hash={:08x?}\n",
        staged.status(),
        request.face_size(),
        request.mip_count(),
        staged.source_zcube_path().expect("zcube path").display(),
        staged.asset_derived_path().expect("asset-derived path").display(),
        environment.bake_artifact_hash,
    );
    fs::write(
        workspace_root
            .join("docs/tests/runtime/shader/polyhaven_lakes_2k_staged_ibl_20260710_report.txt"),
        report,
    )
    .expect("write staged IBL validation report");
}

#[test]
fn automatic_environment_import_skips_non_equirect_hdr() {
    let root = unique_temp_root("environment_ibl_non_equirect_auto");
    let context = hdr_context("res://textures/lookup.hdr", 4, 4, "");

    let result = stage_environment_ibl_source(&context, &root)
        .expect("automatic mode should leave non-equirect HDR as an ordinary texture");

    assert_eq!(result.status(), EnvironmentIblSourceStagingStatus::Skipped);
    assert!(result.request().is_none());
    assert!(!root.exists());
}

#[test]
fn explicit_environment_import_rejects_non_equirect_source() {
    let root = unique_temp_root("environment_ibl_non_equirect_explicit");
    let context = hdr_context(
        "res://textures/not_equirect.hdr",
        4,
        4,
        "environment_ibl = true",
    );

    let error = stage_environment_ibl_source(&context, &root)
        .expect_err("explicit environment mode should reject a non-2:1 source")
        .to_string();

    assert!(
        error.contains("2:1 equirectangular"),
        "unexpected error: {error}"
    );
    assert!(!root.exists());
}

fn hdr_context(uri: &str, width: u32, height: u32, settings: &str) -> AssetImportContext {
    AssetImportContext::new(
        uri.rsplit('/').next().expect("source name").into(),
        AssetUri::parse(uri).expect("valid asset URI"),
        hdr_bytes(width, height),
        settings.parse().expect("valid import settings"),
    )
}

fn hdr_bytes(width: u32, height: u32) -> Vec<u8> {
    let image = ImageBuffer::<Rgb<f32>, _>::from_fn(width, height, |x, y| {
        let u = x as f32 / width.max(1) as f32;
        let v = y as f32 / height.max(1) as f32;
        Rgb([0.25 + 8.0 * u, 0.5 + 3.0 * v, 1.0 + 5.0 * (1.0 - u)])
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgb32F(image)
        .write_to(&mut bytes, ImageFormat::Hdr)
        .expect("encode test HDR");
    bytes.into_inner()
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
