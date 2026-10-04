use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const UPDATE_COUNT: usize = 16_384;

#[test]
fn optimization_batch_jq_editor656_catalog_updates_use_the_iterator_lower_bound() {
    let source = include_str!("../../generation.rs");

    assert!(source.contains("let updates = updates.into_iter();"));
    assert!(source.contains("HashMap::with_capacity(updates.size_hint().0)"));
}

#[test]
#[ignore = "Windows Release performance gate"]
fn optimization_batch_jq_editor656_catalog_update_index_capacity_benchmark() {
    for _ in 0..4 {
        black_box(measure(false));
        black_box(measure(true));
    }
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut unreserved_growths = 0;
    let mut reserved_growths = 0;
    for pair_index in 0..SAMPLE_PAIRS {
        let (unreserved, reserved) = if pair_index % 2 == 0 {
            (measure(false), measure(true))
        } else {
            let reserved = measure(true);
            (measure(false), reserved)
        };
        unreserved_samples.push(unreserved.0);
        reserved_samples.push(reserved.0);
        unreserved_growths = unreserved.1;
        reserved_growths = reserved.1;
    }
    let unreserved_p95 = nearest_rank_p95(&unreserved_samples);
    let reserved_p95 = nearest_rank_p95(&reserved_samples);
    println!(
        "EDITOR656_CATALOG_UPDATE_INDEX_CAPACITY_BENCH_V1 \
sample_pairs={SAMPLE_PAIRS} update_count={UPDATE_COUNT} \
unreserved_p95_ns={unreserved_p95} reserved_p95_ns={reserved_p95} \
unreserved_growths={unreserved_growths} reserved_growths={reserved_growths}"
    );
    assert_eq!(reserved_growths, 0);
    assert!(unreserved_growths > reserved_growths);
    assert!(reserved_p95.saturating_mul(100) <= unreserved_p95.saturating_mul(80));
}

fn measure(reserved: bool) -> (u128, usize) {
    let started = Instant::now();
    let mut growths = 0;
    let updates = (0..UPDATE_COUNT).collect::<Vec<_>>();
    let iterator = updates.into_iter();
    let mut index = if reserved {
        HashMap::with_capacity(iterator.size_hint().0)
    } else {
        HashMap::new()
    };
    for update in iterator {
        let old_capacity = index.capacity();
        index.insert(black_box(update), black_box(update));
        growths += usize::from(index.capacity() != old_capacity);
    }
    black_box(index);
    (started.elapsed().as_nanos().max(1), growths)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
