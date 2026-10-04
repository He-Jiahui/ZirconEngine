use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn updates_preserve_old_snapshot_and_delete_vacated_cells() {
    let original =
        PersistentCellBuckets::from_cells(HashMap::from([((0, 0), vec![1, 2]), ((1, 0), vec![3])]));
    let updated = original.with_updates(BTreeMap::from([
        ((0, 0), Some(vec![4])),
        ((1, 0), None),
        ((2, 0), Some(vec![5])),
    ]));

    assert_eq!(original.get(&(0, 0)), Some(&vec![1, 2]));
    assert_eq!(original.get(&(1, 0)), Some(&vec![3]));
    assert_eq!(updated.get(&(0, 0)), Some(&vec![4]));
    assert_eq!(updated.get(&(1, 0)), None);
    assert_eq!(updated.get(&(2, 0)), Some(&vec![5]));
}

#[test]
fn repeated_path_copy_updates_keep_lookup_depth_balanced() {
    let mut buckets = PersistentCellBuckets::default();
    for x in 0..512 {
        buckets = buckets.with_updates(BTreeMap::from([((x, 0), Some(vec![x as usize]))]));
    }

    assert!(buckets.height_for_test() <= 11);
    assert_eq!(buckets.get(&(511, 0)), Some(&vec![511]));
}

#[test]
fn optimization_batch_gy_editor580_from_cells_sorts_one_dense_buffer() {
    let buckets = PersistentCellBuckets::from_cells(HashMap::from([
        ((3, 1), vec![31]),
        ((-2, 4), Vec::new()),
        ((0, 0), vec![1, 2]),
        ((-1, -1), vec![7]),
    ]));

    assert_eq!(buckets.len, 3);
    assert_eq!(buckets.get(&(3, 1)), Some(&vec![31]));
    assert_eq!(buckets.get(&(0, 0)), Some(&vec![1, 2]));
    assert_eq!(buckets.get(&(-1, -1)), Some(&vec![7]));
    assert_eq!(buckets.get(&(-2, 4)), None);

    let source = include_str!("../persistent_buckets.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("persistent bucket production source");
    assert!(production.contains("sorted.sort_unstable_by_key"));
    assert!(!production.contains("collect::<BTreeMap<_, _>>()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_gy_editor580_dense_sort_performance_evidence() {
    fn legacy_from_cells(cells: HashMap<Cell, Vec<usize>>) -> PersistentCellBuckets {
        let sorted = cells
            .into_iter()
            .filter(|(_, rows)| !rows.is_empty())
            .map(|(cell, rows)| (cell_key(cell), Arc::new(rows)))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect::<Vec<_>>();
        PersistentCellBuckets {
            root: build_balanced(&sorted),
            len: sorted.len(),
        }
    }

    const CELL_COUNT: usize = 8_192;
    let cells = (0..CELL_COUNT)
        .map(|index| {
            let x = (index as i32).wrapping_mul(17);
            let y = (index as i32).wrapping_mul(-31);
            ((x, y), vec![index, index + 1])
        })
        .collect::<HashMap<_, _>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let legacy_input = cells.clone();
        let started = Instant::now();
        black_box(legacy_from_cells(black_box(legacy_input)));
        legacy_samples.push(started.elapsed().as_nanos());

        let optimized_input = cells.clone();
        let started = Instant::now();
        black_box(PersistentCellBuckets::from_cells(black_box(
            optimized_input,
        )));
        optimized_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "EDITOR580_PERSISTENT_BUCKET_DENSE_SORT_BENCH_V1 cells={CELL_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=7000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "persistent bucket dense sort P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}
