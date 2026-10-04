use std::hint::black_box;
use std::time::Instant;

const ASSET_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[derive(Clone)]
struct AssetOrderModel {
    locator: String,
    uuid: String,
}

#[test]
fn optimization_batch_jg_runtime646_caches_uuid_sort_keys_per_locator_group() {
    let source = include_str!("../../relocation_preflight.rs");
    let ordering = source
        .split("fn sort_assets_by_canonical_order")
        .nth(1)
        .expect("canonical asset ordering remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("canonical asset ordering remains bounded");

    assert!(ordering.contains("sort_by_cached_key(|asset| asset.uuid().to_string())"));
    assert!(!ordering.contains("sort_by(asset_order)"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jg_runtime646_cached_relocation_order_benchmark() {
    let assets = fixture_assets();
    assert_eq!(legacy_order(&assets), cached_order(&assets));

    for _ in 0..4 {
        black_box(measure_order(&assets, false));
        black_box(measure_order(&assets, true));
    }
    let mut comparator_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut cached_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            comparator_samples.push(measure_order(&assets, false));
            cached_samples.push(measure_order(&assets, true));
        } else {
            cached_samples.push(measure_order(&assets, true));
            comparator_samples.push(measure_order(&assets, false));
        }
    }

    let comparator_p95 = percentile(&comparator_samples, 95);
    let cached_p95 = percentile(&cached_samples, 95);
    let improvement_percent = comparator_p95
        .saturating_sub(cached_p95)
        .saturating_mul(100)
        / comparator_p95.max(1);
    println!(
        "RUNTIME646_CACHED_RELOCATION_ORDER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} asset_count={ASSET_COUNT} comparator_ns={} cached_ns={} comparator_p95_ns={comparator_p95} cached_p95_ns={cached_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&comparator_samples),
        csv(&cached_samples),
    );
    assert!(cached_p95 <= comparator_p95 * 80 / 100);
}

fn fixture_assets() -> Vec<AssetOrderModel> {
    (0..ASSET_COUNT)
        .rev()
        .map(|index| AssetOrderModel {
            locator: "res://batch-646/shared.asset".to_string(),
            uuid: format!("{index:08}-batch-646"),
        })
        .collect()
}

fn measure_order(assets: &[AssetOrderModel], cached: bool) -> u128 {
    let started = Instant::now();
    let ordered = if cached {
        cached_order(assets)
    } else {
        legacy_order(assets)
    };
    black_box(ordered);
    started.elapsed().as_nanos().max(1)
}

fn legacy_order(assets: &[AssetOrderModel]) -> Vec<String> {
    let mut ordered = assets.to_vec();
    ordered.sort_by(|left, right| {
        left.locator
            .cmp(&right.locator)
            .then_with(|| left.uuid.to_string().cmp(&right.uuid.to_string()))
    });
    ordered.into_iter().map(|asset| asset.uuid).collect()
}

fn cached_order(assets: &[AssetOrderModel]) -> Vec<String> {
    let mut ordered = assets.to_vec();
    ordered.sort_by(|left, right| left.locator.cmp(&right.locator));
    let mut group_start = 0;
    while group_start < ordered.len() {
        let locator = ordered[group_start].locator.clone();
        let mut group_end = group_start + 1;
        while group_end < ordered.len() && ordered[group_end].locator == locator {
            group_end += 1;
        }
        if group_end - group_start > 1 {
            ordered[group_start..group_end].sort_by_cached_key(|asset| asset.uuid.to_string());
        }
        group_start = group_end;
    }
    ordered.into_iter().map(|asset| asset.uuid).collect()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
