use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::runtime::{CoreHandle, CoreRuntime};
use zircon_runtime::scene::components::{ActiveInHierarchy, NodeKind, NodeRecord, WorldMatrix};
use zircon_runtime::scene::{self, LevelSystem, World};

const CHAIN_DEPTHS: [usize; 3] = [1, 32, 1_024];
const SAMPLE_COUNT: usize = 31;
const QUERIES_PER_SAMPLE: usize = 256;
const FIRST_ENTITY: u64 = 4_000_000;

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
fn runtime62_dirty_projected_reads_match_flushed_values_after_ancestor_changes() {
    warm_allocation_window();

    for depth in CHAIN_DEPTHS {
        let (runtime, core, level, root, leaf) = level_with_committed_chain(depth);
        let projected_matrix = dirty_world_matrix_read(&level, root, leaf, depth);
        assert_zero_allocations(
            measure_read_batch(&level, leaf, ProjectedRead::WorldMatrix, 1),
            depth,
            ProjectedRead::WorldMatrix,
        );
        assert_flushed_world_matrix(&runtime, &core, &level, leaf, projected_matrix);

        let projected_active = dirty_active_read(&level, root, leaf);
        assert_zero_allocations(
            measure_read_batch(&level, leaf, ProjectedRead::ActiveInHierarchy, 1),
            depth,
            ProjectedRead::ActiveInHierarchy,
        );
        assert_flushed_active(&runtime, &core, &level, leaf, projected_active);
    }
}

#[test]
#[ignore = "managed Windows Release Runtime62 G19 dirty compatibility-path samples"]
fn runtime62_dirty_projected_read_depth_compatibility_profile() {
    assert!(
        cfg!(target_os = "windows"),
        "run this ignored profile on Windows"
    );
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );
    warm_allocation_window();

    for depth in CHAIN_DEPTHS {
        let (runtime, core, level, root, leaf) = level_with_committed_chain(depth);

        let expected_matrix = dirty_world_matrix_read(&level, root, leaf, depth);
        let matrix_samples = (0..SAMPLE_COUNT)
            .map(|_| {
                measure_read_batch(&level, leaf, ProjectedRead::WorldMatrix, QUERIES_PER_SAMPLE)
            })
            .collect::<Vec<_>>();
        for sample in &matrix_samples {
            assert_zero_allocations(*sample, depth, ProjectedRead::WorldMatrix);
        }
        report_samples(depth, ProjectedRead::WorldMatrix, &matrix_samples);
        assert_flushed_world_matrix(&runtime, &core, &level, leaf, expected_matrix);

        let expected_active = dirty_active_read(&level, root, leaf);
        let active_samples = (0..SAMPLE_COUNT)
            .map(|_| {
                measure_read_batch(
                    &level,
                    leaf,
                    ProjectedRead::ActiveInHierarchy,
                    QUERIES_PER_SAMPLE,
                )
            })
            .collect::<Vec<_>>();
        for sample in &active_samples {
            assert_zero_allocations(*sample, depth, ProjectedRead::ActiveInHierarchy);
        }
        report_samples(depth, ProjectedRead::ActiveInHierarchy, &active_samples);
        assert_flushed_active(&runtime, &core, &level, leaf, expected_active);
    }
}

fn level_with_committed_chain(depth: usize) -> (CoreRuntime, CoreHandle, LevelSystem, u64, u64) {
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
    let (world, root, leaf) = chain_world(depth);
    level.replace_world_and_reset_runtime_state(world);
    level
        .tick(&core, runtime.tick_time(4))
        .expect("initial derived-state publication should complete");
    level
        .tick(&core, runtime.tick_time(4))
        .expect("stable follow-up tick should complete");

    (runtime, core, level, root, leaf)
}

fn chain_world(depth: usize) -> (World, u64, u64) {
    assert!(depth > 0, "chain depth must be nonzero");
    let mut template_world = World::empty();
    let template_entity = template_world
        .spawn_node(NodeKind::Empty)
        .expect("dirty-read profile template should spawn");
    let template = template_world
        .node_record(template_entity)
        .expect("dirty-read profile template should have a node record");

    let mut records = Vec::with_capacity(depth);
    for index in 0..depth {
        let mut record: NodeRecord = template.clone();
        record.id = FIRST_ENTITY + index as u64;
        record.name = format!("Runtime62 dirty projected read {index}");
        record.parent = index
            .checked_sub(1)
            .map(|parent_index| FIRST_ENTITY + parent_index as u64);
        record.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        record.active = true;
        records.push(record);
    }
    let root = records.first().expect("chain depth is nonzero").id;
    let leaf = records.last().expect("chain depth is nonzero").id;

    let mut world = World::empty();
    world
        .insert_owned_node_records(records)
        .expect("dirty-read profile chain should publish");
    (world, root, leaf)
}

fn dirty_world_matrix_read(
    level: &LevelSystem,
    root: u64,
    leaf: u64,
    depth: usize,
) -> zircon_runtime::core::math::Mat4 {
    let published_before = level.with_world(|world| {
        world
            .get::<WorldMatrix>(leaf)
            .expect("initial tick should publish the leaf matrix")
            .0
    });
    level.with_world_mut(|world| {
        world
            .update_transform(root, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
            .expect("ancestor transform should update");
    });

    level.with_world(|world| {
        assert_eq!(
            world.get::<WorldMatrix>(leaf).map(|matrix| matrix.0),
            Some(published_before),
            "the derived component remains at the previous publication until the tick"
        );
        let projected = world
            .world_matrix(leaf)
            .expect("dirty leaf should project through its ancestor chain");
        assert_ne!(projected, published_before);
        assert_eq!(
            world.world_transform(leaf).unwrap().translation.x,
            depth as f32 + 1.0
        );
        projected
    })
}

fn dirty_active_read(level: &LevelSystem, root: u64, leaf: u64) -> bool {
    let published_before = level.with_world(|world| {
        world
            .get::<ActiveInHierarchy>(leaf)
            .expect("initial tick should publish the leaf active value")
            .0
    });
    assert!(published_before);
    level.with_world_mut(|world| {
        world
            .set_active_self(root, false)
            .expect("ancestor ActiveSelf should update");
    });

    level.with_world(|world| {
        assert_eq!(
            world.get::<ActiveInHierarchy>(leaf).map(|active| active.0),
            Some(published_before),
            "the derived component remains at the previous publication until the tick"
        );
        let projected = world
            .active_in_hierarchy(leaf)
            .expect("dirty leaf should project active state through its ancestors");
        assert!(!projected);
        projected
    })
}

fn assert_flushed_world_matrix(
    runtime: &CoreRuntime,
    core: &CoreHandle,
    level: &LevelSystem,
    leaf: u64,
    expected: zircon_runtime::core::math::Mat4,
) {
    level
        .tick(core, runtime.tick_time(4))
        .expect("world-matrix derived state should publish after the sample");
    level.with_world(|world| {
        assert_eq!(
            world.get::<WorldMatrix>(leaf).map(|matrix| matrix.0),
            Some(expected)
        );
        assert_eq!(world.world_matrix(leaf), Some(expected));
    });
}

fn assert_flushed_active(
    runtime: &CoreRuntime,
    core: &CoreHandle,
    level: &LevelSystem,
    leaf: u64,
    expected: bool,
) {
    level
        .tick(core, runtime.tick_time(4))
        .expect("active derived state should publish after the sample");
    level.with_world(|world| {
        assert_eq!(
            world.get::<ActiveInHierarchy>(leaf).map(|active| active.0),
            Some(expected)
        );
        assert_eq!(world.active_in_hierarchy(leaf), Some(expected));
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
        "dirty {} query at depth {depth} requested heap allocation",
        read.label()
    );
    assert_eq!(
        sample.gross_requested_bytes,
        0,
        "dirty {} query at depth {depth} requested heap bytes",
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
        "RUNTIME62_DIRTY_PROJECTED_READ_DEPTH_COMPAT_PROFILE_V1 shape=chain query={} depth={depth} samples={SAMPLE_COUNT} queries_per_sample={QUERIES_PER_SAMPLE} measured_scope=dirty_public_query_loop allocator_scope=current_thread_tls allocation_bytes_kind=gross_requested_bytes_including_realloc state=ancestor_mutated_before_flush elapsed_ns_per_batch_p50={} elapsed_ns_per_batch_p95={} elapsed_ns_per_batch_p99={} request_calls_p50={} request_calls_p95={} request_calls_p99={} gross_requested_bytes_p50={} gross_requested_bytes_p95={} gross_requested_bytes_p99={} elapsed_ns_raw={elapsed_ns:?} request_calls_raw={request_calls:?} gross_requested_bytes_raw={gross_requested_bytes:?}",
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
