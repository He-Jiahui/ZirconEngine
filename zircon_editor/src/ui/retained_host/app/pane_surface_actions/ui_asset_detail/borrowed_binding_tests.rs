use std::hint::black_box;
use std::time::Instant;

use super::UiAssetDetailSurfaceBinding;

const SAMPLE_PAIRS: usize = 31;
const PARSES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260828iq_editor235_borrowed_binding_preserves_fields() {
    let binding_id = "ui_asset_detail|view.asset.7|style.rule|set_value|42";

    let binding = UiAssetDetailSurfaceBinding::parse(binding_id).expect("valid binding");

    assert_eq!(binding.instance_id, "view.asset.7");
    assert_eq!(binding.detail_id, "style.rule");
    assert_eq!(binding.action_id, "set_value");
    assert_eq!(binding.item_index, 42);
    assert!(UiAssetDetailSurfaceBinding::parse(
        "ui_asset_detail|view.asset.7|style.rule|set_value|42|extra"
    )
    .is_none());
}

#[test]
fn optimization_batch_20260828iq_editor235_binding_parser_borrows_segments() {
    let source = include_str!("../ui_asset_detail.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("binding implementation");

    assert!(implementation.contains("pub(super) instance_id: &'a str"));
    assert!(implementation.contains("pub(super) detail_id: &'a str"));
    assert!(implementation.contains("pub(super) action_id: &'a str"));
    assert!(!implementation.contains("parts.next()?.to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828iq_editor235_borrowed_ui_asset_detail_binding_bench() {
    let binding_id = format!(
        "ui_asset_detail|{}|{}|{}|17",
        "view.asset.instance.".repeat(8),
        "style.rule.detail.".repeat(8),
        "set_property_value.".repeat(8),
    );
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(binding_id.as_str(), false));
            optimized_samples.push(measure(binding_id.as_str(), true));
        } else {
            optimized_samples.push(measure(binding_id.as_str(), true));
            legacy_samples.push(measure(binding_id.as_str(), false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR235_BORROWED_UI_ASSET_DETAIL_BINDING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} binding_bytes={} \
legacy_field_allocations_per_parse=3 optimized_field_allocations_per_parse=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        binding_id.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

struct LegacyBinding {
    instance_id: String,
    detail_id: String,
    action_id: String,
    item_index: i32,
}

fn legacy_parse(binding_id: &str) -> Option<LegacyBinding> {
    let mut parts = binding_id.split('|');
    if parts.next()? != "ui_asset_detail" {
        return None;
    }
    let instance_id = parts.next()?.to_string();
    let detail_id = parts.next()?.to_string();
    let action_id = parts.next()?.to_string();
    let item_index = parts.next()?.parse().ok()?;
    if parts.next().is_some()
        || instance_id.is_empty()
        || detail_id.is_empty()
        || action_id.is_empty()
    {
        return None;
    }
    Some(LegacyBinding {
        instance_id,
        detail_id,
        action_id,
        item_index,
    })
}

fn measure(binding_id: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..PARSES_PER_SAMPLE {
        if optimized {
            let binding = black_box(UiAssetDetailSurfaceBinding::parse(black_box(binding_id)))
                .expect("valid optimized binding");
            checksum ^= black_box(
                binding.instance_id.len()
                    ^ binding.detail_id.len()
                    ^ binding.action_id.len()
                    ^ binding.item_index as usize,
            );
        } else {
            let binding =
                black_box(legacy_parse(black_box(binding_id))).expect("valid legacy binding");
            checksum ^= black_box(
                binding.instance_id.len()
                    ^ binding.detail_id.len()
                    ^ binding.action_id.len()
                    ^ binding.item_index as usize,
            );
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
