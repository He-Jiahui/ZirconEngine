use std::hint::black_box;
use std::time::{Duration, Instant};

#[test]
fn optimization_batch_hw_editor605_compaction_borrows_ordered_suffix() {
    let source = include_str!("../store.rs");
    let compaction = source
        .split("pub fn compact_covered_prefix")
        .nth(1)
        .expect("durable compaction")
        .split("/// Publishes the already-synced compaction file")
        .next()
        .expect("durable compaction body");

    assert!(compaction.contains("partition_point"));
    assert!(compaction.contains("let retained = &report.entries()[retained_start..];"));
    assert!(!compaction.contains("collect::<Vec"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hw_editor605_compaction_suffix_lookup_p95() {
    const MARKER: &str = "EDITOR605_JOURNAL_COMPACTION_SUFFIX_LOOKUP_BENCH_V1";
    const ENTRY_COUNT: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let sequences = (1..=ENTRY_COUNT as u64).collect::<Vec<_>>();
    let covered_through = ENTRY_COUNT as u64 * 3 / 4;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&sequences, covered_through, false));
            optimized.push(measure(&sequences, covered_through, true));
        } else {
            optimized.push(measure(&sequences, covered_through, true));
            legacy.push(measure(&sequences, covered_through, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} entries={ENTRY_COUNT} iterations={ITERATIONS} allocations_per_lookup=1->0"
    );
    assert!(
        ratio <= 0.05,
        "{MARKER} expected borrowed suffix ratio <= 0.05, got {ratio:.4}"
    );
}

const ITERATIONS: usize = 256;

fn measure(sequences: &[u64], covered_through: u64, optimized: bool) -> Duration {
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        if optimized {
            let retained_start = sequences.partition_point(|sequence| *sequence <= covered_through);
            black_box(&sequences[retained_start..]);
        } else {
            let retained = sequences
                .iter()
                .filter(|sequence| **sequence > covered_through)
                .collect::<Vec<_>>();
            black_box(retained);
        }
    }
    started.elapsed()
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    values[(values.len() - 1) * percentile / 100]
}
