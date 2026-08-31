use std::path::Path;
use std::time::Instant;

use super::{
    decode_texture_source_image_metadata, environment_ibl_import_mode,
    environment_ibl_request_for_source_image, source_image_identity, AssetImportContext,
    EnvironmentIblImportMode, EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingOutput,
    EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming, EnvironmentIblStagingPhase, IblSourceCubemapStagingStore,
    PreparedEnvironmentIblSourceStaging,
};

pub(super) enum EnvironmentIblWarmCacheProbe {
    Finished(PreparedEnvironmentIblSourceStaging),
    Miss {
        source_image: crate::asset::artifact::IblSourceImageIdentity,
        timing: EnvironmentIblSourceStagingTiming,
    },
}

pub(super) fn probe_environment_ibl_warm_cache(
    context: &AssetImportContext,
    cache_root: &Path,
) -> Result<EnvironmentIblWarmCacheProbe, EnvironmentIblSourceStagingError> {
    let classify_started = Instant::now();
    let mode = {
        let _phase = EnvironmentIblStagingPhase::SourceClassify.enter();
        environment_ibl_import_mode(context)?
    };
    let source_classify = classify_started.elapsed();
    let store = IblSourceCubemapStagingStore::new(cache_root);
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(EnvironmentIblWarmCacheProbe::Finished(
            PreparedEnvironmentIblSourceStaging {
                store,
                writes: Vec::new(),
                report: EnvironmentIblSourceStagingReport::skipped(),
            },
        ));
    }

    let metadata_started = Instant::now();
    let metadata = {
        let _phase = EnvironmentIblStagingPhase::SourceDecode.enter();
        decode_texture_source_image_metadata(context)
            .map_err(EnvironmentIblSourceStagingError::Decode)?
    };
    let source_image = source_image_identity(metadata);
    let mut timing = EnvironmentIblSourceStagingTiming {
        source_classify,
        source_decode: metadata_started.elapsed(),
        ..Default::default()
    };
    let identity_started = Instant::now();
    let request = {
        let _phase = EnvironmentIblStagingPhase::SourceIdentity.enter();
        environment_ibl_request_for_source_image(context, source_image)?
    };
    timing.source_identity = identity_started.elapsed();
    let Some(request) = request else {
        return Ok(EnvironmentIblWarmCacheProbe::Finished(
            PreparedEnvironmentIblSourceStaging {
                store,
                writes: Vec::new(),
                report: EnvironmentIblSourceStagingReport::skipped(),
            },
        ));
    };

    let source_path = store.source_cubemap_path(&request);
    let derived_path = store.asset_derived_store().asset_derived_path(&request);
    let probe_started = Instant::now();
    let current = {
        let _phase = EnvironmentIblStagingPhase::CacheProbe.enter();
        store.current_bundle_manifest_matches(&request, source_image)?
    };
    timing.cache_probe = probe_started.elapsed();
    if !current {
        return Ok(EnvironmentIblWarmCacheProbe::Miss {
            source_image,
            timing,
        });
    }

    let output = EnvironmentIblSourceStagingOutput::from_reused_paths(&source_path, &derived_path)?;
    Ok(EnvironmentIblWarmCacheProbe::Finished(
        PreparedEnvironmentIblSourceStaging {
            store,
            writes: Vec::new(),
            report: EnvironmentIblSourceStagingReport::current(
                EnvironmentIblSourceStagingStatus::Reused,
                request,
                source_path,
                derived_path,
                timing,
                output,
            ),
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use crate::asset::AssetUri;

    use super::super::{
        stage_environment_ibl_source, AssetImportContext, EnvironmentIblSourceStagingStatus,
        IblSourceCubemapStagingStore,
    };

    const ORCHESTRATOR: &str = include_str!("../environment_ibl.rs");
    const STAGING_STORE: &str = include_str!("../../artifact/ibl_source_cubemap_staging.rs");

    #[test]
    fn warm_probe_precedes_full_rgba32f_decode_in_both_staging_entries() {
        for entry in [
            "pub(crate) fn prepare_environment_ibl_source(",
            "pub(crate) fn prepare_environment_ibl_source_with_parallel_executor<E>(",
        ] {
            let body = ORCHESTRATOR
                .split(entry)
                .nth(1)
                .expect("staging entry must remain present")
                .split("prepare_environment_ibl_source_with_builder(")
                .next()
                .expect("staging entry must end at the shared builder call");
            let probe = body
                .find("probe_environment_ibl_warm_cache")
                .expect("staging must probe its metadata cache");
            let decode = body
                .find("decode_texture_source_image_rgba32f")
                .expect("a cache miss must retain full HDR decode");

            assert!(
                probe < decode,
                "warm cache probing must precede pixel decode"
            );
        }
    }

    #[test]
    fn caller_decoded_image_probes_manifest_before_payload_hydration() {
        let caller_decoded_entry = ORCHESTRATOR
            .split(
                "pub fn stage_environment_ibl_source_with_parallel_executor_and_decoded_image<E>",
            )
            .nth(1)
            .expect("caller-decoded staging entry must remain present")
            .split("fn prepare_environment_ibl_source_with_builder")
            .next()
            .expect("caller-decoded staging entry must end at the shared builder");
        assert!(caller_decoded_entry.contains("EnvironmentIblBundleProbeState::Required"));

        let builder = ORCHESTRATOR
            .split("fn prepare_environment_ibl_source_with_builder")
            .nth(1)
            .expect("shared staging builder must remain present")
            .split("enum EnvironmentIblStagedBundleState")
            .next()
            .expect("shared staging builder must end before bundle state declarations");
        let manifest_probe = builder
            .find("current_bundle_manifest_matches")
            .expect("caller-decoded warm hits must use the manifest-only probe");
        let payload_hydration = builder
            .find("staged_bundle_state")
            .expect("cache misses must retain staged payload hydration");

        assert!(manifest_probe < payload_hydration);
        assert!(builder.contains("EnvironmentIblBundleProbeState::AlreadyMissed"));
        assert!(builder.contains("timing.cache_probe = timing"));
        assert!(builder.contains(".saturating_add(cache_probe_started.elapsed())"));
    }

    #[test]
    fn current_manifest_probe_does_not_read_or_hash_payload_bytes() {
        let body = STAGING_STORE
            .split("pub(crate) fn current_bundle_manifest_matches(")
            .nth(1)
            .expect("metadata cache probe must remain present")
            .split("fn read_source_cubemap_environment_with_snapshot_hooks")
            .next()
            .expect("metadata probe must end before hydration");

        assert!(body.contains("regular_file_has_len"));
        assert!(!body.contains("read_source_cubemap_bytes"));
        assert!(!body.contains("read_asset_derived_bytes"));
        assert!(!body.contains("matches_bytes"));
    }

    #[test]
    #[ignore = "release-only current-source Poly Haven 2K warm-cache profile"]
    fn shader06_p0_9_polyhaven_2k_warm_cache_latency_gate() {
        const SAMPLE_COUNT: usize = 31;
        const P50_LIMIT_US: u128 = 39_050;
        const P95_LIMIT_US: u128 = 73_590;

        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr");
        let source_bytes = std::fs::read(&source_path).expect("Poly Haven 2K fixture must exist");
        assert_eq!(source_bytes.len(), 5_918_432);
        let context = AssetImportContext::new(
            source_path,
            AssetUri::parse("res://environment/polyhaven_lakes_2k.hdr")
                .expect("profile URI must parse"),
            source_bytes,
            "environment_ibl = true\nenvironment_ibl_face_size = 16\nenvironment_ibl_pmrem_face_size = 8"
                .parse()
                .expect("profile settings must parse"),
        );
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow the Unix epoch")
            .as_nanos();
        let cache_root = PathBuf::from(format!(
            "E:/zircon-profiles/shader06-p0-9-warm-cache-current-source-{}-{nonce}",
            std::process::id()
        ));

        let cold = stage_environment_ibl_source(&context, &cache_root)
            .expect("profile setup must materialize one current bundle");
        assert_eq!(cold.status(), EnvironmentIblSourceStagingStatus::Written);
        let request = *cold
            .request()
            .expect("written cache must retain its request");
        let store = IblSourceCubemapStagingStore::new(&cache_root);
        assert_eq!(
            std::fs::metadata(store.bundle_manifest_path(&request))
                .expect("current bundle manifest must exist")
                .len(),
            252
        );

        let mut samples_us = Vec::with_capacity(SAMPLE_COUNT);
        for _ in 0..SAMPLE_COUNT {
            let started = Instant::now();
            let warm = stage_environment_ibl_source(&context, &cache_root)
                .expect("current bundle must remain reusable");
            samples_us.push(started.elapsed().as_micros());
            assert_eq!(warm.status(), EnvironmentIblSourceStagingStatus::Reused);
            assert_eq!(warm.timing().cubemap_build().as_nanos(), 0);
            assert_eq!(warm.timing().bundle_encode().as_nanos(), 0);
        }
        let p50_us = percentile_us(&samples_us, 50);
        let p95_us = percentile_us(&samples_us, 95);
        println!(
            "PERF_RESULT shader06_p0_9_polyhaven_2k_warm_cache sample_count={SAMPLE_COUNT} source_bytes=5918432 manifest_bytes=252 p50_us={p50_us} p95_us={p95_us} p50_limit_us={P50_LIMIT_US} p95_limit_us={P95_LIMIT_US} raw_us={}",
            samples_us
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(
            p50_us <= P50_LIMIT_US && p95_us <= P95_LIMIT_US,
            "warm-cache latency exceeds the recorded 25% baseline gate: p50={p50_us}us p95={p95_us}us"
        );

        std::fs::remove_dir_all(cache_root).expect("profile cache root must be removable");
    }

    fn percentile_us(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }
}
