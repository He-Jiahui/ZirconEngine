use std::{hint::black_box, sync::Arc, time::Instant};

use super::{UiPersistentSequence, UiPersistentSequenceNode, UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE};

const ITEM_COUNT: usize = 65;
const BUILD_COUNT: usize = 50_000;
const SAMPLE_COUNT: usize = 11;

fn fixed_capacity_sequence(len: usize) -> UiPersistentSequence<u64> {
    let mut items = 0..len as u64;
    let mut nodes = Vec::with_capacity(len.div_ceil(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE));
    let mut item_count = 0usize;

    while let Some(first) = items.next() {
        let mut segment = Vec::with_capacity(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE);
        segment.push(first);
        segment.extend(items.by_ref().take(UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE - 1));
        item_count += segment.len();
        nodes.push(Arc::new(UiPersistentSequenceNode::Segment(segment.into())));
    }

    UiPersistentSequence::from_segment_nodes(nodes, item_count)
}

fn exact_capacity_sequence(len: usize) -> UiPersistentSequence<u64> {
    (0..len as u64).collect()
}

#[test]
fn runtime_interface03_batch66_67_exact_tail_capacity_preserves_layout() {
    for len in [0, 1, 2, 63, 64, 65, 66, 127, 128, 129] {
        let fixed = fixed_capacity_sequence(len);
        let exact = exact_capacity_sequence(len);

        assert_eq!(exact, fixed);
        assert_eq!(exact.len(), fixed.len());
        assert_eq!(exact.segment_count(), fixed.segment_count());
        assert_eq!(exact.directory_depth(), fixed.directory_depth());
        assert_eq!(exact.directory_node_count(), fixed.directory_node_count());
    }
}

#[test]
#[ignore = "release-only exact persistent-sequence tail capacity benchmark"]
fn runtime_interface03_batch66_67_exact_persistent_tail_release_benchmark() {
    let mut fixed_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut exact_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_fixed = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(fixed_capacity_sequence(black_box(ITEM_COUNT)));
            }
            started.elapsed().as_nanos()
        };
        let measure_exact = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(exact_capacity_sequence(black_box(ITEM_COUNT)));
            }
            started.elapsed().as_nanos()
        };

        if sample % 2 == 0 {
            fixed_samples.push(measure_fixed());
            exact_samples.push(measure_exact());
        } else {
            exact_samples.push(measure_exact());
            fixed_samples.push(measure_fixed());
        }
    }

    fixed_samples.sort_unstable();
    exact_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_EXACT_PERSISTENT_TAIL_BENCH_V1 items={ITEM_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} fixed_p50_ns={} exact_p50_ns={} fixed_p95_ns={} exact_p95_ns={}",
        fixed_samples[p50], exact_samples[p50], fixed_samples[p95], exact_samples[p95],
    );
    assert!(
        exact_samples[p95].saturating_mul(10) <= fixed_samples[p95].saturating_mul(9),
        "exact persistent tail capacity must improve P95 by at least 10%: fixed={}ns exact={}ns",
        fixed_samples[p95],
        exact_samples[p95],
    );
}
