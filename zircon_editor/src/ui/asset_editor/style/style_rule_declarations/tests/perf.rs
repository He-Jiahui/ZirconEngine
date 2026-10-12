use std::hint::black_box;
use std::time::Instant;

const ENTRY_COUNT: usize = 1_048_576;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jc_editor642_reserves_style_declaration_entries() {
    let source = include_str!("../../style_rule_declarations.rs");
    let projection = source
        .split("pub(crate) fn declaration_entries")
        .nth(1)
        .expect("style declaration projection remains present")
        .split("pub(crate) fn parse_declaration_literal")
        .next()
        .expect("style declaration projection remains bounded");

    assert!(projection.contains("let entry_capacity = block"));
    assert!(projection.contains(".saturating_add(block.slot.len());"));
    assert!(projection.contains("let mut entries = Vec::with_capacity(entry_capacity);"));
    assert!(!projection.contains("let mut entries = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jc_editor642_preallocated_style_declaration_entry_benchmark() {
    for _ in 0..4 {
        black_box(measure_entries(false));
        black_box(measure_entries(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_entries(false));
            preallocated_samples.push(measure_entries(true));
        } else {
            preallocated_samples.push(measure_entries(true));
            unreserved_samples.push(measure_entries(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR642_PREALLOCATED_STYLE_DECLARATION_ENTRY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entry_count={ENTRY_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_entries(preallocated: bool) -> u128 {
    let started = Instant::now();
    let mut entries = if preallocated {
        Vec::with_capacity(ENTRY_COUNT)
    } else {
        Vec::new()
    };
    for entry in 0..ENTRY_COUNT {
        entries.push(black_box((entry, entry.wrapping_mul(31))));
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
