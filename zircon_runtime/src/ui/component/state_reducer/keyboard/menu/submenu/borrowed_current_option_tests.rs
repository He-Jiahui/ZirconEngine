use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hy_runtime_borrowed_option_scan_preserves_first_match() {
    let options = benchmark_options(8, 32);

    assert_eq!(current_option_id_index(&options, &options[3].id), Some(3));
    assert_eq!(current_option_id_index(&options, "missing-option"), None);
}

#[test]
fn optimization_batch_20260828hy_runtime_submenu_current_index_has_no_id_clone_vector() {
    let source = include_str!("../submenu.rs");
    let open_submenu = source
        .split("pub(in crate::ui::component::state_reducer::keyboard) fn open_focused_submenu")
        .nth(1)
        .and_then(|body| {
            body.split(
                "pub(in crate::ui::component::state_reducer::keyboard) fn close_active_submenu",
            )
            .next()
        })
        .expect("open focused submenu implementation");

    assert!(open_submenu.contains("current_submenu_option_index(state, descriptor, &options)"));
    assert!(!open_submenu.contains("let option_ids ="));
    assert!(!open_submenu.contains("option.id.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hy_runtime_borrowed_submenu_current_option_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 64;
    let options = benchmark_options(4_096, 256);
    let target = options.last().unwrap().id.as_str();

    black_box(legacy_current_option_id_index(&options, target));
    black_box(current_option_id_index(&options, target));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_current_option_id_index(
                    black_box(&options),
                    black_box(target),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(current_option_id_index(
                    black_box(&options),
                    black_box(target),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME271_BORROWED_SUBMENU_CURRENT_OPTION_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_options(count: usize, payload_bytes: usize) -> Vec<super::super::super::OptionEntry> {
    let payload = "x".repeat(payload_bytes);
    (0..count)
        .map(|index| super::super::super::OptionEntry {
            id: format!("option-{index:05}-{payload}"),
            text: format!("Option {index:05} {payload}"),
        })
        .collect()
}

fn legacy_current_option_id_index(
    options: &[super::super::super::OptionEntry],
    value: &str,
) -> Option<i64> {
    options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>()
        .iter()
        .position(|option| option == value)
        .map(|index| index as i64)
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
