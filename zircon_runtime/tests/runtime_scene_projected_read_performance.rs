#![cfg(target_os = "windows")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{ActiveInHierarchy, WorldMatrix};
use zircon_runtime::scene::{NodeKind, NodeRecord, World};

const CHAIN_DEPTHS: [usize; 3] = [1, 32, 1_024];
const SAMPLE_COUNT: usize = 31;
const QUERIES_PER_SAMPLE: usize = 256;

struct CountingAllocator;

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;
static PROFILE_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

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
struct ProfileSample {
    elapsed_ns: u64,
    allocation_count: u64,
    allocated_bytes: u64,
}

#[test]
#[ignore = "managed Windows Release raw latency and allocation evidence"]
fn runtime1021_dirty_projected_reads_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    for depth in CHAIN_DEPTHS {
        let (world, leaf) = dirty_chain_world(depth);
        let world_matrix_samples = collect_samples(|| {
            black_box(world.world_matrix(leaf));
        });
        report_samples("world_matrix", depth, &world_matrix_samples);

        let active_samples = collect_samples(|| {
            black_box(world.active_in_hierarchy(leaf));
        });
        report_samples("active_in_hierarchy", depth, &active_samples);
    }
}

#[test]
#[ignore = "managed Windows Release clean world-matrix latency and allocation evidence for Runtime62 G19"]
fn runtime62_clean_world_matrix_allocation_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    let world = World::new();
    let camera = world.active_camera();
    let cached = world
        .get::<WorldMatrix>(camera)
        .expect("World::new should publish the active camera world matrix")
        .0;
    assert_eq!(world.world_matrix(camera), Some(cached));

    let samples = collect_samples(|| {
        black_box(world.world_matrix(camera));
    });
    report_clean_samples(
        "RUNTIME62_CLEAN_WORLD_MATRIX_ALLOCATION_PROFILE_V1",
        "world_matrix",
        1,
        &samples,
    );
}

#[test]
#[ignore = "managed Windows Release clean active-hierarchy allocation evidence for Runtime62 G19"]
fn runtime62_clean_active_in_hierarchy_allocation_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    let world = World::new();
    let camera = world.active_camera();
    let cached = world
        .get::<ActiveInHierarchy>(camera)
        .expect("World::new should publish the active camera hierarchy state")
        .0;
    assert_eq!(world.active_in_hierarchy(camera), Some(cached));

    let samples = collect_samples(|| {
        black_box(world.active_in_hierarchy(camera));
    });
    report_clean_samples(
        "RUNTIME62_CLEAN_ACTIVE_ALLOCATION_PROFILE_V1",
        "active_in_hierarchy",
        1,
        &samples,
    );
}

fn dirty_chain_world(depth: usize) -> (World, u64) {
    let mut template_world = World::empty();
    let template_entity = template_world
        .spawn_node(NodeKind::Empty)
        .expect("profile fixture template should spawn");
    let template = template_world
        .node_record(template_entity)
        .expect("profile fixture template should have a node record");

    let first_id = 10_000_u64;
    let mut records = Vec::with_capacity(depth);
    for index in 0..depth {
        let id = first_id + index as u64;
        let mut record: NodeRecord = template.clone();
        record.id = id;
        record.name = format!("Projected read profile {index}");
        record.parent = index.checked_sub(1).map(|parent| first_id + parent as u64);
        record.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        record.active = true;
        records.push(record);
    }
    let leaf = records
        .last()
        .expect("profile hierarchy depth must be nonzero")
        .id;

    let mut world = World::empty();
    world
        .insert_node_records(&records)
        .expect("profile hierarchy records should publish");
    assert!(world.get::<ActiveInHierarchy>(leaf).is_none());
    assert_eq!(world.active_in_hierarchy(leaf), Some(true));
    assert!(world.get::<ActiveInHierarchy>(leaf).is_none());
    (world, leaf)
}

fn collect_samples(mut operation: impl FnMut()) -> Vec<ProfileSample> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            begin_profile();
            let started_at = Instant::now();
            for _ in 0..QUERIES_PER_SAMPLE {
                operation();
            }
            let elapsed_ns = started_at.elapsed().as_nanos() as u64;
            let (allocation_count, allocated_bytes) = finish_profile();

            ProfileSample {
                elapsed_ns,
                allocation_count,
                allocated_bytes,
            }
        })
        .collect()
}

fn report_samples(query: &str, depth: usize, samples: &[ProfileSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let latency_ns_per_query = elapsed_ns
        .iter()
        .map(|elapsed| elapsed / QUERIES_PER_SAMPLE as u64)
        .collect::<Vec<_>>();
    let allocation_counts = samples
        .iter()
        .map(|sample| sample.allocation_count)
        .collect::<Vec<_>>();
    let allocated_bytes = samples
        .iter()
        .map(|sample| sample.allocated_bytes)
        .collect::<Vec<_>>();

    println!(
        "RUNTIME1021_DIRTY_PROJECTED_READ_PROFILE_V1 query={query} depth={depth} samples={SAMPLE_COUNT} queries_per_sample={QUERIES_PER_SAMPLE} latency_ns_per_query_p50={} latency_ns_per_query_p95={} latency_ns_per_query_p99={} elapsed_ns_raw={elapsed_ns:?} latency_ns_per_query_raw={latency_ns_per_query:?} allocation_count_raw={allocation_counts:?} allocated_bytes_raw={allocated_bytes:?}",
        nearest_rank_percentile(&latency_ns_per_query, 50),
        nearest_rank_percentile(&latency_ns_per_query, 95),
        nearest_rank_percentile(&latency_ns_per_query, 99),
    );
    assert!(
        samples
            .iter()
            .all(|sample| sample.allocation_count == 0 && sample.allocated_bytes == 0),
        "dirty {query} reads at depth {depth} must allocate no heap memory"
    );
}

fn report_clean_samples(marker: &str, query: &str, depth: usize, samples: &[ProfileSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let latency_ns_per_query = elapsed_ns
        .iter()
        .map(|elapsed| elapsed / QUERIES_PER_SAMPLE as u64)
        .collect::<Vec<_>>();
    let allocation_counts = samples
        .iter()
        .map(|sample| sample.allocation_count)
        .collect::<Vec<_>>();
    let allocated_bytes = samples
        .iter()
        .map(|sample| sample.allocated_bytes)
        .collect::<Vec<_>>();

    println!(
        "{marker} query={query} depth={depth} samples={SAMPLE_COUNT} queries_per_sample={QUERIES_PER_SAMPLE} latency_ns_per_query_p50={} latency_ns_per_query_p95={} latency_ns_per_query_p99={} allocation_count_p50={} allocation_count_p95={} allocation_count_p99={} allocated_bytes_p50={} allocated_bytes_p95={} allocated_bytes_p99={} elapsed_ns_raw={elapsed_ns:?} latency_ns_per_query_raw={latency_ns_per_query:?} allocation_count_raw={allocation_counts:?} allocated_bytes_raw={allocated_bytes:?}",
        nearest_rank_percentile(&latency_ns_per_query, 50),
        nearest_rank_percentile(&latency_ns_per_query, 95),
        nearest_rank_percentile(&latency_ns_per_query, 99),
        nearest_rank_percentile(&allocation_counts, 50),
        nearest_rank_percentile(&allocation_counts, 95),
        nearest_rank_percentile(&allocation_counts, 99),
        nearest_rank_percentile(&allocated_bytes, 50),
        nearest_rank_percentile(&allocated_bytes, 95),
        nearest_rank_percentile(&allocated_bytes, 99),
    );
    assert!(
        samples
            .iter()
            .all(|sample| sample.allocation_count == 0 && sample.allocated_bytes == 0),
        "clean {query} reads at depth {depth} must allocate no heap memory"
    );
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = percentile.saturating_mul(ordered.len()).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn begin_profile() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    assert!(!PROFILE_ACTIVE.swap(true, Ordering::SeqCst));
}

fn finish_profile() -> (u64, u64) {
    assert!(PROFILE_ACTIVE.swap(false, Ordering::SeqCst));
    (
        ALLOCATION_COUNT.load(Ordering::Relaxed),
        ALLOCATED_BYTES.load(Ordering::Relaxed),
    )
}

fn record_allocation(pointer: *mut u8, size: usize) {
    if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}
