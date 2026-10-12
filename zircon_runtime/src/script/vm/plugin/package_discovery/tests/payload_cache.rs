use std::{
    collections::{BTreeMap, HashMap},
    hint::black_box,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use super::{read_bounded_stream, CachedPayload, PayloadCacheState, PayloadFingerprint};

const PERF_SAMPLE_PAIRS: usize = 21;

struct ChunkedReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    chunk_bytes: usize,
}

impl<'a> ChunkedReader<'a> {
    fn new(bytes: &'a [u8], chunk_bytes: usize) -> Self {
        Self {
            bytes,
            offset: 0,
            chunk_bytes,
        }
    }
}

impl Read for ChunkedReader<'_> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        if self.offset == self.bytes.len() {
            return Ok(0);
        }
        let read_bytes = destination
            .len()
            .min(self.chunk_bytes)
            .min(self.bytes.len() - self.offset);
        destination[..read_bytes]
            .copy_from_slice(&self.bytes[self.offset..self.offset + read_bytes]);
        self.offset += read_bytes;
        Ok(read_bytes)
    }
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn duration_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(Duration::as_nanos)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn vm_payload_cache_uses_hash_index_with_borrowed_paths() {
    fn assert_hash_index(_: &HashMap<PathBuf, Arc<CachedPayload>>) {}

    let path = PathBuf::from("cache/package/main.zrbc");
    let payload = Arc::new(CachedPayload {
        fingerprint: PayloadFingerprint {
            len: 128,
            modified: None,
        },
        bytes: OnceLock::new(),
    });
    let mut state = PayloadCacheState::default();
    state.entries.insert(path.clone(), Arc::clone(&payload));

    assert_hash_index(&state.entries);
    assert!(Arc::ptr_eq(
        state
            .entries
            .get(Path::new("cache/package/main.zrbc"))
            .unwrap(),
        &payload
    ));
}

#[test]
fn vm_payload_bounded_reader_reserves_known_length_and_keeps_overflow_sentinel() {
    let payload = vec![0x5a; 16 * 1024];
    let bytes = read_bounded_stream(
        ChunkedReader::new(&payload, 257),
        payload.len(),
        payload.len(),
    )
    .unwrap();
    assert_eq!(bytes, payload);
    assert!(bytes.capacity() >= payload.len());

    let oversized = vec![0xa5; 1_025];
    let bytes =
        read_bounded_stream(ChunkedReader::new(&oversized, 31), oversized.len(), 1_024).unwrap();
    assert_eq!(bytes.len(), 1_025);
}

#[test]
#[ignore = "managed Runtime07 performance evidence"]
fn vm_payload_cache_runtime07_performance_hash_lookup() {
    const ENTRIES: usize = 16_384;
    let paths = (0..ENTRIES)
        .map(|index| PathBuf::from(format!("packages/{index:05}/module/main.zrbc")))
        .collect::<Vec<_>>();
    let legacy = paths
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, path)| (path, index))
        .collect::<BTreeMap<_, _>>();
    let optimized = paths
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, path)| (path, index))
        .collect::<HashMap<_, _>>();

    let legacy_lookup = || {
        (0..ENTRIES).fold(0usize, |sum, index| {
            let query = &paths[(index * 8_191) % ENTRIES];
            sum.wrapping_add(*black_box(legacy.get(query.as_path()).unwrap()))
        })
    };
    let optimized_lookup = || {
        (0..ENTRIES).fold(0usize, |sum, index| {
            let query = &paths[(index * 8_191) % ENTRIES];
            sum.wrapping_add(*black_box(optimized.get(query.as_path()).unwrap()))
        })
    };
    assert_eq!(legacy_lookup(), optimized_lookup());
    black_box(legacy_lookup());
    black_box(optimized_lookup());

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair in 0..PERF_SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            black_box(legacy_lookup());
            legacy_samples.push(started.elapsed());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            black_box(optimized_lookup());
            optimized_samples.push(started.elapsed());
        };
        if pair % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let legacy_csv = duration_csv(&legacy_samples);
    let optimized_csv = duration_csv(&optimized_samples);
    eprintln!(
        "RUNTIME07_PAYLOAD_HASH_LOOKUP_BENCH_V1 entries={ENTRIES} lookups_per_sample={ENTRIES} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100)
            <= legacy_p95.as_nanos().saturating_mul(75),
        "hash payload lookup must reduce P95 by at least 25%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

#[test]
#[ignore = "managed Runtime07 performance evidence"]
fn vm_payload_cache_runtime07_performance_bounded_read_capacity() {
    const PAYLOAD_BYTES: usize = 8 * 1024 * 1024;
    const CHUNK_BYTES: usize = 8 * 1024;
    let payload = vec![0x3c; PAYLOAD_BYTES];

    let legacy_read = || {
        let bytes =
            read_bounded_stream(ChunkedReader::new(&payload, CHUNK_BYTES), 0, PAYLOAD_BYTES)
                .unwrap();
        black_box(bytes.len() + usize::from(bytes[PAYLOAD_BYTES - 1]))
    };
    let optimized_read = || {
        let bytes = read_bounded_stream(
            ChunkedReader::new(&payload, CHUNK_BYTES),
            PAYLOAD_BYTES,
            PAYLOAD_BYTES,
        )
        .unwrap();
        black_box(bytes.len() + usize::from(bytes[PAYLOAD_BYTES - 1]))
    };
    assert_eq!(legacy_read(), optimized_read());
    black_box(legacy_read());
    black_box(optimized_read());

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair in 0..PERF_SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            black_box(legacy_read());
            legacy_samples.push(started.elapsed());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            black_box(optimized_read());
            optimized_samples.push(started.elapsed());
        };
        if pair % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let legacy_csv = duration_csv(&legacy_samples);
    let optimized_csv = duration_csv(&optimized_samples);
    eprintln!(
        "RUNTIME07_PAYLOAD_READ_CAPACITY_BENCH_V1 payload_bytes={PAYLOAD_BYTES} chunk_bytes={CHUNK_BYTES} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_initial_capacity=0 optimized_initial_capacity={PAYLOAD_BYTES} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100)
            <= legacy_p95.as_nanos().saturating_mul(90),
        "known-length payload read must reduce P95 by at least 10%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}
