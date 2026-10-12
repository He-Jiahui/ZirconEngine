use std::hint::black_box;
use std::time::Instant;

const STYLESHEET_COUNT: usize = 16;
const RULES_PER_STYLESHEET: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jb_editor641_reserves_local_style_rule_entries() {
    let source = include_str!("../../style_inspection.rs");
    let projection = source
        .split("pub(super) fn local_style_rule_entries")
        .nth(1)
        .expect("local style rule projection remains present")
        .split("pub(super) fn selected_style_rule_declaration_entries")
        .next()
        .expect("local style rule projection remains bounded");

    assert!(projection.contains("let rule_count = document"));
    assert!(projection.contains(".map(|stylesheet| stylesheet.rules.len())"));
    assert!(projection.contains("let mut entries = Vec::with_capacity(rule_count);"));
    assert!(!projection.contains("let mut entries = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jb_editor641_preallocated_local_style_rule_entry_benchmark() {
    let stylesheet_rule_counts = [RULES_PER_STYLESHEET; STYLESHEET_COUNT];
    for _ in 0..4 {
        black_box(measure_entries(&stylesheet_rule_counts, false));
        black_box(measure_entries(&stylesheet_rule_counts, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_entries(&stylesheet_rule_counts, false));
            preallocated_samples.push(measure_entries(&stylesheet_rule_counts, true));
        } else {
            preallocated_samples.push(measure_entries(&stylesheet_rule_counts, true));
            unreserved_samples.push(measure_entries(&stylesheet_rule_counts, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR641_PREALLOCATED_LOCAL_STYLE_RULE_ENTRY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} stylesheet_count={STYLESHEET_COUNT} rules_per_stylesheet={RULES_PER_STYLESHEET} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_entries(stylesheet_rule_counts: &[usize], preallocated: bool) -> u128 {
    let started = Instant::now();
    let mut entries = if preallocated {
        Vec::with_capacity(stylesheet_rule_counts.iter().sum())
    } else {
        Vec::new()
    };
    for (stylesheet_index, rule_count) in stylesheet_rule_counts.iter().copied().enumerate() {
        for rule_index in 0..rule_count {
            entries.push(black_box((stylesheet_index, rule_index)));
        }
    }
    black_box(entries);
    started.elapsed().as_nanos().max(1)
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
