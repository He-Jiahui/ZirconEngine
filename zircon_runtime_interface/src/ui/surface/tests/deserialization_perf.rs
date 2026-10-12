use std::{hint::black_box, time::Instant};

use serde::{
    de::value::{Error as ValueError, SeqDeserializer},
    Deserialize,
};

use super::UiPersistentSequence;

const ITEM_COUNT: usize = 262_144;
const BUILD_COUNT: usize = 4;
const SAMPLE_COUNT: usize = 11;

fn buffered_deserialize(len: usize) -> UiPersistentSequence<u64> {
    let deserializer = SeqDeserializer::<_, ValueError>::new(0..len as u64);
    Vec::<u64>::deserialize(deserializer)
        .expect("buffered sequence")
        .into()
}

fn streamed_deserialize(len: usize) -> UiPersistentSequence<u64> {
    let deserializer = SeqDeserializer::<_, ValueError>::new(0..len as u64);
    UiPersistentSequence::<u64>::deserialize(deserializer).expect("streamed sequence")
}

#[test]
fn runtime_interface03_batch64_65_streamed_deserialize_preserves_layout() {
    for len in [0, 1, 63, 64, 65, 2_047, 2_048, 2_049] {
        let buffered = buffered_deserialize(len);
        let streamed = streamed_deserialize(len);

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
#[ignore = "release-only streamed persistent-sequence deserialization benchmark"]
fn runtime_interface03_batch64_65_streamed_persistent_deserialize_release_benchmark() {
    let mut buffered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_buffered = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(buffered_deserialize(black_box(ITEM_COUNT)));
            }
            started.elapsed().as_nanos()
        };
        let measure_streamed = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(streamed_deserialize(black_box(ITEM_COUNT)));
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
        "RUNTIME_INTERFACE03_STREAMED_PERSISTENT_DESERIALIZE_BENCH_V1 items={ITEM_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} buffered_p50_ns={} streamed_p50_ns={} buffered_p95_ns={} streamed_p95_ns={}",
        buffered_samples[p50], streamed_samples[p50], buffered_samples[p95], streamed_samples[p95],
    );
    assert!(
        streamed_samples[p95].saturating_mul(10) <= buffered_samples[p95].saturating_mul(9),
        "streamed persistent-sequence deserialization must improve P95 by at least 10%: buffered={}ns streamed={}ns",
        buffered_samples[p95],
        streamed_samples[p95],
    );
}
