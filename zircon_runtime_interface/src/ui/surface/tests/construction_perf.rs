use std::{hint::black_box, time::Instant};

use super::UiPersistentSequence;

const ITEM_COUNT: usize = 131_072;
const BUILD_COUNT: usize = 8;
const SAMPLE_COUNT: usize = 11;

fn buffered_sequence(len: usize) -> UiPersistentSequence<u64> {
    (0..len as u64).collect::<Vec<_>>().into()
}

fn streamed_sequence(len: usize) -> UiPersistentSequence<u64> {
    (0..len as u64).collect()
}

#[test]
fn runtime_interface03_batch62_63_streamed_persistent_sequence_preserves_layout() {
    for len in [0, 1, 63, 64, 65, 2_047, 2_048, 2_049] {
        let buffered = buffered_sequence(len);
        let streamed = streamed_sequence(len);

        assert_eq!(streamed, buffered);
        assert_eq!(streamed.len(), buffered.len());
        assert_eq!(streamed.segment_count(), buffered.segment_count());
        assert_eq!(streamed.directory_depth(), buffered.directory_depth());
        assert_eq!(
            streamed.directory_node_count(),
            buffered.directory_node_count()
        );
    }

    struct NonClone(u64);
    let sequence: UiPersistentSequence<_> = (0..65).map(NonClone).collect();
    assert_eq!(sequence.len(), 65);
    assert_eq!(sequence[64].0, 64);
}

#[test]
#[ignore = "release-only streamed persistent-sequence construction benchmark"]
fn runtime_interface03_batch62_63_streamed_persistent_sequence_release_benchmark() {
    let mut buffered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_buffered = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(buffered_sequence(black_box(ITEM_COUNT)));
            }
            started.elapsed().as_nanos()
        };
        let measure_streamed = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(streamed_sequence(black_box(ITEM_COUNT)));
            }
            started.elapsed().as_nanos()
        };

        if sample % 2 == 0 {
            buffered_samples.push(measure_buffered());
            streamed_samples.push(measure_streamed());
        } else {
            streamed_samples.push(measure_streamed());
            buffered_samples.push(measure_buffered());
        }
    }

    buffered_samples.sort_unstable();
    streamed_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_SEQUENCE_BENCH_V1 items={ITEM_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} buffered_p50_ns={} streamed_p50_ns={} buffered_p95_ns={} streamed_p95_ns={}",
        buffered_samples[p50], streamed_samples[p50], buffered_samples[p95], streamed_samples[p95],
    );
    assert!(
        streamed_samples[p95].saturating_mul(5) <= buffered_samples[p95].saturating_mul(4),
        "streamed persistent-sequence construction must improve P95 by at least 20%: buffered={}ns streamed={}ns",
        buffered_samples[p95],
        streamed_samples[p95],
    );
}

fn buffered_slice(items: &[u64]) -> UiPersistentSequence<u64> {
    items.to_vec().into()
}

fn streamed_slice(items: &[u64]) -> UiPersistentSequence<u64> {
    UiPersistentSequence::from_slice(items)
}

#[test]
fn runtime_interface03_batch62_63_streamed_slice_preserves_layout() {
    for len in [0, 1, 63, 64, 65, 2_047, 2_048, 2_049] {
        let items = (0..len as u64).collect::<Vec<_>>();
        let buffered = buffered_slice(&items);
        let streamed = streamed_slice(&items);

        assert_eq!(streamed, buffered);
        assert_eq!(streamed.len(), buffered.len());
        assert_eq!(streamed.segment_count(), buffered.segment_count());
        assert_eq!(streamed.directory_depth(), buffered.directory_depth());
        assert_eq!(
            streamed.directory_node_count(),
            buffered.directory_node_count()
        );
    }
}

#[test]
#[ignore = "release-only streamed persistent-sequence slice benchmark"]
fn runtime_interface03_batch62_63_streamed_persistent_slice_release_benchmark() {
    let items = (0..ITEM_COUNT as u64).collect::<Vec<_>>();
    let mut buffered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_buffered = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(buffered_slice(black_box(&items)));
            }
            started.elapsed().as_nanos()
        };
        let measure_streamed = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(streamed_slice(black_box(&items)));
            }
            started.elapsed().as_nanos()
        };

        if sample % 2 == 0 {
            buffered_samples.push(measure_buffered());
            streamed_samples.push(measure_streamed());
        } else {
            streamed_samples.push(measure_streamed());
            buffered_samples.push(measure_buffered());
        }
    }

    buffered_samples.sort_unstable();
    streamed_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_SLICE_BENCH_V1 items={ITEM_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} buffered_p50_ns={} streamed_p50_ns={} buffered_p95_ns={} streamed_p95_ns={}",
        buffered_samples[p50], streamed_samples[p50], buffered_samples[p95], streamed_samples[p95],
    );
    assert!(
        streamed_samples[p95].saturating_mul(5) <= buffered_samples[p95].saturating_mul(4),
        "streamed persistent-sequence slice construction must improve P95 by at least 20%: buffered={}ns streamed={}ns",
        buffered_samples[p95],
        streamed_samples[p95],
    );
}
