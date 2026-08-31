use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::import_settings::requested_artifact_contents_from_value;
use super::restore::{recover_source_restore_error, source_restore_is_rebuildable_cache_miss};
use super::{
    environment_ibl_request_for_dimensions, sample_equirect_bilinear, DecodedTextureImageRgba32F,
    EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingTiming,
    MeasuredParallelSliceExecutor,
};
use crate::core::framework::render::IblBakeArtifactContents;
use crate::core::framework::tasks::ParallelSliceExecutor;

const SOURCE: &str = include_str!("../environment_ibl.rs");
const SOURCE_CUBEMAP_TEXTURE_MODULE: &str = include_str!("source_cubemap_texture.rs");
const SOURCE_STAGING_MODULE: &str = include_str!("source_staging/mod.rs");
const SOURCE_STAGING_PHASE_MODULE: &str = include_str!("source_staging/phase.rs");
const SOURCE_STAGING_REPORT_MODULE: &str = include_str!("source_staging/report.rs");
const TEST_SOURCE: &str = include_str!("tests.rs");

#[test]
fn source_cubemap_texture_import_is_child_owned() {
    assert!(SOURCE.contains("mod source_cubemap_texture;"));
    assert!(SOURCE.contains("pub use source_cubemap_texture::{"));
    assert!(SOURCE.contains("stage_source_cubemap_texture"));
    assert!(SOURCE.contains("prepare_source_cubemap_texture"));
    for function in [
        "pub fn stage_external_source_cubemap_texture(",
        "pub(crate) fn prepare_external_source_cubemap_texture(",
    ] {
        assert!(SOURCE_CUBEMAP_TEXTURE_MODULE.contains(function));
        assert!(!SOURCE.contains(function));
    }
    assert!(SOURCE.contains("#[cfg(test)]\nmod tests;"));
    assert!(!SOURCE.contains("#[cfg(test)]\nmod tests {"));
    assert!(SOURCE.lines().count() < 800);
    assert!(SOURCE_CUBEMAP_TEXTURE_MODULE.lines().count() < 280);
    assert!(TEST_SOURCE.lines().count() < 500);
}

#[test]
fn captured_zcube_is_classified_without_pixel_decode() {
    use super::source_cubemap_texture::{source_cubemap_texture_kind, SourceCubemapTextureKind};
    use crate::asset::assets::texture_asset_from_source_cubemap_zcube_rgba16f_mips;
    use crate::asset::AssetUri;

    let face_size = 2;
    let mip_count = 2;
    let rgba16f = vec![0_u8; 6 * (2 * 2 + 1) * 8];
    let texture = texture_asset_from_source_cubemap_zcube_rgba16f_mips(
        AssetUri::parse("res://captures/probe.zcube").expect("test URI should parse"),
        face_size,
        mip_count,
        &rgba16f,
    )
    .expect("complete source mips should encode");

    assert!(matches!(
        source_cubemap_texture_kind(&texture).expect("captured zcube should classify"),
        Some(SourceCubemapTextureKind::CapturedZcube {
            face_size: 2,
            mip_count: 2,
        })
    ));

    let classifier = SOURCE_CUBEMAP_TEXTURE_MODULE
        .split("pub(super) fn source_cubemap_texture_kind(")
        .nth(1)
        .expect("source-cubemap texture classifier should exist")
        .split("/// Convert a cmft-style DDS/KTX source cubemap")
        .next()
        .expect("classifier should end before staging entry points");
    assert!(classifier.contains("zcube_source_cubemap_texture_info"));
    assert!(!classifier.contains("decode_zcube_source_cubemap_texture"));
    assert!(!classifier.contains("decode_rgba16f_texels"));
}

#[test]
fn source_staging_contract_is_isolated_from_import_orchestration() {
    assert!(SOURCE.contains("mod source_staging;"));
    assert!(SOURCE.contains("pub use source_staging::{"));
    for type_name in [
        "EnvironmentIblSourceStagingError",
        "EnvironmentIblSourceStagingOutput",
        "EnvironmentIblSourceStagingReport",
        "EnvironmentIblSourceStagingStatus",
        "EnvironmentIblSourceStagingTiming",
    ] {
        assert!(
            SOURCE_STAGING_MODULE.contains(type_name),
            "source staging module must retain `{type_name}`"
        );
        assert!(
            !SOURCE.contains(&format!("pub struct {type_name}"))
                && !SOURCE.contains(&format!("pub enum {type_name}")),
            "entry orchestration must not redefine `{type_name}`"
        );
    }
}

#[test]
fn environment_staging_defaults_to_pmrem_and_sh9_without_iem() {
    assert_eq!(
        requested_artifact_contents_from_value(None)
            .expect("omitted IEM setting should use the MVP artifact"),
        IblBakeArtifactContents::PMREM_SH9
    );
    assert_eq!(
        requested_artifact_contents_from_value(Some(&toml::Value::Boolean(false)))
            .expect("explicitly disabled IEM should use the MVP artifact"),
        IblBakeArtifactContents::PMREM_SH9
    );
}

#[test]
fn environment_staging_requires_explicit_boolean_iem_opt_in() {
    assert_eq!(
        requested_artifact_contents_from_value(Some(&toml::Value::Boolean(true)))
            .expect("IEM opt-in should be accepted"),
        IblBakeArtifactContents::PMREM_SH9_IEM
    );
    assert!(
        requested_artifact_contents_from_value(Some(&toml::Value::String("true".into()))).is_err()
    );
}

#[test]
fn known_dimensions_preserve_the_canonical_source_request_identity() {
    let context = crate::asset::importer::AssetImportContext::new(
        "sunset.hdr".into(),
        crate::asset::AssetUri::parse("res://environment/sunset.hdr")
            .expect("test URI should parse"),
        b"unchanged source bytes".to_vec(),
        "environment_ibl = true\nenvironment_ibl_face_size = 128\nenvironment_ibl_pmrem_face_size = 64"
            .parse()
            .expect("test settings should parse"),
    );
    let request = environment_ibl_request_for_dimensions(&context, 1024, 512)
        .expect("valid equirectangular dimensions should build a request")
        .expect("enabled IBL should retain a request");

    assert_eq!(request.source_face_size(), 128);
    assert_eq!(request.source_mip_count(), 8);
    assert_eq!(request.pmrem_face_size(), 64);
    assert_eq!(request.pmrem_mip_count(), 7);
    assert_eq!(
        request.required_contents(),
        IblBakeArtifactContents::PMREM_SH9
    );
}

#[test]
fn canonical_request_rejects_saturated_equirectangular_dimensions() {
    let context = crate::asset::importer::AssetImportContext::new(
        "overflow.hdr".into(),
        crate::asset::AssetUri::parse("res://environment/overflow.hdr")
            .expect("test URI should parse"),
        b"unchanged source bytes".to_vec(),
        "environment_ibl = true"
            .parse()
            .expect("test settings should parse"),
    );

    assert!(environment_ibl_request_for_dimensions(&context, u32::MAX, u32::MAX).is_err());
}

#[test]
fn source_restore_only_swallows_known_rebuildable_cache_misses() {
    use crate::asset::artifact::IblSourceCubemapStagingError;

    for error in [
        IblSourceCubemapStagingError::MissingSourceCubemap,
        IblSourceCubemapStagingError::MissingAssetDerived,
        IblSourceCubemapStagingError::MissingBundleManifest,
        IblSourceCubemapStagingError::BundlePayloadStampMismatch {
            payload: "asset-derived.zribl",
        },
    ] {
        assert!(source_restore_is_rebuildable_cache_miss(&error));
    }
    let apply_error = IblSourceCubemapStagingError::ApplyAssetDerived(
        crate::core::framework::render::SourceCubemapBakeArtifactError::MissingPmrem,
    );
    assert!(!source_restore_is_rebuildable_cache_miss(&apply_error));
}

#[test]
fn apply_asset_derived_restore_failure_invalidates_the_manifest_and_derived_artifact() {
    use crate::asset::artifact::IblSourceCubemapStagingError;
    use crate::core::framework::render::SourceCubemapBakeArtifactError;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-environment-ibl-apply-error-{}-{nonce}",
        std::process::id()
    ));
    let source_zcube_path = root.join("current.zcube");
    let asset_derived_path = root.join("broken.zribl");
    let bundle_manifest_path = root.join("bundle.zriblmeta");
    std::fs::create_dir_all(&root).expect("test cache directory should be created");
    std::fs::write(&source_zcube_path, b"current source cubemap")
        .expect("test source cubemap should be written");
    std::fs::write(&asset_derived_path, b"bad derived artifact")
        .expect("test derived artifact should be written");
    std::fs::write(&bundle_manifest_path, b"previous current marker")
        .expect("test bundle manifest should be written");

    let recovery = recover_source_restore_error(
        IblSourceCubemapStagingError::ApplyAssetDerived(
            SourceCubemapBakeArtifactError::MissingPmrem,
        ),
        &source_zcube_path,
        &asset_derived_path,
        &bundle_manifest_path,
    );

    assert!(recovery.is_ok());
    assert!(
        !asset_derived_path.exists(),
        "the invalid derived artifact must be removed before fallback staging"
    );
    assert!(
        !bundle_manifest_path.exists(),
        "the current marker must be invalidated with its bad payload"
    );
    assert!(
        source_zcube_path.exists(),
        "the current source cubemap must remain available for derived-only rebuild"
    );
    std::fs::remove_dir_all(root).expect("test cache directory should be removed");
}

#[test]
fn environment_equirect_bilinear_sampling_clamps_poles_to_edge_rows() {
    let image = DecodedTextureImageRgba32F {
        width: 2,
        height: 2,
        rgba: vec![
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
        ],
    };

    assert_eq!(
        sample_equirect_bilinear(&image, 0.25, 0.0),
        [1.0, 0.0, 0.0, 1.0]
    );
    assert_eq!(
        sample_equirect_bilinear(&image, 0.25, 1.0),
        [0.0, 1.0, 0.0, 1.0]
    );
}

#[test]
fn environment_equirect_bilinear_sampling_wraps_the_horizontal_seam() {
    let image = DecodedTextureImageRgba32F {
        width: 2,
        height: 1,
        rgba: vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]],
    };

    assert_eq!(
        sample_equirect_bilinear(&image, 0.0, 0.5),
        [0.5, 0.5, 0.0, 1.0]
    );
    assert_eq!(
        sample_equirect_bilinear(&image, 1.0, 0.5),
        [0.5, 0.5, 0.0, 1.0]
    );
}

#[test]
fn parallel_environment_staging_uses_its_executor_for_iem_bake() {
    let parallel_staging = SOURCE
        .split("pub fn stage_environment_ibl_source_with_parallel_executor_and_decoded_image")
        .nth(1)
        .expect("parallel environment staging entry point should exist")
        .split("fn prepare_environment_ibl_source_with_builder")
        .next()
        .expect("parallel environment staging should end before its shared builder");

    assert!(
        parallel_staging.contains(
            "build_source_cubemap_irradiance_cube_with_parallel_executor(cubemap, &irradiance_cube_executor)"
        ),
        "parallel environment staging must keep optional IEM convolution on the caller executor"
    );
    assert!(parallel_staging.contains("MeasuredParallelSliceExecutor"));
}

#[test]
fn derived_only_staging_prepares_without_rewriting_the_existing_source_file() {
    let output_writer = SOURCE
        .split("fn prepare_environment_ibl_staged_outputs")
        .nth(1)
        .expect("shared staging output writer should exist")
        .split("fn staged_bundle_state")
        .next()
        .expect("output writer should end before staged bundle inspection");
    let reused_source_branch = output_writer
        .split("if source_was_reused")
        .nth(1)
        .expect("derived-only output branch should exist")
        .split("let (writes, bundle)")
        .next()
        .expect("new-source bundle write should follow the reuse branch");

    assert!(reused_source_branch.contains("prepare_source_cubemap_asset_derived_artifact"));
    assert!(!reused_source_branch.contains("write_source_cubemap_staged_bundle"));
    assert!(!reused_source_branch.contains("commit_prepared_bundle_writes"));
    assert!(output_writer.contains("prepare_source_cubemap_staged_bundle"));
    assert!(!output_writer.contains("write_source_cubemap_staged_bundle"));
}

#[derive(Default)]
struct SerialParallelSliceExecutor;

impl ParallelSliceExecutor for SerialParallelSliceExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        for chunk in items.chunks_mut(chunk_size.max(1)) {
            task(chunk);
        }
    }
}

#[test]
fn measured_parallel_executor_reports_submitted_chunk_work() {
    let inner = SerialParallelSliceExecutor;
    let work_items = AtomicUsize::new(0);
    let executor = MeasuredParallelSliceExecutor {
        inner: &inner,
        work_items: &work_items,
    };
    let mut values = [0_u32; 5];

    executor.parallel_for(&mut values, 2, |chunk| {
        for value in chunk {
            *value += 1;
        }
    });

    assert_eq!(values, [1; 5]);
    assert_eq!(work_items.load(Ordering::Relaxed), 3);
}

#[test]
fn environment_staging_reports_subphases_without_double_counting_them() {
    let builder = SOURCE
        .split("fn prepare_environment_ibl_source_with_builder")
        .nth(1)
        .expect("shared environment staging builder should exist")
        .split("enum EnvironmentIblStagedBundleState")
        .next()
        .expect("shared environment staging builder should end before external import");

    for phase in [
        "equirect_projection",
        "source_mip_build",
        "pmrem_build",
        "sh9_build",
    ] {
        assert!(
            builder.contains(&format!("timing.{phase} = cubemap_timing.{phase}()")),
            "shared staging builder must copy {phase} from framework attribution"
        );
    }
    assert!(
        builder.contains(
            "timing.source_classify = timing.source_classify.saturating_add(classify_started.elapsed())"
        ),
        "the shared builder must attribute its compatibility classification pass"
    );

    let timing = EnvironmentIblSourceStagingTiming {
        source_classify: Duration::from_millis(2),
        source_identity: Duration::from_millis(5),
        cache_probe: Duration::from_millis(7),
        source_decode: Duration::from_millis(3),
        cubemap_build: Duration::from_millis(48),
        equirect_projection: Duration::from_millis(7),
        source_mip_build: Duration::from_millis(11),
        pmrem_build: Duration::from_millis(13),
        sh9_build: Duration::from_millis(17),
        irradiance_cube_build: Duration::from_millis(19),
        bundle_encode: Duration::from_millis(20),
        bundle_commit: Duration::from_millis(3),
    };

    assert_eq!(timing.source_classify(), Duration::from_millis(2));
    assert_eq!(timing.source_identity(), Duration::from_millis(5));
    assert_eq!(timing.cache_probe(), Duration::from_millis(7));
    assert_eq!(timing.equirect_projection(), Duration::from_millis(7));
    assert_eq!(timing.source_mip_build(), Duration::from_millis(11));
    assert_eq!(timing.pmrem_build(), Duration::from_millis(13));
    assert_eq!(timing.sh9_build(), Duration::from_millis(17));
    assert_eq!(timing.bundle_encode(), Duration::from_millis(20));
    assert_eq!(timing.bundle_commit(), Duration::from_millis(3));
    assert_eq!(timing.bundle_write(), Duration::from_millis(23));
    let output = EnvironmentIblSourceStagingOutput {
        source_zcube_bytes: 1_024,
        asset_derived_bytes: 2_048,
        equirect_projection_parallel_work_items: 6,
        source_mip_build_parallel_work_items: 12,
        pmrem_build_parallel_work_items: 24,
        irradiance_cube_build_parallel_work_items: 0,
        irradiance_cube_source_sample_visits: 37_748_736,
    };
    assert_eq!(output.source_zcube_bytes(), 1_024);
    assert_eq!(output.asset_derived_bytes(), 2_048);
    assert_eq!(output.parallel_executor_work_items(), 42);
    assert_eq!(output.equirect_projection_parallel_work_items(), 6);
    assert_eq!(output.source_mip_build_parallel_work_items(), 12);
    assert_eq!(output.pmrem_build_parallel_work_items(), 24);
    assert_eq!(output.irradiance_cube_build_parallel_work_items(), 0);
    assert_eq!(
        output.irradiance_cube_source_sample_visits(),
        37_748_736,
        "direct IEM throughput must retain its actual source-sample visit count"
    );
    assert_eq!(
        super::irradiance_cube_source_sample_visits_for_layout(64, 7, 32),
        37_748_736,
        "the canonical 32x32 source mip must report every direct IEM visit"
    );
    assert_eq!(
        super::irradiance_cube_source_sample_visits_for_layout(16, 5, 32),
        9_437_184,
        "a source below the diffuse cap must not be reported as a 32x32 source"
    );
    assert_eq!(
        timing.total(),
        Duration::from_millis(107),
        "cubemap_build owns its diagnostic subphases and total must not add them again"
    );
}

#[test]
fn environment_staging_profile_names_match_the_cold_path_cost_model() {
    for phase in [
        "source_classify",
        "source_identity",
        "cache_probe",
        "source_decode",
        "cubemap_build",
        "irradiance_cube_build",
        "bundle_encode",
        "bundle_commit",
    ] {
        assert!(
            SOURCE_STAGING_PHASE_MODULE.contains(&format!("\"{phase}\"")),
            "missing production IBL staging profile phase {phase}"
        );
    }
    for counter in [
        "asset.environment_ibl.source_zcube_bytes",
        "asset.environment_ibl.asset_derived_bytes",
        "asset.environment_ibl.source_identity_us",
        "asset.environment_ibl.cache_probe_us",
        "asset.environment_ibl.bundle_encode_us",
        "asset.environment_ibl.bundle_commit_us",
    ] {
        assert!(
            SOURCE_STAGING_REPORT_MODULE.contains(counter),
            "missing production IBL staging profile counter {counter}"
        );
    }
    assert!(SOURCE_STAGING_REPORT_MODULE.contains("record_counter_batch"));

    let external_decode = SOURCE_CUBEMAP_TEXTURE_MODULE
        .rsplit("SourceCubemapTextureKind::External(info) =>")
        .next()
        .expect("external source cubemap decode branch should exist");
    assert!(external_decode.contains("decode_external_source_cubemap_texels(texture, info)"));
    assert!(!external_decode.contains("decode_external_source_cubemap(texture)"));
}
