use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;

use super::target_mode_summary;
use crate::ui::retained_host::app::module_plugin_projection::rows::target_mode_label;

const SAMPLE_PAIRS: usize = 21;
const SUMMARIES_PER_SAMPLE: usize = 8_192;
const MODES_PER_SUMMARY: usize = 32;

#[test]
fn optimization_batch_20260826di_editor98_target_mode_summary_preserves_order_and_empty() {
    use RuntimeTargetMode::{ClientRuntime, EditorHost, ServerRuntime};

    assert_eq!(target_mode_summary(&[]), "");
    assert_eq!(
        target_mode_summary(&[EditorHost, ClientRuntime, ServerRuntime, EditorHost]),
        "editor, client, server, editor"
    );
}

#[test]
fn optimization_batch_20260826di_editor98_target_mode_summary_avoids_temporary_vec() {
    let modes = fixture_modes();
    let summary = target_mode_summary(&modes);
    assert!(summary.capacity() >= summary.len());
    assert!(summary.capacity() - summary.len() <= 2);

    let source = include_str!("../view_rows.rs");
    assert!(source.contains("target_modes: target_mode_summary(&plugin.target_modes).into()"));
    assert!(source.contains("String::with_capacity("));
    assert!(source.contains("summary.push_str(target_mode_label(mode));"));
    assert!(!source.contains(".map(target_mode_label)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826di_editor98_target_mode_direct_join_bench() {
    let modes = fixture_modes();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&modes, legacy_target_mode_summary));
            optimized_samples.push(measure(&modes, target_mode_summary));
        } else {
            optimized_samples.push(measure(&modes, target_mode_summary));
            legacy_samples.push(measure(&modes, legacy_target_mode_summary));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR98_MODULE_ROW_TARGET_MODE_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
summaries_per_sample={SUMMARIES_PER_SAMPLE} modes_per_summary={MODES_PER_SUMMARY} \
legacy_temporary_vec_allocations_per_sample={SUMMARIES_PER_SAMPLE} \
optimized_temporary_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct target-mode summary P95 {optimized_p95_ns}ns must be at most 70% of temporary-vector joining P95 {legacy_p95_ns}ns"
    );
}

fn fixture_modes() -> Vec<RuntimeTargetMode> {
    use RuntimeTargetMode::{ClientRuntime, EditorHost, ServerRuntime};

    [ClientRuntime, ServerRuntime, EditorHost]
        .into_iter()
        .cycle()
        .take(MODES_PER_SUMMARY)
        .collect()
}

fn legacy_target_mode_summary(target_modes: &[RuntimeTargetMode]) -> String {
    target_modes
        .iter()
        .map(target_mode_label)
        .collect::<Vec<_>>()
        .join(", ")
}

fn measure(modes: &[RuntimeTargetMode], summarize: fn(&[RuntimeTargetMode]) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SUMMARIES_PER_SAMPLE {
        checksum ^= black_box(summarize(black_box(modes))).len();
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
