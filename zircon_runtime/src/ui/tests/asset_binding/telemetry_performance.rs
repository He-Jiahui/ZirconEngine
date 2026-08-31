use std::{collections::BTreeMap, hint::black_box, time::Instant};

use zircon_runtime_interface::ui::binding::UiBindingExecutionReceipt;

const SAMPLE_PAIRS: usize = 21;
const RECEIPTS_PER_SAMPLE: usize = 4_096;
const DYNAMIC_IDENTITIES: usize = 128;

#[test]
#[ignore = "release performance gate; run through the Runtime74 validator"]
fn bounded_binding_execution_receipt_p95_beats_dynamic_metric_cardinality() {
    let asset_id = "res://ui/editor/components/workbench/shell/workbench_component_drawer.zui";
    let binding_id = "ComponentLab/InputDropdownSelect";
    let generation = 0x74_046;
    let mut dynamic_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut bounded_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            dynamic_samples.push(dynamic_metric_sample(asset_id, binding_id, generation));
            bounded_samples.push(bounded_receipt_sample(asset_id, binding_id, generation));
        } else {
            bounded_samples.push(bounded_receipt_sample(asset_id, binding_id, generation));
            dynamic_samples.push(dynamic_metric_sample(asset_id, binding_id, generation));
        }
    }

    let dynamic_p95 = nearest_rank_p95(&dynamic_samples);
    let bounded_p95 = nearest_rank_p95(&bounded_samples);
    let improvement_percent =
        dynamic_p95.saturating_sub(bounded_p95).saturating_mul(100) / dynamic_p95.max(1);
    println!(
        "PERF-RUNTIME74-BOUNDED-BINDING-TELEMETRY sample_pairs={SAMPLE_PAIRS} receipts_per_sample={RECEIPTS_PER_SAMPLE} dynamic_identities={DYNAMIC_IDENTITIES} pair_order=alternating_dynamic_even dynamic_first_pairs=11 bounded_first_pairs=10 dynamic_metric_keys_per_sample={DYNAMIC_IDENTITIES} bounded_metric_keys_per_sample=4 dynamic_p95_ns={dynamic_p95} bounded_p95_ns={bounded_p95} improvement_percent={improvement_percent} improvement_threshold_percent=50 dynamic_samples_ns={} bounded_samples_ns={}",
        samples_csv(&dynamic_samples),
        samples_csv(&bounded_samples),
    );
    assert!(
        bounded_p95.saturating_mul(2) <= dynamic_p95,
        "bounded receipt P95 must be at least 50% faster than dynamic metric cardinality: dynamic={dynamic_p95}ns bounded={bounded_p95}ns"
    );
}

fn dynamic_metric_sample(asset_id: &str, binding_id: &str, generation: u64) -> u64 {
    let started_at = Instant::now();
    let mut counters = BTreeMap::<String, u64>::new();
    for index in 0..RECEIPTS_PER_SAMPLE {
        let identity = index % DYNAMIC_IDENTITIES;
        let key = format!(
            "ui.binding.asset={asset_id}.binding={binding_id}.{identity}.generation={generation}.execution_count"
        );
        counters
            .entry(key)
            .and_modify(|value| *value = value.saturating_add(1))
            .or_insert(1);
    }
    black_box(counters);
    elapsed_nanos(started_at)
}

fn bounded_receipt_sample(asset_id: &str, binding_id: &str, generation: u64) -> u64 {
    let started_at = Instant::now();
    for index in 0..RECEIPTS_PER_SAMPLE {
        black_box(UiBindingExecutionReceipt::executed(
            asset_id,
            binding_id,
            generation,
            index % 17 == 0,
            index as u64,
        ));
    }
    elapsed_nanos(started_at)
}

fn nearest_rank_p95(samples: &[u64]) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn elapsed_nanos(started_at: Instant) -> u64 {
    started_at.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn samples_csv(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
