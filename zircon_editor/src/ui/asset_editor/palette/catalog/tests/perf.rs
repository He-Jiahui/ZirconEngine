use std::hint::black_box;
use std::time::Instant;

const NATIVE_ENTRY_COUNT: usize = 64;
const COMPONENT_COUNT: usize = 4_096;
const REFERENCE_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jg_editor646_reserves_palette_append_capacity() {
    let source = include_str!("../../catalog.rs");
    let build = source
        .split("pub(crate) fn build")
        .nth(1)
        .expect("palette catalog build remains present")
        .split("fn canonical_reference_imports")
        .next()
        .expect("palette catalog build remains bounded");

    assert!(build.contains("let append_capacity = document"));
    assert!(build.contains("reference_imports.len()"));
    assert!(build.contains("entries.reserve_exact(append_capacity)"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jg_editor646_palette_append_benchmark() {
    for _ in 0..4 {
        black_box(measure_append(false));
        black_box(measure_append(true));
    }
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_append(false));
            reserved_samples.push(measure_append(true));
        } else {
            reserved_samples.push(measure_append(true));
            unreserved_samples.push(measure_append(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let reserved_p95 = percentile(&reserved_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(reserved_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR646_PALETTE_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} native_entries={NATIVE_ENTRY_COUNT} components={COMPONENT_COUNT} references={REFERENCE_COUNT} unreserved_ns={} reserved_ns={} unreserved_p95_ns={unreserved_p95} reserved_p95_ns={reserved_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&reserved_samples),
    );
    assert!(reserved_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_append(reserved: bool) -> u128 {
    let started = Instant::now();
    let mut entries = if reserved {
        Vec::with_capacity(NATIVE_ENTRY_COUNT)
    } else {
        Vec::new()
    };
    entries.extend((0..NATIVE_ENTRY_COUNT).map(black_box));
    if reserved {
        entries.reserve_exact(COMPONENT_COUNT + REFERENCE_COUNT);
    }
    for index in 0..COMPONENT_COUNT + REFERENCE_COUNT {
        entries.push(black_box(NATIVE_ENTRY_COUNT + index));
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
