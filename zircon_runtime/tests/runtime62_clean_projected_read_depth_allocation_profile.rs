use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::runtime::{CoreHandle, CoreRuntime};
use zircon_runtime::scene::components::{ActiveInHierarchy, NodeKind, NodeRecord, WorldMatrix};
use zircon_runtime::scene::ecs::WorldDerivedStateDiagnostics;
use zircon_runtime::scene::{self, LevelSystem, World};

const CHAIN_DEPTHS: [usize; 3] = [1, 32, 1_024];
const SAMPLE_COUNT: usize = 31;
const QUERIES_PER_SAMPLE: usize = 256;
const FIRST_ENTITY: u64 = 3_000_000;

#[derive(Clone, Copy, Default)]
struct AllocationWindow {
    enabled: bool,
    request_calls: u64,
    requested_bytes: u64,
}

thread_local! {
    static ALLOCATION_WINDOW: Cell<AllocationWindow> = const {
        Cell::new(AllocationWindow {
            enabled: false,
            request_calls: 0,
            requested_bytes: 0,
        })
    };
}

struct ThreadLocalCountingAllocator;

#[global_allocator]
static PROFILE_ALLOCATOR: ThreadLocalCountingAllocator = ThreadLocalCountingAllocator;

unsafe impl GlobalAlloc for ThreadLocalCountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        record_allocation_request(layout.size());
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation_request(layout.size());
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        record_allocation_request(new_size);
        replacement
    }
}

#[derive(Clone, Copy)]
enum ProjectedRead {
    WorldMatrix,
    ActiveInHierarchy,
}

impl ProjectedRead {
    const fn label(self) -> &'static str {
        match self {
            Self::WorldMatrix => "world_matrix",
            Self::ActiveInHierarchy => "active_in_hierarchy",
        }
    }
}

#[derive(Clone, Copy)]
struct ReadSample {
    elapsed_ns: u64,
    request_calls: u64,
    gross_requested_bytes: u64,
}

#[test]
fn runtime62_clean_projected_reads_match_published_values_at_chain_depths() {
    warm_allocation_window();

    for depth in CHAIN_DEPTHS {
        let (_runtime, _core, level, leaf) = level_with_committed_chain(depth);
        assert_published_values_match(&level, depth, leaf);

        for read in [ProjectedRead::WorldMatrix, ProjectedRead::ActiveInHierarchy] {
            let sample = measure_read_batch(&level, leaf, read, 1);
            assert_zero_allocations(sample, depth, read);
        }
    }
}

#[test]
#[ignore = "managed Windows Release Runtime62 G19 clean-read depth and allocation samples"]
fn runtime62_clean_projected_read_depth_allocation_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );
    warm_allocation_window();

    for depth in CHAIN_DEPTHS {
        let (_runtime, _core, level, leaf) = level_with_committed_chain(depth);
        assert_published_values_match(&level, depth, leaf);

        for read in [ProjectedRead::WorldMatrix, ProjectedRead::ActiveInHierarchy] {
            let samples = (0..SAMPLE_COUNT)
                .map(|_| measure_read_batch(&level, leaf, read, QUERIES_PER_SAMPLE))
                .collect::<Vec<_>>();
            for sample in &samples {
                assert_zero_allocations(*sample, depth, read);
            }
            report_samples(depth, read, &samples);
        }
    }
}

fn level_with_committed_chain(depth: usize) -> (CoreRuntime, CoreHandle, LevelSystem, u64) {
    let runtime = CoreRuntime::new();
    for descriptor in [
        zircon_runtime::foundation::module_descriptor(),
        zircon_runtime::asset::module_descriptor(),
        scene::module_descriptor(),
    ] {
        runtime
            .register_module(descriptor)
            .expect("runtime module registration should succeed");
    }
    for module_name in [
        zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
        zircon_runtime::asset::ASSET_MODULE_NAME,
        SCENE_MODULE_NAME,
    ] {
        runtime
            .activate_module(module_name)
            .expect("runtime module activation should succeed");
    }

    let core = runtime.handle();
    let level = scene::create_default_level(&core).expect("default level should be available");
    let (world, leaf) = chain_world(depth);
    level.replace_world_and_reset_runtime_state(world);
    level
        .tick(&core, runtime.tick_time(4))
        .expect("initial derived-state publication should complete");
    level
        .tick(&core, runtime.tick_time(4))
        .expect("stable follow-up tick should complete");
    assert_no_derived_propagation_on_stable_tick(&level);

    (runtime, core, level, leaf)
}

fn chain_world(depth: usize) -> (World, u64) {
    assert!(depth > 0, "chain depth must be nonzero");
    let mut template_world = World::empty();
    let template_entity = template_world
        .spawn_node(NodeKind::Empty)
        .expect("clean-read profile template should spawn");
    let template = template_world
        .node_record(template_entity)
        .expect("clean-read profile template should have a node record");

    let mut records = Vec::with_capacity(depth);
    for index in 0..depth {
        let mut record: NodeRecord = template.clone();
        record.id = FIRST_ENTITY + index as u64;
        record.name = format!("Runtime62 clean projected read {index}");
        record.parent = index
            .checked_sub(1)
            .map(|parent_index| FIRST_ENTITY + parent_index as u64);
        record.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        records.push(record);
    }
    let leaf = records.last().expect("chain depth is nonzero").id;

    let mut world = World::empty();
    world
        .insert_owned_node_records(records)
        .expect("clean-read profile chain should publish");
    (world, leaf)
}

fn assert_no_derived_propagation_on_stable_tick(level: &LevelSystem) {
    level.with_world(|world| {
        let diagnostics: WorldDerivedStateDiagnostics =
            world.ecs_frame_performance_diagnostics().derived_state;
        assert_eq!(diagnostics.world_matrix_propagation_passes, 0);
        assert_eq!(diagnostics.active_propagation_passes, 0);
    });
}

fn assert_published_values_match(level: &LevelSystem, depth: usize, leaf: u64) {
    level.with_world(|world| {
        let cached_matrix = world
            .get::<WorldMatrix>(leaf)
            .expect("public level tick should publish the leaf world matrix")
            .0;
        let cached_active = world
            .get::<ActiveInHierarchy>(leaf)
            .expect("public level tick should publish leaf active state")
            .0;

        assert_eq!(world.world_matrix(leaf), Some(cached_matrix));
        assert_eq!(world.active_in_hierarchy(leaf), Some(cached_active));
        assert!(cached_active);
        assert_eq!(
            world.world_transform(leaf).unwrap().translation.x,
            depth as f32
        );
    });
}

fn measure_read_batch(
    level: &LevelSystem,
    leaf: u64,
    read: ProjectedRead,
    query_count: usize,
) -> ReadSample {
    level.with_world(|world| {
        begin_allocation_window();
        let started_at = Instant::now();
        for _ in 0..query_count {
            match read {
                ProjectedRead::WorldMatrix => {
                    black_box(world.world_matrix(leaf));
                }
                ProjectedRead::ActiveInHierarchy => {
                    black_box(world.active_in_hierarchy(leaf));
                }
            }
        }
        let elapsed_ns = started_at.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        let (request_calls, gross_requested_bytes) = finish_allocation_window();
        ReadSample {
            elapsed_ns,
            request_calls,
            gross_requested_bytes,
        }
    })
}

fn assert_zero_allocations(sample: ReadSample, depth: usize, read: ProjectedRead) {
    assert_eq!(
        sample.request_calls,
        0,
        "clean {} query at depth {depth} requested heap allocation",
        read.label()
    );
    assert_eq!(
        sample.gross_requested_bytes,
        0,
        "clean {} query at depth {depth} requested heap bytes",
        read.label()
    );
}

fn report_samples(depth: usize, read: ProjectedRead, samples: &[ReadSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let request_calls = samples
        .iter()
        .map(|sample| sample.request_calls)
        .collect::<Vec<_>>();
    let gross_requested_bytes = samples
        .iter()
        .map(|sample| sample.gross_requested_bytes)
        .collect::<Vec<_>>();

    println!(
        "RUNTIME62_CLEAN_PROJECTED_READ_DEPTH_ALLOCATION_PROFILE_V1 shape=chain query={} depth={depth} samples={SAMPLE_COUNT} queries_per_sample={QUERIES_PER_SAMPLE} measured_scope=clean_public_query_loop allocator_scope=current_thread_tls allocation_bytes_kind=gross_requested_bytes_including_realloc elapsed_ns_per_batch_p50={} elapsed_ns_per_batch_p95={} elapsed_ns_per_batch_p99={} request_calls_p50={} request_calls_p95={} request_calls_p99={} gross_requested_bytes_p50={} gross_requested_bytes_p95={} gross_requested_bytes_p99={} elapsed_ns_raw={elapsed_ns:?} request_calls_raw={request_calls:?} gross_requested_bytes_raw={gross_requested_bytes:?}",
        read.label(),
        nearest_rank_percentile(&elapsed_ns, 50),
        nearest_rank_percentile(&elapsed_ns, 95),
        nearest_rank_percentile(&elapsed_ns, 99),
        nearest_rank_percentile(&request_calls, 50),
        nearest_rank_percentile(&request_calls, 95),
        nearest_rank_percentile(&request_calls, 99),
        nearest_rank_percentile(&gross_requested_bytes, 50),
        nearest_rank_percentile(&gross_requested_bytes, 95),
        nearest_rank_percentile(&gross_requested_bytes, 99),
    );
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn warm_allocation_window() {
    let _ = ALLOCATION_WINDOW.try_with(|window| window.get());
}

fn begin_allocation_window() {
    ALLOCATION_WINDOW.with(|window| {
        let previous = window.replace(AllocationWindow {
            enabled: true,
            request_calls: 0,
            requested_bytes: 0,
        });
        assert!(!previous.enabled, "allocation windows must not nest");
    });
}

fn finish_allocation_window() -> (u64, u64) {
    ALLOCATION_WINDOW.with(|window| {
        let measured = window.replace(AllocationWindow::default());
        assert!(measured.enabled, "allocation window should be active");
        (measured.request_calls, measured.requested_bytes)
    })
}

fn record_allocation_request(size: usize) {
    let _ = ALLOCATION_WINDOW.try_with(|window| {
        let current = window.get();
        if current.enabled {
            window.set(AllocationWindow {
                request_calls: current.request_calls.saturating_add(1),
                requested_bytes: current.requested_bytes.saturating_add(size as u64),
                ..current
            });
        }
    });
}
