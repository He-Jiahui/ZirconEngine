use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;

use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::runtime::{CoreHandle, CoreRuntime};
use zircon_runtime::scene::ecs::WorldDerivedStateDiagnostics;
use zircon_runtime::scene::{self, LevelSystem, NodeKind, NodeRecord, World};

const STAR_NODE_COUNTS: [usize; 3] = [1_000, 100_000, 1_000_000];
const SAMPLE_COUNT: usize = 31;
const FIRST_ENTITY: u64 = 2_000_000;

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
enum DerivedCounter {
    WorldMatrix,
    ActiveInHierarchy,
}

impl DerivedCounter {
    fn values(self, diagnostics: WorldDerivedStateDiagnostics) -> (u64, u64) {
        match self {
            Self::WorldMatrix => (
                diagnostics.world_matrix_propagation_entities,
                diagnostics.world_matrix_propagation_written_entities,
            ),
            Self::ActiveInHierarchy => (
                diagnostics.active_propagation_entities,
                diagnostics.active_propagation_written_entities,
            ),
        }
    }
}

#[derive(Clone, Copy)]
struct TickSample {
    elapsed_ns: u64,
    allocation_requests: u64,
    gross_requested_bytes: u64,
    visited: u64,
    written: u64,
}

#[test]
fn runtime62_derived_scale_tick_publishes_matrix_and_active_state_for_small_star() {
    let node_count = 1_000;
    let (runtime, core, level) = level_with_star(node_count);

    level.with_world_mut(|world| {
        assert!(world
            .update_transform(
                FIRST_ENTITY,
                Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            )
            .expect("root transform should update"));
    });
    level
        .tick(&core, runtime.tick_time(4))
        .expect("world-matrix tick should complete");
    level.with_world(|world| {
        assert_eq!(
            world
                .world_transform(FIRST_ENTITY + 1)
                .unwrap()
                .translation
                .x,
            3.0
        );
        let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
        let (visited, written) = DerivedCounter::WorldMatrix.values(diagnostics);
        assert_eq!(visited, node_count as u64);
        assert_eq!(written, node_count as u64);
    });

    level.with_world_mut(|world| {
        assert!(world
            .set_active_self(FIRST_ENTITY, false)
            .expect("root active state should update"));
    });
    level
        .tick(&core, runtime.tick_time(4))
        .expect("active-state tick should complete");
    level.with_world(|world| {
        assert_eq!(world.active_in_hierarchy(FIRST_ENTITY + 1), Some(false));
        let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
        let (visited, written) = DerivedCounter::ActiveInHierarchy.values(diagnostics);
        assert_eq!(visited, node_count as u64);
        assert_eq!(written, node_count as u64);
    });
}

#[test]
#[ignore = "managed Windows Release Runtime62 G24 1K/100K/1M star full-tick allocation and latency samples"]
fn runtime62_derived_scale_allocation_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    // Warm the thread-local before opening the measurement window.
    let _ = ALLOCATION_WINDOW.try_with(|window| window.get());

    for node_count in STAR_NODE_COUNTS {
        let (runtime, core, level) = level_with_star(node_count);

        let matrix_samples = collect_matrix_tick_samples(&runtime, &core, &level, node_count);
        report_samples("world_matrix", node_count, &matrix_samples);

        let active_samples = collect_active_tick_samples(&runtime, &core, &level, node_count);
        report_samples("active_in_hierarchy", node_count, &active_samples);
    }
}

fn level_with_star(node_count: usize) -> (CoreRuntime, CoreHandle, LevelSystem) {
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
    level.replace_world_and_reset_runtime_state(star_world(node_count));
    level
        .tick(&core, runtime.tick_time(4))
        .expect("initial derived-state publication should complete");
    (runtime, core, level)
}

fn star_world(node_count: usize) -> World {
    let mut template_world = World::empty();
    let template_entity = template_world
        .spawn_node(NodeKind::Empty)
        .expect("star profile template should spawn");
    let template = template_world
        .node_record(template_entity)
        .expect("star profile template should have a node record");

    let mut records = Vec::with_capacity(node_count);
    for index in 0..node_count {
        let mut record: NodeRecord = template.clone();
        record.id = FIRST_ENTITY + index as u64;
        record.name = format!("Runtime62 derived scale node {index}");
        record.parent = if index == 0 { None } else { Some(FIRST_ENTITY) };
        records.push(record);
    }

    let mut world = World::empty();
    world
        .insert_owned_node_records(records)
        .expect("star profile records should publish");
    world
}

fn collect_matrix_tick_samples(
    runtime: &CoreRuntime,
    core: &CoreHandle,
    level: &LevelSystem,
    node_count: usize,
) -> Vec<TickSample> {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        level.with_world_mut(|world| {
            assert!(world
                .update_transform(
                    FIRST_ENTITY,
                    Transform::from_translation(Vec3::new(sample as f32 + 10.0, 0.0, 0.0)),
                )
                .expect("root transform should change before the tick"));
        });
        let measured = measure_level_tick(runtime, core, level, DerivedCounter::WorldMatrix);
        assert_sample_work(measured, node_count, "world_matrix");
        samples.push(measured);
    }
    samples
}

fn collect_active_tick_samples(
    runtime: &CoreRuntime,
    core: &CoreHandle,
    level: &LevelSystem,
    node_count: usize,
) -> Vec<TickSample> {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let active = sample % 2 == 1;
        level.with_world_mut(|world| {
            assert!(world
                .set_active_self(FIRST_ENTITY, active)
                .expect("root active state should change before the tick"));
        });
        let measured = measure_level_tick(runtime, core, level, DerivedCounter::ActiveInHierarchy);
        assert_sample_work(measured, node_count, "active_in_hierarchy");
        samples.push(measured);
    }
    samples
}

fn measure_level_tick(
    runtime: &CoreRuntime,
    core: &CoreHandle,
    level: &LevelSystem,
    counter: DerivedCounter,
) -> TickSample {
    let snapshot = runtime.tick_time(4);
    begin_allocation_window();
    let started_at = Instant::now();
    let tick_result = level.tick(core, snapshot);
    let elapsed_ns = started_at.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    let (allocation_requests, gross_requested_bytes) = finish_allocation_window();
    tick_result.expect("measured level tick should complete");

    let diagnostics =
        level.with_world(|world| world.ecs_frame_performance_diagnostics().derived_state);
    let (visited, written) = counter.values(diagnostics);
    TickSample {
        elapsed_ns,
        allocation_requests,
        gross_requested_bytes,
        visited,
        written,
    }
}

fn assert_sample_work(sample: TickSample, node_count: usize, operation: &str) {
    assert_eq!(
        sample.visited, node_count as u64,
        "{operation} should visit the full star during every root-change tick"
    );
    assert_eq!(
        sample.written, node_count as u64,
        "{operation} should publish the changed value for every star node"
    );
}

fn report_samples(operation: &str, node_count: usize, samples: &[TickSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let allocation_requests = samples
        .iter()
        .map(|sample| sample.allocation_requests)
        .collect::<Vec<_>>();
    let gross_requested_bytes = samples
        .iter()
        .map(|sample| sample.gross_requested_bytes)
        .collect::<Vec<_>>();
    let visited = samples
        .iter()
        .map(|sample| sample.visited)
        .collect::<Vec<_>>();
    let written = samples
        .iter()
        .map(|sample| sample.written)
        .collect::<Vec<_>>();

    println!(
        "RUNTIME62_DERIVED_SCALE_ALLOCATION_PROFILE_V1 shape=star operation={operation} node_count={node_count} samples={SAMPLE_COUNT} measured_scope=public_level_tick elapsed_kind=instrumented_tick_envelope allocator_scope=current_thread_tls allocation_bytes_kind=gross_requested_bytes_including_realloc elapsed_ns_p50={} elapsed_ns_p95={} elapsed_ns_p99={} allocation_requests_p50={} allocation_requests_p95={} allocation_requests_p99={} gross_requested_bytes_p50={} gross_requested_bytes_p95={} gross_requested_bytes_p99={} elapsed_ns_raw={elapsed_ns:?} allocation_requests_raw={allocation_requests:?} gross_requested_bytes_raw={gross_requested_bytes:?} visited_raw={visited:?} written_raw={written:?}",
        nearest_rank_percentile(&elapsed_ns, 50),
        nearest_rank_percentile(&elapsed_ns, 95),
        nearest_rank_percentile(&elapsed_ns, 99),
        nearest_rank_percentile(&allocation_requests, 50),
        nearest_rank_percentile(&allocation_requests, 95),
        nearest_rank_percentile(&allocation_requests, 99),
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
