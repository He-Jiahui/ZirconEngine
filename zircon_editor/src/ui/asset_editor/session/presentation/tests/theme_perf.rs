use std::hint::black_box;
use std::time::Instant;

#[test]
fn optimization_batch_ee_theme_kind_is_checked_before_direct_move() {
    let source = include_str!("../theme.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("theme presentation production implementation");
    let check = production
        .find("let can_edit_promote_draft =")
        .expect("theme editability check");
    let direct_move = production
        .find("selected_source_kind: summary.selected_kind,")
        .expect("selected theme kind direct move");

    assert!(check < direct_move);
    assert!(!production.contains("selected_kind.clone()"));
}

#[test]
#[ignore = "release-only direct theme-kind move benchmark"]
fn optimization_batch_ee_direct_theme_kind_move_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const PROJECTIONS_PER_SAMPLE: usize = 16_384;

    fn measure_legacy() -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..PROJECTIONS_PER_SAMPLE {
            let selected_kind = black_box(String::from("Local"));
            let projected_kind = black_box(selected_kind.clone());
            let can_edit = black_box(selected_kind == "Local");
            checksum = checksum.wrapping_add(projected_kind.len() + usize::from(can_edit));
            black_box(projected_kind);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized() -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..PROJECTIONS_PER_SAMPLE {
            let selected_kind = black_box(String::from("Local"));
            let can_edit = black_box(selected_kind == "Local");
            let projected_kind = black_box(selected_kind);
            checksum = checksum.wrapping_add(projected_kind.len() + usize::from(can_edit));
            black_box(projected_kind);
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

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR367_DIRECT_THEME_KIND_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             projections_per_sample={PROJECTIONS_PER_SAMPLE} pair_order=alternating_legacy_even \
             legacy_kind_clones_per_sample={PROJECTIONS_PER_SAMPLE} \
             optimized_kind_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
        "moving the theme kind must reduce P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
