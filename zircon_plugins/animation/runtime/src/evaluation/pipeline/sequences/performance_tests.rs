use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::asset::AssetId;
use zircon_runtime::core::resource::ResourceId;

use super::{requested_asset_is_current, sorted_requested_assets};

const ASSET_COUNT: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

#[test]
fn sorted_requested_asset_ids_preserve_membership_and_remove_duplicates() {
    let first = ResourceId::new();
    let second = ResourceId::new();
    let requested = sorted_requested_assets([second, first, second]);

    assert_eq!(requested.len(), 2);
    assert!(requested_asset_is_current(&requested, &first));
    assert!(requested_asset_is_current(&requested, &second));
    assert!(!requested_asset_is_current(&requested, &ResourceId::new()));
}

#[test]
#[ignore = "release-only performance gate"]
fn sorted_requested_asset_ids_release_benchmark_evidence() {
    let assets = (0..ASSET_COUNT)
        .map(|_| ResourceId::new())
        .collect::<Vec<_>>();
    let (legacy_samples, optimized_samples) = paired_samples(
        || legacy_requested_assets(&assets),
        || sorted_requested_assets(assets.iter().copied()),
    );
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime170_sorted_sequence_asset_membership assets={ASSET_COUNT} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_tree_nodes={ASSET_COUNT} optimized_tree_nodes=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT} legacy_raw_ns={} optimized_raw_ns={}",
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
    );
    assert_eq!(legacy_requested_assets(&assets).len(), ASSET_COUNT);
    assert_eq!(sorted_requested_assets(assets).len(), ASSET_COUNT);
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "sorted sequence membership must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn legacy_requested_assets(assets: &[AssetId]) -> BTreeSet<AssetId> {
    assets.iter().copied().collect()
}

fn paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = ordered.len().saturating_mul(percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
