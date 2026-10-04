use super::{
    parse_cached_hdri_exposure, resolved_face_size, resolved_pmrem_face_size,
    sampled_hdri_exposure, validate_equirectangular_dimensions,
};
use zircon_runtime::asset::importer::DecodedTextureImageRgba32F;

const SOURCE: &str = include_str!("../hdri.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("HDRI source should retain a test-module boundary")
}

#[test]
fn automatic_face_size_matches_native_equirect_angular_resolution() {
    assert_eq!(resolved_face_size(None, 512), 256);
    assert_eq!(resolved_face_size(None, 1024), 512);
    assert_eq!(resolved_face_size(None, 4096), 1024);
}

#[test]
fn explicit_face_size_overrides_hdri_resolution() {
    assert_eq!(resolved_face_size(Some(128), 4096), 128);
}

#[test]
fn automatic_pmrem_result_size_matches_resolved_source_size() {
    assert_eq!(resolved_pmrem_face_size(None, 512), 512);
}

#[test]
fn explicit_pmrem_result_size_is_independent_from_source_size() {
    assert_eq!(resolved_pmrem_face_size(Some(256), 512), 256);
}

#[test]
fn viewer_accepts_standard_equirectangular_dimensions() {
    validate_equirectangular_dimensions(4096, 2048)
        .expect("a 2:1 HDRI should be accepted by the equirectangular viewer path");
}

#[test]
fn viewer_rejects_non_equirectangular_dimensions_before_staging() {
    let error = validate_equirectangular_dimensions(2048, 2048)
        .expect_err("the viewer must not project a square image as equirectangular HDRI");

    assert!(error.to_string().contains("2:1 equirectangular"));
}

#[test]
fn viewer_does_not_treat_saturated_dimensions_as_equirectangular() {
    let error = validate_equirectangular_dimensions(u32::MAX, u32::MAX)
        .expect_err("dimension arithmetic must not accept a saturated 2:1 relation");

    assert!(error.to_string().contains("2:1 equirectangular"));
}

#[test]
fn viewer_exposure_ignores_non_finite_hdr_texels() {
    let image = DecodedTextureImageRgba32F {
        width: 2,
        height: 1,
        rgba: vec![[f32::NAN, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],
    };

    let exposure = sampled_hdri_exposure(&image);

    assert!(exposure.is_finite());
    assert!((exposure - 0.45).abs() < 0.0001);
}

#[test]
fn viewer_exposure_has_a_deterministic_fallback_for_all_invalid_texels() {
    let image = DecodedTextureImageRgba32F {
        width: 2,
        height: 1,
        rgba: vec![[f32::NAN, f32::INFINITY, 0.0, 1.0]; 2],
    };

    let exposure = sampled_hdri_exposure(&image);

    assert_eq!(exposure, 4.0);
}

#[test]
fn viewer_hdri_staging_requires_a_runtime_owned_parallel_executor() {
    let source = production_source();
    assert!(source.contains("parallel_executor: &E"));
    assert!(
        source.contains("stage_environment_ibl_source_with_parallel_executor_and_decoded_image(")
    );
    assert!(source.contains("restore_environment_ibl_source_if_current("));
    assert!(source.contains("DecodedTextureImageRgba32F"));
    assert!(source.contains("let staging_started = Instant::now();"));
    assert!(source.contains("staging_elapsed"));
    assert!(
        !source.contains("stage_environment_ibl_source(&context"),
        "the interactive viewer must not fall back to serial IBL staging"
    );
}

#[test]
fn viewer_staging_consumes_the_predecoded_hdr_input_without_a_second_decode() {
    let source = production_source();
    let staging = source
        .split("pub(crate) fn source_cubemap_environment<E>(")
        .nth(1)
        .expect("viewer must retain a dedicated source-cubemap staging owner");

    assert!(source.contains("pub(crate) fn preflight_viewer_hdri("));
    assert!(staging.contains("hdri: ViewerHdriPreflight"));
    assert!(staging.contains("let DecodedViewerHdri { image, exposure }"));
    assert!(staging.contains("let DecodedViewerHdri {"));
    assert!(
        !staging.contains("fs::read("),
        "staging must consume the preflight bytes instead of reading the HDRI again"
    );
    assert!(
        !staging.contains("load_from_memory_with_format"),
        "staging must consume the preflight image instead of decoding it again"
    );
}

#[test]
fn cached_hdri_exposure_requires_the_current_schema_and_finite_clamped_bits() {
    assert_eq!(
        parse_cached_hdri_exposure(
            "schema=zircon_shader_pbr_viewer_hdri_exposure_v1\nexposure_bits=3f000000\n"
        ),
        Some(0.5)
    );
    assert_eq!(
        parse_cached_hdri_exposure(
            "schema=zircon_shader_pbr_viewer_hdri_exposure_v1\nexposure_bits=7fc00000\n"
        ),
        None
    );
    assert_eq!(
        parse_cached_hdri_exposure("schema=old\nexposure_bits=3f000000\n"),
        None
    );
}

#[test]
fn viewer_exposure_sidecar_uses_runtime_atomic_publication() {
    let source = production_source();
    let sidecar_writer = source
        .split("fn write_cached_hdri_exposure(")
        .nth(1)
        .and_then(|writer| writer.split("fn read_cached_hdri_exposure(").next())
        .expect("viewer must retain an exposure sidecar writer");

    assert!(source.contains("core::resource::io::atomic_write"));
    assert!(sidecar_writer.contains("atomic_write("));
    assert!(!sidecar_writer.contains("fs::write("));
}

#[test]
fn valid_ibl_restore_survives_a_missing_viewer_exposure_sidecar() {
    let source = production_source();
    let after_decode = source
        .split("let DecodedViewerHdri { image, exposure } = decode_viewer_hdri(&context)?;")
        .nth(1)
        .expect("viewer must retain a decoded HDRI fallback");
    let restored_environment = after_decode
        .find("if let Some(restored) = restored_without_exposure {")
        .expect("a valid restored bundle must survive a missing exposure sidecar");
    let staging = after_decode
        .find("let staging_started = Instant::now();")
        .expect("a genuine cache miss must retain staging");

    assert!(
        source.contains("let mut restored_without_exposure = None;"),
        "the valid restored bundle must remain available while exposure is decoded"
    );
    assert!(
        source.contains("restored_without_exposure = Some(restored);"),
        "a missing exposure sidecar must not discard a valid restored bundle"
    );
    assert!(
        restored_environment < staging,
        "the restored bundle must be returned before cache-miss staging can run"
    );
}

#[test]
fn viewer_ibl_total_time_excludes_unrelated_startup_and_exposure_sidecar_io() {
    let source = production_source();
    let staging = source
        .split("pub(crate) fn source_cubemap_environment<E>(")
        .nth(1)
        .expect("viewer must retain a dedicated source-cubemap staging owner");
    let exposure_write = staging
        .find("write_cached_hdri_exposure(source_zcube_path, exposure)")
        .expect("the cold staging path must persist exposure separately");
    let hydration_started = staging
        .find("let hydration_started = Instant::now();")
        .expect("artifact hydration must have an explicit timing boundary");

    assert!(
        exposure_write < hydration_started,
        "Viewer-local exposure persistence must not inflate artifact hydration"
    );
    assert!(staging.contains("let hydration_elapsed = hydration_started.elapsed();"));
    assert!(
        staging.contains("let total_elapsed = staging_elapsed.saturating_add(hydration_elapsed);")
    );
    assert!(!staging.contains("let total_elapsed = staging_started.elapsed();"));
    assert!(!source.contains("load_started_at"));
}
