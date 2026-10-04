use std::hint::black_box;
use std::time::Instant;

const IMPORT_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jh_editor647_reserves_theme_source_entries() {
    let source = include_str!("../../theme_summary.rs");
    let entries = source
        .split("fn theme_source_entries")
        .nth(1)
        .expect("theme source entries remain present")
        .split("fn selected_theme_document")
        .next()
        .expect("theme source entries remain bounded");

    assert!(entries.contains("Vec::with_capacity(document.imports.styles.len().saturating_add(1))"));
    assert!(!entries.contains("let mut entries = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jh_editor647_theme_source_entry_capacity_benchmark() {
    for _ in 0..4 {
        black_box(measure_entries(false));
        black_box(measure_entries(true));
    }
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_entries(false));
            reserved_samples.push(measure_entries(true));
        } else {
            reserved_samples.push(measure_entries(true));
            unreserved_samples.push(measure_entries(false));
        }
    }
    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let reserved_p95 = percentile(&reserved_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(reserved_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR647_THEME_SOURCE_ENTRY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} import_count={IMPORT_COUNT} unreserved_ns={} reserved_ns={} unreserved_p95_ns={unreserved_p95} reserved_p95_ns={reserved_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&reserved_samples),
    );
    assert!(reserved_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_entries(reserved: bool) -> u128 {
    let started = Instant::now();
    let mut entries = if reserved {
        Vec::with_capacity(IMPORT_COUNT + 1)
    } else {
        Vec::new()
    };
    for index in 0..IMPORT_COUNT {
        entries.push(black_box(index));
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
