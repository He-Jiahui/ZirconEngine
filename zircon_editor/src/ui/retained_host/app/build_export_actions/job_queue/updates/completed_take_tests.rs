use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::take_completed;

const MARKER: &str = "EDITOR183_COMPLETED_EXPORT_SUMMARY_DIRECT_TAKE_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const BATCH_COUNT: usize = 128;
const SUMMARIES_PER_BATCH: usize = 1_024;

#[test]
fn optimization_batch_20260826gq_editor183_take_completed_preserves_wrapped_order() {
    let mut completed = (0_usize..6).collect::<VecDeque<_>>();
    assert_eq!(completed.pop_front(), Some(0));
    assert_eq!(completed.pop_front(), Some(1));
    completed.extend([6, 7]);

    let summaries = take_completed(&mut completed);

    assert_eq!(summaries, vec![2, 3, 4, 5, 6, 7]);
    assert!(completed.is_empty());
}

#[test]
fn optimization_batch_20260826gq_editor183_poll_takes_completed_storage_directly() {
    let source = include_str!("../updates.rs");
    assert!(source.contains("take_completed(&mut self.completed)"));
    assert!(source.contains("std::mem::take(completed).into()"));
    assert!(!source.contains("self.completed.drain(..).collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gq_editor183_completed_export_summary_direct_take_bench() {
    let template = (0..BATCH_COUNT)
        .map(|batch| {
            (0..SUMMARIES_PER_BATCH)
                .map(|index| batch * SUMMARIES_PER_BATCH + index)
                .collect::<VecDeque<_>>()
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let mut legacy_batches = template.clone();
        let mut optimized_batches = template.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure(&mut legacy_batches, legacy_take_completed));
            optimized_samples.push(measure(&mut optimized_batches, take_completed));
        } else {
            optimized_samples.push(measure(&mut optimized_batches, take_completed));
            legacy_samples.push(measure(&mut legacy_batches, legacy_take_completed));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct take must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_take_completed<T>(completed: &mut VecDeque<T>) -> Vec<T> {
    completed.drain(..).collect()
}

fn measure(
    batches: &mut [VecDeque<usize>],
    implementation: fn(&mut VecDeque<usize>) -> Vec<usize>,
) -> u64 {
    let started = Instant::now();
    let mut count = 0;
    for completed in batches {
        count += implementation(completed).len();
    }
    black_box(count);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
