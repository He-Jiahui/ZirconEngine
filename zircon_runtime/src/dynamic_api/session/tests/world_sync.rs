use std::hint::black_box;

use super::*;

#[test]
fn world_invalidation_pages_commit_only_the_delivered_prefix() {
    let mut pending = vec![InvalidationBatch {
        generation: 7,
        dirty: vec![WatchToken::new(1), WatchToken::new(2), WatchToken::new(3)],
        facts: Vec::new(),
    }];
    reverse_pending_world_invalidations(&mut pending);

    let first = build_world_invalidation_page(&pending, 3);
    assert_eq!(first[0].dirty.len(), 2);
    commit_world_invalidation_page(&mut pending, &first);
    assert_eq!(pending[0].dirty, vec![WatchToken::new(3)]);

    let second = build_world_invalidation_page(&pending, 3);
    commit_world_invalidation_page(&mut pending, &second);
    assert!(pending.is_empty());
}

#[test]
fn world_invalidation_tail_queues_preserve_batch_and_item_order() {
    let original = vec![
        InvalidationBatch {
            generation: 7,
            dirty: vec![WatchToken::new(1), WatchToken::new(2)],
            facts: Vec::new(),
        },
        InvalidationBatch {
            generation: 8,
            dirty: vec![WatchToken::new(3), WatchToken::new(4)],
            facts: Vec::new(),
        },
    ];
    let mut pending = original.clone();
    reverse_pending_world_invalidations(&mut pending);

    let page = build_world_invalidation_page(&pending, 6);
    assert_eq!(page, original);
    commit_world_invalidation_page(&mut pending, &page);
    assert!(pending.is_empty());
}

#[test]
fn borrowed_world_invalidation_pages_match_owned_wire_bytes() {
    let mut pending = vec![
        InvalidationBatch {
            generation: 7,
            dirty: vec![WatchToken::new(1), WatchToken::new(2)],
            facts: vec![
                WorldFact::AssetReloadApplied(Default::default()),
                WorldFact::AssetReloadApplied(Default::default()),
            ],
        },
        InvalidationBatch {
            generation: 8,
            dirty: vec![WatchToken::new(3), WatchToken::new(4)],
            facts: vec![WorldFact::AssetReloadApplied(Default::default())],
        },
    ];
    reverse_pending_world_invalidations(&mut pending);

    for max_items in 0..=9 {
        let owned = build_world_invalidation_page(&pending, max_items);
        let owned_bytes = encode_world_invalidation_page(&owned).unwrap();
        let borrowed_bytes =
            encode_borrowed_world_invalidation_page_at(&pending, max_items, Instant::now())
                .unwrap();
        assert_eq!(borrowed_bytes, owned_bytes, "max_items={max_items}");
    }
}

#[test]
fn world_invalidation_tail_queue_source_has_no_front_removal() {
    let source = include_str!("../world_sync.rs");
    let start = source
        .find("fn commit_world_invalidation_page(")
        .expect("world invalidation commit owner");
    let end = source[start..]
        .find("#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("world invalidation commit boundary");
    let commit = &source[start..end];

    assert!(commit.contains("pending.last_mut()"));
    assert!(commit.contains("pending.pop()"));
    assert_eq!(commit.matches(".truncate(").count(), 2);
    assert!(!commit.contains("remove(0)"));
    assert!(!commit.contains(".drain(.."));
}

#[test]
#[ignore = "managed release performance evidence"]
fn world_invalidation_tail_queue_release_benchmark_evidence() {
    const ITEMS: usize = 20_000;
    const SAMPLE_PAIRS: usize = 21;

    let (batch_legacy_ns, batch_optimized_ns) = measure_batch_tail_queue(ITEMS, SAMPLE_PAIRS);
    write_tail_queue_evidence(
        "WORLD_INVALIDATION_BATCH_TAIL_QUEUE_BENCH_V1",
        ITEMS,
        &batch_legacy_ns,
        &batch_optimized_ns,
    );

    let (item_legacy_ns, item_optimized_ns) = measure_item_tail_queue(ITEMS, SAMPLE_PAIRS);
    write_tail_queue_evidence(
        "WORLD_INVALIDATION_ITEM_TAIL_QUEUE_BENCH_V1",
        ITEMS,
        &item_legacy_ns,
        &item_optimized_ns,
    );
}

fn measure_batch_tail_queue(items: usize, sample_pairs: usize) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_samples_ns = Vec::with_capacity(sample_pairs);
    let mut optimized_samples_ns = Vec::with_capacity(sample_pairs);
    for sample_index in 0..sample_pairs {
        let mut legacy: Vec<_> = (0..items).collect();
        let mut optimized: Vec<_> = (0..items).rev().collect();
        let mut measure_legacy = || {
            let started = Instant::now();
            while !legacy.is_empty() {
                black_box(legacy.remove(0));
            }
            legacy_samples_ns.push(started.elapsed().as_nanos());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            while let Some(batch) = optimized.pop() {
                black_box(batch);
            }
            optimized_samples_ns.push(started.elapsed().as_nanos());
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    (legacy_samples_ns, optimized_samples_ns)
}

fn measure_item_tail_queue(items: usize, sample_pairs: usize) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_samples_ns = Vec::with_capacity(sample_pairs);
    let mut optimized_samples_ns = Vec::with_capacity(sample_pairs);
    for sample_index in 0..sample_pairs {
        let mut legacy: Vec<_> = (0..items).collect();
        let mut optimized: Vec<_> = (0..items).rev().collect();
        let mut measure_legacy = || {
            let started = Instant::now();
            while !legacy.is_empty() {
                let item = legacy[0];
                drop(legacy.drain(..1));
                black_box(item);
            }
            legacy_samples_ns.push(started.elapsed().as_nanos());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            while let Some(item) = optimized.last().copied() {
                optimized.truncate(optimized.len() - 1);
                black_box(item);
            }
            optimized_samples_ns.push(started.elapsed().as_nanos());
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    (legacy_samples_ns, optimized_samples_ns)
}

fn write_tail_queue_evidence(
    marker: &str,
    items: usize,
    legacy_samples_ns: &[u128],
    optimized_samples_ns: &[u128],
) {
    assert_eq!(legacy_samples_ns.len(), optimized_samples_ns.len());
    let sample_pairs = legacy_samples_ns.len();
    let legacy_p95_ns = nearest_rank_percentile(legacy_samples_ns, 95);
    let optimized_p95_ns = nearest_rank_percentile(optimized_samples_ns, 95);
    let legacy = join_nanosecond_samples(legacy_samples_ns);
    let optimized = join_nanosecond_samples(optimized_samples_ns);
    let legacy_moves = (items as u128)
        .checked_mul((items - 1) as u128)
        .and_then(|moves| moves.checked_div(2))
        .unwrap();
    println!(
        "{marker} items={items} sample_pairs={sample_pairs} legacy_moves={legacy_moves} \
             optimized_moves=0 legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_ns={legacy} optimized_ns={optimized}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "optimized P95 {optimized_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn join_nanosecond_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}
