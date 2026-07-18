use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use kira::{
    backend::mock::{MockBackend, MockBackendSettings},
    AudioManagerSettings,
};
use zircon_runtime::core::framework::sound::{
    SoundMixerGraph, SoundMixerGraphManager, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

use crate::kira_bridge::{
    compile_graph, diff_graphs, graph_compile_invocations, reset_graph_compile_invocations,
    GraphSyncAction, KiraEngine,
};
use crate::service_types::{last_graph_commit_lock_hold_for_test, ActiveGraphCommitHarness};
use crate::DefaultSoundManager;

struct CountingAllocator;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_CALLS: Cell<usize> = const { Cell::new(0) };
}

fn count_allocation() {
    COUNT_ALLOCATIONS.with(|enabled| {
        if enabled.get() {
            ALLOCATION_CALLS.with(|calls| calls.set(calls.get().saturating_add(1)));
        }
    });
}

fn begin_allocation_count() {
    ALLOCATION_CALLS.with(|calls| calls.set(0));
    COUNT_ALLOCATIONS.with(|enabled| enabled.set(true));
}

fn finish_allocation_count() -> usize {
    COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
    ALLOCATION_CALLS.with(Cell::get)
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count_allocation();
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static TEST_ALLOCATOR: CountingAllocator = CountingAllocator;

fn graph_with_track(track: u64) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph.tracks.push(SoundTrackDescriptor::child(
        SoundTrackId::new(track),
        format!("Track {track}"),
    ));
    graph
}

fn mock_settings() -> AudioManagerSettings<MockBackend> {
    AudioManagerSettings {
        backend_settings: MockBackendSettings {
            sample_rate: 48_000,
        },
        ..AudioManagerSettings::default()
    }
}

#[test]
fn diff_graphs_compiles_next_graph_once() {
    let before = graph_with_track(2);
    let mut after = before.clone();
    after
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(3), "SFX"));

    reset_graph_compile_invocations();
    diff_graphs(&before, &after).unwrap();

    assert_eq!(graph_compile_invocations(), 1);
}

#[test]
fn active_sync_graph_compiles_next_graph_once() {
    let before = graph_with_track(2);
    let mut after = before.clone();
    after
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(3), "SFX"));
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings()).unwrap();
    engine.sync_graph(&before).unwrap();

    reset_graph_compile_invocations();
    engine.sync_graph(&after).unwrap();

    assert_eq!(graph_compile_invocations(), 1);
}

#[test]
fn active_nested_parent_rebuild_uses_staging_capacity() {
    let parent = SoundTrackId::new(2);
    let mut before = SoundMixerGraph::default_stereo(48_000);
    before
        .tracks
        .push(SoundTrackDescriptor::child(parent, "Parent"));
    for id in 3..=5 {
        let mut child = SoundTrackDescriptor::child(SoundTrackId::new(id), format!("Child {id}"));
        child.parent = Some(parent);
        before.tracks.push(child);
    }
    let mut after = before.clone();
    for child in &mut after.tracks[2..] {
        child.sends.push(SoundTrackSend {
            target: SoundTrackId::master(),
            gain: 0.5,
            pre_effects: false,
        });
    }
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate_with_limits(mock_settings(), before.tracks.len(), 1)
        .unwrap();
    engine.sync_graph(&before).unwrap();

    engine.sync_graph(&after).unwrap();

    assert_eq!(engine.installed_graph_for_test(), Some(&after));
}

#[test]
fn active_parent_growth_rebuilds_with_proportional_child_capacity() {
    let parent = SoundTrackId::new(2);
    let mut before = SoundMixerGraph::default_stereo(48_000);
    before
        .tracks
        .push(SoundTrackDescriptor::child(parent, "Parent"));
    let mut after = before.clone();
    for id in 3..=10 {
        let mut child = SoundTrackDescriptor::child(SoundTrackId::new(id), format!("Child {id}"));
        child.parent = Some(parent);
        after.tracks.push(child);
    }
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate_with_limits(mock_settings(), after.tracks.len(), 1)
        .unwrap();
    engine.sync_graph(&before).unwrap();

    engine.sync_graph(&after).unwrap();

    assert_eq!(engine.installed_graph_for_test(), Some(&after));
}

#[test]
fn compiled_non_master_child_capacity_is_linear_in_track_count() {
    let graph = graph_with_tracks(1_000);

    let compiled = compile_graph(&graph).unwrap();
    let reserved_children = compiled
        .tracks()
        .iter()
        .filter(|track| track.id != SoundTrackId::master())
        .map(|track| track.child_capacity)
        .sum::<usize>();

    assert!(reserved_children <= graph.tracks.len().saturating_mul(3));
}

#[test]
fn send_bus_inherits_target_and_parent_gain_without_bypassing_the_bus_chain() {
    let mut graph = graph_with_tracks(3);
    let bus = graph.tracks[3].id;
    graph.tracks[2].parent = Some(bus);
    graph.tracks[2].controls.gain = 0.5;
    graph.tracks[3].controls.gain = 0.5;
    let target = graph.tracks[2].id;
    graph.tracks[1].sends.push(SoundTrackSend {
        target,
        gain: 1.0,
        pre_effects: false,
    });

    let compiled = compile_graph(&graph).unwrap();

    assert!(
        (compiled
            .send_target_linear_gain(graph.tracks[2].id)
            .unwrap()
            - 0.25)
            .abs()
            < 1.0e-6
    );
}

#[test]
fn target_or_parent_gain_change_tweens_the_existing_send_bus() {
    let mut before = graph_with_tracks(3);
    let bus = before.tracks[3].id;
    before.tracks[2].parent = Some(bus);
    let target = before.tracks[2].id;
    before.tracks[1].sends.push(SoundTrackSend {
        target,
        gain: 1.0,
        pre_effects: false,
    });
    let mut after = before.clone();
    after.tracks[3].controls.gain = 0.5;

    let diff = diff_graphs(&before, &after).unwrap();

    assert!(diff.actions().iter().any(|action| matches!(
        action,
        GraphSyncAction::SetSendVolume {
            target,
            linear_gain,
            ..
        } if *target == after.tracks[2].id && (*linear_gain - 0.5).abs() < 1.0e-6
    )));
}

#[test]
fn inactive_public_preset_mutation_skips_m1_kira_compile_and_records_lock_hold() {
    let manager = DefaultSoundManager::default();
    reset_graph_compile_invocations();

    manager
        .apply_mixer_preset("sound://mixer/spatial_room")
        .unwrap();

    assert_eq!(graph_compile_invocations(), 0);
    let lock_hold = last_graph_commit_lock_hold_for_test()
        .expect("public graph mutation must record its state-lock hold duration");
    assert!(
        lock_hold < Duration::from_millis(100),
        "preset graph commit held state lock for {lock_hold:?}"
    );
}

#[test]
fn concurrent_track_updates_rebase_without_lost_updates() {
    let manager = Arc::new(DefaultSoundManager::default());
    let barrier = Arc::new(Barrier::new(3));
    let workers = [2_u64, 3].map(|id| {
        let manager = Arc::clone(&manager);
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            barrier.wait();
            manager
                .add_or_update_track(SoundTrackDescriptor::child(
                    SoundTrackId::new(id),
                    format!("Track {id}"),
                ))
                .unwrap();
        })
    });
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }

    let snapshot = manager.mixer_snapshot().unwrap();
    assert!(snapshot
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::new(2)));
    assert!(snapshot
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::new(3)));
}

#[derive(Clone, Copy)]
enum Mutation {
    Add,
    Update,
    Remove,
    Send,
}

impl Mutation {
    const ALL: [Self; 4] = [Self::Add, Self::Update, Self::Remove, Self::Send];

    const fn label(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Update => "update",
            Self::Remove => "remove",
            Self::Send => "send",
        }
    }
}

#[test]
fn graph_mutation_benchmark_records_scale_allocations_and_lock_hold_time() {
    for track_count in [10_usize, 100, 1_000] {
        let before = graph_with_tracks(track_count);
        for mutation in Mutation::ALL {
            let after = mutated_graph(&before, mutation);
            let (p50, p95, allocations_p50, allocations_p95) = benchmark_diff(&before, &after, 32);
            eprintln!(
                "sound_graph_diff tracks={track_count} mutation={} p50_us={} p95_us={} allocations_p50={} allocations_p95={}",
                mutation.label(),
                p50.as_micros(),
                p95.as_micros(),
                allocations_p50,
                allocations_p95,
            );
            assert!(
                p95 < Duration::from_millis(100),
                "{track_count}-track {} diff exceeded 100ms p95",
                mutation.label(),
            );
            assert!(
                allocations_p95 <= track_count.saturating_mul(3).saturating_add(96),
                "{track_count}-track {} diff exceeded the bounded scale allocation budget",
                mutation.label(),
            );

            let (lock_p50, lock_p95) =
                benchmark_active_public_manager_lock_hold(&before, mutation, 32);
            eprintln!(
                "sound_graph_active_public_lock tracks={track_count} mutation={} p50_us={} p95_us={}",
                mutation.label(),
                lock_p50.as_micros(),
                lock_p95.as_micros(),
            );
            let lock_budget = active_public_lock_budget(track_count);
            assert!(
                lock_p95 < lock_budget,
                "{track_count}-track {} active public state-lock hold exceeded the {:?} linear budget",
                mutation.label(),
                lock_budget,
            );

            let (p50, p95, allocations_p50, allocations_p95) =
                benchmark_active_kira_mutation(&before, mutation, 32);
            eprintln!(
                "sound_graph_active_kira tracks={track_count} mutation={} p50_us={} p95_us={} allocations_p50={} allocations_p95={}",
                mutation.label(),
                p50.as_micros(),
                p95.as_micros(),
                allocations_p50,
                allocations_p95,
            );
            assert!(
                p95 < Duration::from_millis(250),
                "{track_count}-track {} active Kira mutation exceeded 250ms",
                mutation.label(),
            );
            assert!(
                allocations_p95 <= track_count.saturating_mul(4).saturating_add(256),
                "{track_count}-track {} active Kira mutation exceeded the bounded allocation budget",
                mutation.label(),
            );
        }
    }
}

fn graph_with_tracks(track_count: usize) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph.tracks.extend((0..track_count).map(|index| {
        SoundTrackDescriptor::child(
            SoundTrackId::new(index as u64 + 2),
            format!("Track {index}"),
        )
    }));
    graph
}

fn mutated_graph(before: &SoundMixerGraph, mutation: Mutation) -> SoundMixerGraph {
    let mut after = before.clone();
    match mutation {
        Mutation::Add => after.tracks.push(SoundTrackDescriptor::child(
            SoundTrackId::new(after.tracks.len() as u64 + 2),
            "Added",
        )),
        Mutation::Update => after.tracks[1].controls.gain = 0.5,
        Mutation::Remove => {
            after.tracks.pop();
        }
        Mutation::Send => after.tracks[1].sends.push(SoundTrackSend {
            target: SoundTrackId::master(),
            gain: 0.5,
            pre_effects: false,
        }),
    }
    after
}

fn benchmark_diff(
    before: &SoundMixerGraph,
    after: &SoundMixerGraph,
    samples: usize,
) -> (Duration, Duration, usize, usize) {
    let mut durations = Vec::with_capacity(samples);
    let mut allocations = Vec::with_capacity(samples);
    for _ in 0..samples {
        reset_graph_compile_invocations();
        begin_allocation_count();
        let started = Instant::now();
        let diff = black_box(diff_graphs(black_box(before), black_box(after)).unwrap());
        durations.push(started.elapsed());
        allocations.push(finish_allocation_count());
        assert_eq!(graph_compile_invocations(), 1);
        black_box(diff);
    }
    percentile_pair(durations, allocations)
}

fn benchmark_active_kira_mutation(
    graph: &SoundMixerGraph,
    mutation: Mutation,
    samples: usize,
) -> (Duration, Duration, usize, usize) {
    let mut durations = Vec::with_capacity(samples);
    let mut allocations = Vec::with_capacity(samples);
    let after = mutated_graph(graph, mutation);
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate_with_limits(
            mock_settings(),
            graph.tracks.len().max(after.tracks.len()),
            1,
        )
        .unwrap();
    engine.sync_graph(graph).unwrap();
    for _ in 0..samples {
        begin_allocation_count();
        let started = Instant::now();
        engine.sync_graph(black_box(&after)).unwrap();
        durations.push(started.elapsed());
        allocations.push(finish_allocation_count());
        engine
            .with_backend_mut(|backend| backend.on_start_processing())
            .unwrap();
        engine.sync_graph(graph).unwrap();
        engine
            .with_backend_mut(|backend| backend.on_start_processing())
            .unwrap();
    }
    percentile_pair(durations, allocations)
}

fn benchmark_active_public_manager_lock_hold(
    graph: &SoundMixerGraph,
    mutation: Mutation,
    samples: usize,
) -> (Duration, Duration) {
    let after = mutated_graph(graph, mutation);
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate_with_limits(
            mock_settings(),
            graph.tracks.len().max(after.tracks.len()),
            1,
        )
        .unwrap();
    engine.sync_graph(graph).unwrap();
    let manager = ActiveGraphCommitHarness::new(engine, graph.clone());
    let mut durations = Vec::with_capacity(samples);
    for _ in 0..samples {
        durations.push(manager.replace_graph(after.clone()).unwrap());
        manager
            .with_kira_mut(|engine| {
                engine.with_backend_mut(|backend| backend.on_start_processing())
            })
            .unwrap();
        manager.replace_graph(graph.clone()).unwrap();
        manager
            .with_kira_mut(|engine| {
                engine.with_backend_mut(|backend| backend.on_start_processing())
            })
            .unwrap();
    }
    duration_percentile_pair(durations)
}

fn active_public_lock_budget(track_count: usize) -> Duration {
    Duration::from_micros(5_000_u64.saturating_add((track_count as u64).saturating_mul(250)))
}

fn duration_percentile_pair(mut durations: Vec<Duration>) -> (Duration, Duration) {
    durations.sort_unstable();
    let p50 = durations.len() / 2;
    let p95 = (durations.len() * 95 / 100).min(durations.len() - 1);
    (durations[p50], durations[p95])
}

fn percentile_pair(
    mut durations: Vec<Duration>,
    mut allocations: Vec<usize>,
) -> (Duration, Duration, usize, usize) {
    durations.sort_unstable();
    allocations.sort_unstable();
    let p50 = durations.len() / 2;
    let p95 = (durations.len() * 95 / 100).min(durations.len() - 1);
    (
        durations[p50],
        durations[p95],
        allocations[p50],
        allocations[p95],
    )
}
