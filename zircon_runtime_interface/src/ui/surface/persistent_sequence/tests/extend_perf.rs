use std::{hint::black_box, time::Instant};

use super::UiPersistentSequence;

const EXISTING_COUNT: usize = 131_072;
const APPEND_COUNT: usize = 4_096;
const BUILD_COUNT: usize = 8;
const SAMPLE_COUNT: usize = 11;

fn buffered_extend(
    sequence: &UiPersistentSequence<u64>,
    append_count: usize,
) -> UiPersistentSequence<u64> {
    let mut items = sequence.to_vec();
    items.extend((0..append_count as u64).map(|value| value + sequence.len() as u64));
    items.into()
}

fn streamed_extend(
    sequence: &UiPersistentSequence<u64>,
    append_count: usize,
) -> UiPersistentSequence<u64> {
    let mut next = sequence.clone();
    next.extend((0..append_count as u64).map(|value| value + sequence.len() as u64));
    next
}

#[test]
fn runtime_interface03_batch64_65_streamed_extend_preserves_layout() {
    for (existing_len, append_len) in [
        (0, 0),
        (0, 65),
        (1, 0),
        (63, 2),
        (64, 1),
        (65, 64),
        (2_047, 2),
        (2_048, 65),
    ] {
        let sequence = (0..existing_len as u64).collect::<UiPersistentSequence<_>>();
        let buffered = buffered_extend(&sequence, append_len);
        let streamed = streamed_extend(&sequence, append_len);

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
#[ignore = "release-only streamed persistent-sequence extension benchmark"]
fn runtime_interface03_batch64_65_streamed_persistent_extend_release_benchmark() {
    let sequence = (0..EXISTING_COUNT as u64).collect::<UiPersistentSequence<_>>();
    let mut buffered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_buffered = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(buffered_extend(
                    black_box(&sequence),
                    black_box(APPEND_COUNT),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_streamed = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(streamed_extend(
                    black_box(&sequence),
                    black_box(APPEND_COUNT),
                ));
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
        "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_EXTEND_BENCH_V1 existing={EXISTING_COUNT} appended={APPEND_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} buffered_p50_ns={} streamed_p50_ns={} buffered_p95_ns={} streamed_p95_ns={}",
        buffered_samples[p50], streamed_samples[p50], buffered_samples[p95], streamed_samples[p95],
    );
    assert!(
        streamed_samples[p95].saturating_mul(5) <= buffered_samples[p95].saturating_mul(4),
        "streamed persistent-sequence extension must improve P95 by at least 20%: buffered={}ns streamed={}ns",
        buffered_samples[p95],
        streamed_samples[p95],
    );
}
