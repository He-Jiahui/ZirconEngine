use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use crate::{
    asset::{AssetEvent, Handle, SceneAsset},
    core::resource::ResourceId,
    scene::dynamic_scene::DynamicSceneError,
};

use super::DynamicSceneAssetReloadResult;

const PERF_SAMPLE_PAIRS: usize = 21;

fn error_result(reason: String) -> DynamicSceneAssetReloadResult {
    let event = AssetEvent::Modified {
        handle: Handle::<SceneAsset>::new(ResourceId::from_stable_label(
            "reload result size cache",
        )),
        locator: None,
        revision: 7,
    };
    DynamicSceneAssetReloadResult::new(event, Err(DynamicSceneError::Parse { reason }))
}

#[inline(never)]
fn legacy_estimated_bytes(result: &DynamicSceneAssetReloadResult, reads: usize) -> usize {
    (0..reads).fold(0usize, |sum, _| {
        let bytes = match black_box(result.result()) {
            Ok(prepared) => prepared.estimated_bytes(),
            Err(error) => std::mem::size_of::<DynamicSceneAssetReloadResult>()
                .saturating_add(error.to_string().len()),
        };
        sum.wrapping_add(black_box(bytes))
    })
}

#[inline(never)]
fn cached_estimated_bytes(result: &DynamicSceneAssetReloadResult, reads: usize) -> usize {
    (0..reads).fold(0usize, |sum, _| {
        sum.wrapping_add(black_box(result.estimated_bytes()))
    })
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn dynamic_scene_asset_reload_oversized_result_becomes_bounded_failure() {
    let result = error_result("x".repeat(8 * 1024)).bounded_to(1_024);

    assert!(result.estimated_bytes() <= 1_024);
    assert!(matches!(
        result.result(),
        Err(DynamicSceneError::ReloadResultTooLarge { .. })
    ));
}

#[test]
fn dynamic_scene_asset_reload_result_reuses_cached_size_estimate() {
    let result = error_result("cached-size".repeat(512));
    let expected = legacy_estimated_bytes(&result, 1);

    assert_eq!(result.estimated_bytes(), expected);
    assert_eq!(result.estimated_bytes(), expected);
}

#[test]
#[ignore = "managed Runtime53 performance evidence"]
fn dynamic_scene_asset_reload_runtime53_performance_cached_result_size() {
    const READS_PER_SAMPLE: usize = 1_024;
    let result = error_result("cached-size-benchmark".repeat(256));
    assert_eq!(
        legacy_estimated_bytes(&result, 1),
        cached_estimated_bytes(&result, 1)
    );

    black_box(legacy_estimated_bytes(&result, READS_PER_SAMPLE));
    black_box(cached_estimated_bytes(&result, READS_PER_SAMPLE));

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut cached_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair in 0..PERF_SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            black_box(legacy_estimated_bytes(&result, READS_PER_SAMPLE));
            legacy_samples.push(started.elapsed());
        };
        let mut measure_cached = || {
            let started = Instant::now();
            black_box(cached_estimated_bytes(&result, READS_PER_SAMPLE));
            cached_samples.push(started.elapsed());
        };
        if pair % 2 == 0 {
            measure_legacy();
            measure_cached();
        } else {
            measure_cached();
            measure_legacy();
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let cached_p50 = nearest_rank(&cached_samples, 50);
    let cached_p95 = nearest_rank(&cached_samples, 95);
    let legacy_ns = legacy_samples
        .iter()
        .map(Duration::as_nanos)
        .collect::<Vec<_>>();
    let cached_ns = cached_samples
        .iter()
        .map(Duration::as_nanos)
        .collect::<Vec<_>>();
    let legacy_csv = legacy_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let cached_csv = cached_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");

    eprintln!(
        "RUNTIME53_RESULT_SIZE_CACHE_BENCH_V1 sample_pairs={PERF_SAMPLE_PAIRS} reads_per_sample={READS_PER_SAMPLE} pair_order=alternating_legacy_even legacy_p50_ns={} legacy_p95_ns={} cached_p50_ns={} cached_p95_ns={} legacy_ns={legacy_csv} cached_ns={cached_csv}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        cached_p50.as_nanos(),
        cached_p95.as_nanos(),
    );
    assert!(
        cached_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(20),
        "cached result-size reads must reduce P95 by at least 80%: legacy={legacy_p95:?}, cached={cached_p95:?}"
    );
}
