use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LABELS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fa_editor389_preserves_bridge_status_labels() {
    for status in [
        BridgeInterfaceStatus::Absent,
        BridgeInterfaceStatus::Enabled,
        BridgeInterfaceStatus::Disabled,
    ] {
        assert_eq!(bridge_status_label(status), format!("{status:?}"));
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fa_editor389_direct_bridge_status_label_benchmark() {
    for _ in 0..4 {
        black_box(measure(|status| format!("{status:?}")));
        black_box(measure(bridge_status_label));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(|status| format!("{status:?}")));
            optimized_samples.push(measure(bridge_status_label));
        } else {
            optimized_samples.push(measure(bridge_status_label));
            legacy_samples.push(measure(|status| format!("{status:?}")));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure(mut label: impl FnMut(BridgeInterfaceStatus) -> String) -> u128 {
    let statuses = [
        BridgeInterfaceStatus::Absent,
        BridgeInterfaceStatus::Enabled,
        BridgeInterfaceStatus::Disabled,
    ];
    let started = Instant::now();
    let mut checksum = 0_usize;
    for index in 0..LABELS_PER_SAMPLE {
        let value = label(black_box(statuses[index % statuses.len()]));
        checksum = checksum.wrapping_add(black_box(value.len()));
        black_box(value);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR389_DIRECT_BRIDGE_STATUS_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(65) / 100,
        "direct bridge status labels must reduce P95 by at least 35%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
