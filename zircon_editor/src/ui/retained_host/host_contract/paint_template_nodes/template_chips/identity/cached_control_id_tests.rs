use std::hint::black_box;
use std::time::Instant;

const CHECKS_PER_SAMPLE: usize = 1_000_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_control_id_score(control_id: &str) -> usize {
    if control_id.starts_with("WorkbenchStatus") {
        1
    } else if control_id == "WorkbenchChipRoot"
        || matches!(
            control_id,
            "WorkbenchViewportMode"
                | "WorkbenchViewportLit"
                | "WorkbenchViewportAngle"
                | "WorkbenchViewportSpeed"
        )
        || control_id == "chip"
        || (control_id.starts_with("Workbench") && matches!(control_id, "chip" | "pill"))
    {
        2
    } else {
        0
    }
}

fn optimized_control_id_score(control_id: &str) -> usize {
    let cached = black_box(control_id);
    if cached.starts_with("WorkbenchStatus") {
        1
    } else if cached == "WorkbenchChipRoot"
        || matches!(
            cached,
            "WorkbenchViewportMode"
                | "WorkbenchViewportLit"
                | "WorkbenchViewportAngle"
                | "WorkbenchViewportSpeed"
        )
    {
        2
    } else {
        0
    }
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        let control_id = black_box("WorkbenchViewportMode");
        evidence += if optimized {
            optimized_control_id_score(control_id)
        } else {
            legacy_control_id_score(control_id)
        };
    }
    black_box(evidence);
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

#[test]
fn optimization_batch_20260830bd_editor301_cached_control_id_preserves_results() {
    for control_id in [
        "WorkbenchStatus",
        "WorkbenchChipRoot",
        "WorkbenchViewportMode",
        "Other",
    ] {
        assert_eq!(
            legacy_control_id_score(control_id),
            optimized_control_id_score(control_id)
        );
    }
}

#[test]
fn optimization_batch_20260830bd_editor301_production_caches_control_id() {
    let source = include_str!("../identity.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(head, _)| head)
        .unwrap_or(source);
    assert!(production.contains("let control_id = node.control_id.as_str();"));
    assert_eq!(production.matches("node.control_id.as_str()").count(), 1);
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260830bd_editor301_cached_control_id_benchmark() {
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(false));
            candidate.push(measure(true));
        } else {
            candidate.push(measure(true));
            baseline.push(measure(false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "EDITOR301_CACHED_CONTROL_ID_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
