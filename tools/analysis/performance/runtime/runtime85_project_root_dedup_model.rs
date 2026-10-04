use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::{HashSet, hash_map::RandomState};
use std::hash::BuildHasher;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

const ROOT_COUNT: usize = 1_024;
const SAMPLE_COUNT: usize = 21;
const WARMUP_COUNT: usize = 5;

struct CountingAllocator;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation(new_size);
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

fn record_allocation(bytes: usize) {
    if COUNTING.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(bytes, Ordering::Relaxed);
    }
}

fn main() {
    let roots = (0..ROOT_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "C:/zircon/project/assets/long_shared_prefix/root_{index:04}"
            ))
        })
        .collect::<Vec<_>>();

    let legacy = legacy_unique(&roots);
    let indexed = indexed_unique(&roots).0;
    assert_eq!(legacy, indexed);

    let mut duplicate_fixture = roots.clone();
    duplicate_fixture.push(roots.last().expect("nonempty root corpus").clone());
    assert_eq!(
        legacy_duplicate_index(&duplicate_fixture),
        Some(ROOT_COUNT - 1)
    );
    let (_, indexed_duplicate, _) = indexed_admit(&duplicate_fixture);
    assert_eq!(indexed_duplicate, Some(ROOT_COUNT - 1));

    let (legacy_allocations, legacy_bytes) = measure_allocations(|| legacy_unique(&roots));
    let (indexed_allocations, indexed_bytes) = measure_allocations(|| indexed_unique(&roots).0);
    assert_eq!(legacy_allocations, ROOT_COUNT + 1);
    assert!(indexed_allocations <= legacy_allocations + 2);

    for _ in 0..WARMUP_COUNT {
        black_box(legacy_unique(black_box(&roots)));
        black_box(indexed_unique(black_box(&roots)).0);
    }

    let (legacy_ns, indexed_ns) = paired_samples(
        || checksum(&legacy_unique(black_box(&roots))),
        || checksum(&indexed_unique(black_box(&roots)).0),
    );
    let expected_checksum = checksum(&roots);
    assert_eq!(
        black_box(legacy_unique(&roots))
            .iter()
            .map(|path| path.as_os_str().len())
            .sum::<usize>(),
        expected_checksum
    );
    assert_eq!(
        black_box(indexed_unique(&roots).0)
            .iter()
            .map(|path| path.as_os_str().len())
            .sum::<usize>(),
        expected_checksum
    );

    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let indexed_p50 = percentile(&indexed_ns, 50);
    let indexed_p95 = percentile(&indexed_ns, 95);
    let legacy_comparisons = ROOT_COUNT * (ROOT_COUNT - 1) / 2;
    let (_, _, collision_comparisons) = indexed_admit(&roots);

    assert_eq!(legacy_comparisons, 523_776);
    assert_eq!(collision_comparisons, 0);
    assert!(indexed_p50.saturating_mul(10) <= legacy_p50.saturating_mul(3));
    assert!(indexed_p95.saturating_mul(2) <= legacy_p95);

    println!(
        "RUNTIME85_PROJECT_ROOT_DEDUP_MODEL_V1 roots={ROOT_COUNT} samples={SAMPLE_COUNT} legacy_path_comparisons={legacy_comparisons} indexed_hash_probes={ROOT_COUNT} collision_path_comparisons={collision_comparisons} legacy_allocations={legacy_allocations} indexed_allocations={indexed_allocations} legacy_allocated_bytes={legacy_bytes} indexed_allocated_bytes={indexed_bytes} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} indexed_p50_ns={indexed_p50} indexed_p95_ns={indexed_p95} checksum={expected_checksum} legacy_ns={} indexed_ns={}",
        samples(&legacy_ns),
        samples(&indexed_ns)
    );
}

fn legacy_unique(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut unique = Vec::with_capacity(roots.len());
    for root in roots {
        if !unique.contains(root) {
            unique.push(root.clone());
        }
    }
    unique
}

fn indexed_unique(roots: &[PathBuf]) -> (Vec<PathBuf>, usize) {
    let (unique, duplicate, collision_comparisons) = indexed_admit(roots);
    assert_eq!(duplicate, None);
    (unique, collision_comparisons)
}

fn indexed_admit(roots: &[PathBuf]) -> (Vec<PathBuf>, Option<usize>, usize) {
    let hasher = RandomState::new();
    let mut hashes = HashSet::with_capacity(roots.len());
    let mut unique = Vec::with_capacity(roots.len());
    let mut collision_comparisons = 0;
    for root in roots {
        let hash = hasher.hash_one(root);
        if !hashes.insert(hash) {
            collision_comparisons += unique.len();
            if let Some(index) = unique.iter().position(|candidate| candidate == root) {
                return (unique, Some(index), collision_comparisons);
            }
        }
        unique.push(root.clone());
    }
    (unique, None, collision_comparisons)
}

fn legacy_duplicate_index(roots: &[PathBuf]) -> Option<usize> {
    let mut unique = Vec::with_capacity(roots.len());
    for root in roots {
        if let Some(index) = unique.iter().position(|candidate| candidate == root) {
            return Some(index);
        }
        unique.push(root.clone());
    }
    None
}

fn measure_allocations(operation: impl FnOnce() -> Vec<PathBuf>) -> (usize, usize) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    let result = black_box(operation());
    COUNTING.store(false, Ordering::Relaxed);
    let counts = (
        ALLOCATIONS.load(Ordering::Relaxed),
        ALLOCATED_BYTES.load(Ordering::Relaxed),
    );
    black_box(result);
    counts
}

fn paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut indexed: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_ns = Vec::with_capacity(SAMPLE_COUNT);
    for index in 0..SAMPLE_COUNT {
        if index % 2 == 0 {
            legacy_ns.push(measure_time(&mut legacy));
            indexed_ns.push(measure_time(&mut indexed));
        } else {
            indexed_ns.push(measure_time(&mut indexed));
            legacy_ns.push(measure_time(&mut legacy));
        }
    }
    (legacy_ns, indexed_ns)
}

fn measure_time(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn checksum(paths: &[PathBuf]) -> usize {
    paths.iter().map(|path| path.as_os_str().len()).sum()
}

fn percentile(values: &[u128], percentile: usize) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}

fn samples(values: &[u128]) -> String {
    values
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
