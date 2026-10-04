#![cfg(target_os = "windows")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use zr_contracts::random::{
    RandomAlgorithmId, RandomEntityKey, RandomPurposeKey, RandomServiceCheckpoint,
    RandomServiceState, RandomState, RandomStreamCheckpoint, RandomStreamKey, RandomSystemKey,
    RandomWorldKey,
};

const STREAM_COUNTS: [usize; 3] = [1, 1_024, RandomServiceCheckpoint::MAX_STREAMS];
const SAMPLE_COUNT: usize = 31;
const AUTHORITY_GENERATION: u64 = 3;

struct CountingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

thread_local! {
    static PROFILE_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_COUNT: Cell<u64> = const { Cell::new(0) };
    static REQUESTED_ALLOCATION_BYTES: Cell<u64> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        record_allocation(pointer, layout.size());
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation(pointer, layout.size());
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        record_allocation(replacement, new_size);
        replacement
    }
}

#[derive(Clone, Copy)]
struct DecodeSample {
    elapsed_ns: u64,
    allocation_count: u64,
    requested_allocation_bytes: u64,
}

#[test]
#[ignore = "managed Windows Release raw checkpoint decode latency and allocation evidence"]
fn runtime210_checkpoint_decode_release_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    for stream_count in STREAM_COUNTS {
        let checkpoint = checkpoint_with_streams(stream_count);
        assert_eq!(checkpoint.streams().len(), stream_count);
        assert_canonical_order(&checkpoint);

        // Fixture construction and serialization stay outside the timed region.
        let encoded = serde_json::to_vec(&checkpoint).expect("checkpoint should serialize");
        let restored: RandomServiceCheckpoint =
            serde_json::from_slice(&encoded).expect("valid checkpoint should decode");
        assert_eq!(restored, checkpoint);
        assert_eq!(restored.streams().len(), stream_count);
        assert_canonical_order(&restored);

        let samples = collect_decode_samples(&encoded, &checkpoint);
        report_samples(stream_count, encoded.len(), &samples);
    }
}

fn checkpoint_with_streams(stream_count: usize) -> RandomServiceCheckpoint {
    assert!(stream_count <= RandomServiceCheckpoint::MAX_STREAMS);

    let service =
        RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 17, AUTHORITY_GENERATION);
    let world = RandomWorldKey::new(1, 1);
    let system = RandomSystemKey::new(7);
    let purpose = RandomPurposeKey::new(11);
    let streams = (0..stream_count)
        .map(|index| {
            let id = index as u64;
            let key = RandomStreamKey::for_entity(
                world,
                RandomEntityKey::new(id, 1),
                system,
                purpose,
                13,
            );
            let state = RandomState::new(RandomAlgorithmId::Pcg32XshRrV1, id, 5, id)
                .expect("odd PCG increment is valid");
            RandomStreamCheckpoint::new(key, state, AUTHORITY_GENERATION)
        })
        .collect();

    RandomServiceCheckpoint::try_new(service, streams)
        .expect("generated streams should form a canonical checkpoint")
}

fn collect_decode_samples(encoded: &[u8], expected: &RandomServiceCheckpoint) -> Vec<DecodeSample> {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);

    for _ in 0..SAMPLE_COUNT {
        begin_profile();
        let started_at = Instant::now();
        let restored: RandomServiceCheckpoint =
            serde_json::from_slice(black_box(encoded)).expect("valid checkpoint should decode");
        let restored = black_box(restored);
        let elapsed_ns = started_at.elapsed().as_nanos() as u64;
        let (allocation_count, requested_allocation_bytes) = finish_profile();

        // Keep correctness checks and destruction outside the timed/allocation window.
        assert_eq!(&restored, expected);
        assert_eq!(restored.streams().len(), expected.streams().len());
        assert_canonical_order(&restored);
        samples.push(DecodeSample {
            elapsed_ns,
            allocation_count,
            requested_allocation_bytes,
        });
    }

    samples
}

fn assert_canonical_order(checkpoint: &RandomServiceCheckpoint) {
    assert!(checkpoint
        .streams()
        .windows(2)
        .all(|pair| pair[0].key() < pair[1].key()));
}

fn report_samples(stream_count: usize, encoded_bytes: usize, samples: &[DecodeSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let allocation_counts = samples
        .iter()
        .map(|sample| sample.allocation_count)
        .collect::<Vec<_>>();
    let requested_allocation_bytes = samples
        .iter()
        .map(|sample| sample.requested_allocation_bytes)
        .collect::<Vec<_>>();

    println!(
        "RUNTIME210_CHECKPOINT_DECODE_PROFILE_V1 streams={stream_count} encoded_bytes={encoded_bytes} samples={} latency_ns_p50={} latency_ns_p95={} latency_ns_p99={} latency_ns_raw={elapsed_ns:?} allocation_count_raw={allocation_counts:?} requested_allocation_bytes_raw={requested_allocation_bytes:?}",
        samples.len(),
        nearest_rank_percentile(&elapsed_ns, 50),
        nearest_rank_percentile(&elapsed_ns, 95),
        nearest_rank_percentile(&elapsed_ns, 99),
    );
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = percentile.saturating_mul(ordered.len()).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn begin_profile() {
    ALLOCATION_COUNT.with(|count| count.set(0));
    REQUESTED_ALLOCATION_BYTES.with(|bytes| bytes.set(0));
    PROFILE_ACTIVE.with(|active| assert!(!active.replace(true)));
}

fn finish_profile() -> (u64, u64) {
    PROFILE_ACTIVE.with(|active| assert!(active.replace(false)));
    (
        ALLOCATION_COUNT.with(Cell::get),
        REQUESTED_ALLOCATION_BYTES.with(Cell::get),
    )
}

fn record_allocation(pointer: *mut u8, bytes: usize) {
    if pointer.is_null() {
        return;
    }
    let Ok(is_active) = PROFILE_ACTIVE.try_with(Cell::get) else {
        return;
    };
    if !is_active {
        return;
    }
    let _ = ALLOCATION_COUNT.try_with(|count| count.set(count.get().saturating_add(1)));
    let _ = REQUESTED_ALLOCATION_BYTES
        .try_with(|total| total.set(total.get().saturating_add(bytes as u64)));
}
