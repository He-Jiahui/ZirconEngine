use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crate::core::editor_operation::EditorOperationPath;

use super::{EditorCommandPaletteMru, EditorCommandPaletteMruIndices};

const BENCH_CATALOG_SIZE: usize = 100_000;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn editor08_palette_mru_catalog_indices_ignore_missing_commands_and_preserve_membership() {
    let entry_indices = BTreeMap::from([
        ("palette.alpha".to_string(), 7),
        ("palette.beta".to_string(), 2),
        ("palette.gamma".to_string(), 11),
    ]);
    let mru = EditorCommandPaletteMru::new([
        operation("palette.missing"),
        operation("palette.gamma"),
        operation("palette.alpha"),
    ])
    .unwrap();

    let indices = EditorCommandPaletteMruIndices::new(&mru, &entry_indices);

    assert!(indices.contains(7));
    assert!(indices.contains(11));
    assert!(!indices.contains(2));
    assert!(!indices.contains(usize::MAX));
}

#[test]
#[ignore = "release-only palette MRU membership benchmark"]
fn editor08_palette_mru_catalog_index_release_benchmark_evidence() {
    let entry_ids = (0..BENCH_CATALOG_SIZE)
        .map(|index| format!("palette.command_{index:06}"))
        .collect::<Vec<_>>();
    let entry_indices = entry_ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect::<BTreeMap<_, _>>();
    let mru = EditorCommandPaletteMru::new(
        (BENCH_CATALOG_SIZE - 32..BENCH_CATALOG_SIZE)
            .rev()
            .map(|index| operation(&entry_ids[index])),
    )
    .unwrap();
    assert_eq!(
        legacy_checksum(&entry_ids, &mru),
        indexed_checksum(&entry_ids, &entry_indices, &mru)
    );

    let (legacy_samples, indexed_samples) = paired_samples(
        || measure_legacy(&entry_ids, &mru),
        || measure_indexed(&entry_ids, &entry_indices, &mru),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let indexed_p50_ns = percentile(&indexed_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let indexed_p95_ns = percentile(&indexed_samples, 95);

    println!(
        "PERF_RESULT plan=Editor08 task=palette_mru_catalog_index \
sample_pairs={SAMPLE_PAIRS} catalog_size={BENCH_CATALOG_SIZE} mru_capacity=32 \
legacy_membership=bounded_linear_string_scan optimized_membership=stack_index_binary_search \
pair_order=alternating_legacy_even legacy_first_pairs=11 indexed_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} indexed_p50_ns={indexed_p50_ns} \
legacy_p95_ns={legacy_p95_ns} indexed_p95_ns={indexed_p95_ns} \
legacy_raw_ns={} indexed_raw_ns={}",
        raw(&legacy_samples),
        raw(&indexed_samples),
    );

    assert!(
        indexed_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "indexed MRU membership must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns indexed={indexed_p95_ns}ns"
    );
}

fn operation(value: &str) -> EditorOperationPath {
    EditorOperationPath::parse(value).expect("valid benchmark operation path")
}

fn legacy_checksum(entry_ids: &[String], mru: &EditorCommandPaletteMru) -> usize {
    entry_ids
        .iter()
        .enumerate()
        .filter(|(_, id)| mru.contains_id(id))
        .map(|(index, _)| index)
        .sum()
}

fn indexed_checksum(
    entry_ids: &[String],
    entry_indices: &BTreeMap<String, usize>,
    mru: &EditorCommandPaletteMru,
) -> usize {
    let indices = EditorCommandPaletteMruIndices::new(mru, entry_indices);
    entry_ids
        .iter()
        .enumerate()
        .filter(|(index, _)| indices.contains(*index))
        .map(|(index, _)| index)
        .sum()
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_indexed: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_indexed());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, indexed_samples)
}

fn measure_legacy(entry_ids: &[String], mru: &EditorCommandPaletteMru) -> u128 {
    let started = Instant::now();
    black_box(legacy_checksum(black_box(entry_ids), black_box(mru)));
    started.elapsed().as_nanos().max(1)
}

fn measure_indexed(
    entry_ids: &[String],
    entry_indices: &BTreeMap<String, usize>,
    mru: &EditorCommandPaletteMru,
) -> u128 {
    let started = Instant::now();
    black_box(indexed_checksum(
        black_box(entry_ids),
        black_box(entry_indices),
        black_box(mru),
    ));
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
