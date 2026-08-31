use std::hint::black_box;
use std::time::Instant;

use super::{first_completion_indices, SaveDirtyViewCompletionSlot};
use crate::core::editor_message::DocumentId;

const SAMPLE_PAIRS: usize = 31;
const LOOKUPS_PER_SAMPLE: usize = 5_000;
const COMPLETION_COUNT: usize = 1_024;

#[test]
fn optimization_batch_20260829ao_editor260_completion_index_preserves_first_duplicate() {
    let completions = vec![slot(7), slot(11), slot(7)];
    let indices = first_completion_indices(&completions);

    assert_eq!(indices.get(&DocumentId::new(7)), Some(&0));
    assert_eq!(indices.get(&DocumentId::new(11)), Some(&1));
}

#[test]
fn optimization_batch_20260829ao_editor260_completion_lookup_uses_batch_index() {
    let source = include_str!("../save_job_adapter.rs");
    let batch = source
        .split("impl SaveDirtyViewsCompletionBatch")
        .nth(1)
        .expect("save completion batch")
        .split("pub fn into_completions")
        .next()
        .expect("save completion lookup body");

    assert!(source.contains("completion_indices: HashMap<DocumentId, usize>"));
    assert!(batch.contains("self.completion_indices.get(&document)"));
    assert!(!batch.contains(".iter().find("));
    assert!(source.contains("or_insert(index)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ao_editor260_indexed_save_completion_lookups_bench() {
    let completions = completions();
    let expected = legacy_completion_index(&completions, DocumentId::new(1_023));
    assert_eq!(expected, Some(1_023));
    assert_eq!(
        first_completion_indices(&completions)
            .get(&DocumentId::new(1_023))
            .copied(),
        expected
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&completions, false));
            optimized_samples.push(measure(&completions, true));
        } else {
            optimized_samples.push(measure(&completions, true));
            legacy_samples.push(measure(&completions, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR260_INDEXED_SAVE_COMPLETION_LOOKUPS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} completions={COMPLETION_COUNT} \
optimized_index_builds_per_sample=1 legacy_worst_case_comparisons_per_lookup=1024 \
optimized_hash_lookups_per_query=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn slot(document: u64) -> SaveDirtyViewCompletionSlot {
    SaveDirtyViewCompletionSlot {
        document: DocumentId::new(document),
        completion: None,
    }
}

fn completions() -> Vec<SaveDirtyViewCompletionSlot> {
    (0..COMPLETION_COUNT)
        .map(|document| slot(document as u64))
        .collect()
}

fn legacy_completion_index(
    completions: &[SaveDirtyViewCompletionSlot],
    document: DocumentId,
) -> Option<usize> {
    completions
        .iter()
        .position(|slot| slot.document == document)
}

fn measure(completions: &[SaveDirtyViewCompletionSlot], optimized: bool) -> u128 {
    let document = DocumentId::new((COMPLETION_COUNT - 1) as u64);
    let started = Instant::now();
    let index = optimized.then(|| first_completion_indices(completions));
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let found = if let Some(index) = &index {
            index.get(black_box(&document)).copied()
        } else {
            legacy_completion_index(black_box(completions), black_box(document))
        }
        .expect("benchmark completion");
        checksum = checksum.wrapping_add(found);
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
