use std::hint::black_box;
use std::time::Instant;

use super::runtime_ui_graph_pass_order;

const SAMPLE_PAIRS: usize = 31;
const RESOLUTIONS_PER_SAMPLE: usize = 5_000;
const PASS_COUNT: usize = 1_024;

#[test]
fn optimization_batch_20260829aq_runtime317_early_exit_preserves_first_pass_order() {
    let passes = string_passes(&[
        "uber",
        "runtime-ui",
        "overlay-gizmo",
        "overlay-gizmo",
        "runtime-ui",
        "uber",
    ]);

    assert_eq!(
        runtime_ui_graph_pass_order(&passes, 1),
        legacy_runtime_ui_graph_pass_order(&passes, 1)
    );
    assert_eq!(
        runtime_ui_graph_pass_order(&passes, 1),
        Some("postprocess-ui-overlay")
    );
}

#[test]
fn optimization_batch_20260829aq_runtime317_ui_graph_order_stops_after_three_first_matches() {
    let source = include_str!("../ui_stats.rs");
    let implementation = source
        .split("fn runtime_ui_graph_pass_order")
        .nth(1)
        .expect("runtime UI graph order resolver")
        .split("#[cfg(test)]")
        .next()
        .expect("runtime UI graph order resolver body");

    assert!(implementation.contains("postprocess.is_some()"));
    assert!(implementation.contains("runtime_ui.is_some()"));
    assert!(implementation.contains("overlay.is_some()"));
    assert!(implementation.contains("break;"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aq_runtime317_early_ui_graph_order_resolution_bench() {
    let passes = passes();
    assert_eq!(
        runtime_ui_graph_pass_order(&passes, 1),
        legacy_runtime_ui_graph_pass_order(&passes, 1)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&passes, false));
            optimized_samples.push(measure(&passes, true));
        } else {
            optimized_samples.push(measure(&passes, true));
            legacy_samples.push(measure(&passes, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME317_EARLY_UI_GRAPH_ORDER_RESOLUTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} passes_per_resolution={PASS_COUNT} \
legacy_pass_visits_per_resolution=1024 optimized_pass_visits_per_resolution=3 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn passes() -> Vec<String> {
    let mut passes = Vec::with_capacity(PASS_COUNT);
    passes.extend(string_passes(&["uber", "runtime-ui", "overlay-gizmo"]));
    passes.extend((passes.len()..PASS_COUNT).map(|index| format!("tail-pass-{index:04}")));
    passes
}

fn string_passes(passes: &[&str]) -> Vec<String> {
    passes.iter().map(|pass| (*pass).to_string()).collect()
}

fn legacy_runtime_ui_graph_pass_order(
    executed_passes: &[String],
    ui_graph_executed_pass_count: usize,
) -> Option<&'static str> {
    if ui_graph_executed_pass_count == 0 {
        return None;
    }
    let mut postprocess = None;
    let mut runtime_ui = None;
    let mut overlay = None;
    for (index, pass) in executed_passes.iter().enumerate() {
        match pass.as_str() {
            "uber" if postprocess.is_none() => postprocess = Some(index),
            "runtime-ui" if runtime_ui.is_none() => runtime_ui = Some(index),
            "overlay-gizmo" if overlay.is_none() => overlay = Some(index),
            _ => {}
        }
    }
    let postprocess = postprocess?;
    let runtime_ui = runtime_ui?;
    let overlay = overlay?;
    if postprocess < overlay && overlay < runtime_ui {
        Some("postprocess-overlay-ui")
    } else if postprocess < runtime_ui && runtime_ui < overlay {
        Some("postprocess-ui-overlay")
    } else {
        None
    }
}

fn measure(passes: &[String], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..RESOLUTIONS_PER_SAMPLE {
        let order = if optimized {
            runtime_ui_graph_pass_order(black_box(passes), 1)
        } else {
            legacy_runtime_ui_graph_pass_order(black_box(passes), 1)
        }
        .expect("benchmark UI graph order");
        checksum = checksum.wrapping_add(order.len());
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
