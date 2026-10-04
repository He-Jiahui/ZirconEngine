use std::{hint::black_box, sync::Arc, time::Instant};

use super::{
    UiPersistentSequence, UiPersistentSequenceNode, UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT,
    UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH, UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE,
};

const SEGMENT_COUNT: usize = 16_384;
const BUILD_COUNT: usize = 8;
const SAMPLE_COUNT: usize = 11;

fn segment_nodes(count: usize) -> Vec<Arc<UiPersistentSequenceNode<u64>>> {
    (0..count)
        .map(|segment_index| {
            let start = segment_index * UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
            let items = (start..start + UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE)
                .map(|value| value as u64)
                .collect::<Vec<_>>();
            Arc::new(UiPersistentSequenceNode::Segment(items.into()))
        })
        .collect()
}

fn cloned_directory_sequence(
    mut nodes: Vec<Arc<UiPersistentSequenceNode<u64>>>,
) -> UiPersistentSequence<u64> {
    if nodes.is_empty() {
        return UiPersistentSequence::default();
    }

    let segment_count = nodes.len();
    let len = segment_count * UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
    let mut directory_depth = 0_u8;
    let mut directory_node_count = 0usize;
    loop {
        nodes = nodes
            .chunks(UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT)
            .map(|children| {
                directory_node_count += 1;
                Arc::new(UiPersistentSequenceNode::Directory(
                    children.to_vec().into(),
                ))
            })
            .collect();
        directory_depth += 1;
        assert!(usize::from(directory_depth) <= UI_PERSISTENT_SEQUENCE_MAX_DIRECTORY_DEPTH);
        if nodes.len() == 1 {
            break;
        }
    }

    UiPersistentSequence {
        root: nodes.pop(),
        len,
        segment_count,
        directory_depth,
        directory_node_count,
    }
}

fn moved_directory_sequence(
    nodes: Vec<Arc<UiPersistentSequenceNode<u64>>>,
) -> UiPersistentSequence<u64> {
    let len = nodes.len() * UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE;
    UiPersistentSequence::from_segment_nodes(nodes, len)
}

#[test]
fn runtime_interface03_batch66_67_moved_directory_nodes_preserve_layout() {
    for segment_count in [0, 1, 31, 32, 33, 1_023, 1_024, 1_025] {
        let nodes = segment_nodes(segment_count);
        let cloned = cloned_directory_sequence(nodes.clone());
        let moved = moved_directory_sequence(nodes);

        assert_eq!(moved, cloned);
        assert_eq!(moved.len(), cloned.len());
        assert_eq!(moved.segment_count(), cloned.segment_count());
        assert_eq!(moved.directory_depth(), cloned.directory_depth());
        assert_eq!(moved.directory_node_count(), cloned.directory_node_count());
    }
}

#[test]
#[ignore = "release-only moved persistent-sequence directory benchmark"]
fn runtime_interface03_batch66_67_moved_persistent_directory_release_benchmark() {
    let nodes = segment_nodes(SEGMENT_COUNT);
    let mut cloned_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut moved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_cloned = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(cloned_directory_sequence(black_box(nodes.clone())));
            }
            started.elapsed().as_nanos()
        };
        let measure_moved = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(moved_directory_sequence(black_box(nodes.clone())));
            }
            started.elapsed().as_nanos()
        };

        if sample % 2 == 0 {
            cloned_samples.push(measure_cloned());
            moved_samples.push(measure_moved());
        } else {
            moved_samples.push(measure_moved());
            cloned_samples.push(measure_cloned());
        }
    }

    cloned_samples.sort_unstable();
    moved_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_MOVED_PERSISTENT_DIRECTORY_BENCH_V1 segments={SEGMENT_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} cloned_p50_ns={} moved_p50_ns={} cloned_p95_ns={} moved_p95_ns={}",
        cloned_samples[p50], moved_samples[p50], cloned_samples[p95], moved_samples[p95],
    );
    assert!(
        moved_samples[p95].saturating_mul(10) <= cloned_samples[p95].saturating_mul(9),
        "moved persistent directory nodes must improve P95 by at least 10%: cloned={}ns moved={}ns",
        cloned_samples[p95],
        moved_samples[p95],
    );
}
