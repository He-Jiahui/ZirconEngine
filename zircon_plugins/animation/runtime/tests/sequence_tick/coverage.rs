//! Independent disk-backed paused-sequence controls, through real PostUpdate.

use std::fs;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::pipeline::manager::ProjectAssetGenerationToken;
use zircon_runtime::asset::AssetManager;
use zircon_runtime::core::framework::animation::{
    AnimationChannelValueAsset, AnimationInterpolationAsset,
};
use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
use zircon_runtime::core::resource::{
    ResourceActiveVersionKey, ResourceScheme, ResourceState, RuntimeResourceState,
};
use zircon_runtime::scene::ecs::{
    ChangeTick, ComponentLifecycleEvent, LifecycleEventKind, Resource,
};
use zircon_runtime::scene::world::{CompiledScenePropertyAccessStats, SceneResult};

use super::super::{
    checked_fixture_path, edit_mixed_translation, sequence_asset_for_entity,
    AnimationSequenceAsset, AnimationSequenceMarker, AssetUri, EntityId, LocalTransform,
    PlayerStamp, ProjectPaths, ReadySequence, Real, ResourceHandle, ResourceKind, ResourceRecord,
    SequenceFrames, Vec3, World, PAUSED_TIME, SAMPLED, SENTINEL,
};
use super::{assert_fresh_project, authored_sequence, write_project, CloseOnUnwind, BOUND, LATE};

fn new_project_paths(child: &str) -> ProjectPaths {
    let env_path = |name| PathBuf::from(std::env::var_os(name).expect(name));
    let scratch = checked_fixture_path(&env_path("ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT"), true);
    assert_eq!(scratch, checked_fixture_path(&env_path("TEMP"), true));
    assert_eq!(scratch, checked_fixture_path(&std::env::temp_dir(), true));
    let physical = scratch.operation_path();
    let config = checked_fixture_path(&env_path("ZIRCON_CONFIG_PATH"), false);
    assert!(config != scratch && config.is_within(&scratch));
    assert!(ProjectPaths::same_lexical_path(
        config.operation_path(),
        physical.join(r"home\Config\config.json")
    )
    .unwrap());
    assert_fresh_project(&ProjectPaths::from_root(physical).unwrap());

    // Only the old case claims sequence-residency. Never reuse even an empty child.
    let root = physical.join(child);
    fs::create_dir(&root).expect("qualification: exclusively create a fresh case project");
    let project = checked_fixture_path(&root, true);
    assert!(project != scratch && project.is_within(&scratch));
    ProjectPaths::from_root(project.operation_path()).unwrap()
}

struct DiskSequence<'a> {
    frames: &'a SequenceFrames,
    ready: ReadySequence,
    record: ResourceRecord,
    authored: AnimationSequenceAsset,
    generation: ProjectAssetGenerationToken,
}

fn run_disk<T>(
    child: &str,
    filename: &str,
    authored: AnimationSequenceAsset,
    run: impl FnOnce(&DiskSequence<'_>) -> T,
) -> T {
    let paths = new_project_paths(child);
    let uri = AssetUri::parse(&format!("res://animation/{filename}.sequence.zranim")).unwrap();
    let encoded = authored
        .to_bytes()
        .expect("qualification: encode disk sequence");
    let source = write_project(&paths, &uri, &encoded);
    let frames = SequenceFrames::open();
    let unwind = CloseOnUnwind(&frames);
    let assets = frames.assets();
    let project = AssetManager::open_project(&*assets, paths.root().to_str().unwrap())
        .expect("qualification: open and import the fresh project");
    assert_eq!(Path::new(&project.root_path), paths.root());
    let generation = assets
        .current_project_generation_snapshot()
        .unwrap()
        .into_parts()
        .1;
    assert_eq!(generation.project_root(), paths.root());
    let indexed = AssetManager::current_project_source_path(&*assets, &uri)
        .unwrap()
        .unwrap();
    assert_eq!(
        checked_fixture_path(&indexed, false).operation_path(),
        ProjectPaths::resolve_existing_path(&source).unwrap()
    );
    assert_eq!(fs::read(&source).unwrap(), encoded);
    let id = assets
        .resolve_asset_id(&uri)
        .expect("qualification: normal imported ID");
    let resources = assets.resource_manager();
    let record = resources
        .registry()
        .get(id)
        .cloned()
        .expect("qualification: imported record");
    assert_eq!(
        (record.kind, record.state),
        (ResourceKind::AnimationSequence, ResourceState::Ready)
    );
    assert_eq!(record.primary_locator, uri);
    assert!(!record.source_hash.is_empty());
    let artifact = record
        .artifact_locator()
        .expect("qualification: committed artifact");
    assert_eq!(artifact.scheme(), ResourceScheme::Library);
    let artifact_path = paths.asset_artifact_root().join(artifact.path());
    let artifact_file = checked_fixture_path(&artifact_path, false);
    let artifact_bytes = fs::read(artifact_file.operation_path()).unwrap();
    assert!(!artifact_bytes.is_empty());
    println!("SEQUENCE_COVERAGE_V1 case={child} phase=opened generation={generation:?} id={id} revision={} source_hash={}", record.revision, record.source_hash);
    drop(assets);

    let result = run(&DiskSequence {
        frames: &frames,
        ready: ReadySequence {
            id,
            revision: record.revision,
        },
        record,
        authored,
        generation,
    });

    let same_input_bytes = fs::read(&source).unwrap() == encoded
        && fs::read(artifact_file.operation_path()).unwrap() == artifact_bytes;
    let assets = frames.assets();
    let closed = AssetManager::close_project(&*assets).expect("teardown: close owned project");
    assert_eq!(closed.as_deref(), Some(paths.root()));
    assert!(assets.current_project_manager().is_none() && resources.registry().get(id).is_none());
    drop(assets);
    drop(unwind);
    println!("SEQUENCE_COVERAGE_V1 case={child} phase=closed same_input_bytes={same_input_bytes}");
    assert!(
        same_input_bytes,
        "qualification: same source and artifact manifest bytes"
    );
    result
}

#[derive(Debug)]
struct AssetPhase {
    same_generation: bool,
    same_record: bool,
    same_id: bool,
    state: Option<RuntimeResourceState>,
    leases: Option<usize>,
    key: Option<ResourceActiveVersionKey>,
    has_payload: bool,
    typed_pair: bool,
}

impl AssetPhase {
    fn assert_ready(&self, stage: &str) {
        assert!(
            self.same_generation && self.same_record && self.same_id,
            "qualification {stage}: same project generation and Ready source: {self:?}"
        );
    }

    fn assert_cold(&self, stage: &str) {
        self.assert_ready(stage);
        assert_eq!(
            (
                self.state,
                self.leases,
                self.key.is_some(),
                self.has_payload,
                self.typed_pair
            ),
            (
                Some(RuntimeResourceState::Unloaded),
                Some(0),
                false,
                false,
                false
            ),
            "qualification {stage}: cold Ready, no active key or payload"
        );
    }

    fn installed(&self, stage: &str) -> &ResourceActiveVersionKey {
        self.assert_ready(stage);
        assert_eq!(
            (self.state, self.leases, self.has_payload, self.typed_pair),
            (Some(RuntimeResourceState::Loaded), Some(0), true, true),
            "first ordinary PostUpdate must install the catalog-matched typed payload: {stage}"
        );
        self.key
            .as_ref()
            .expect("the installed payload has an active version key")
    }
}

impl DiskSequence<'_> {
    // Observe only. In particular, never call load/acquire to check that the first frame loaded.
    fn asset_phase(&self, stage: &str) -> AssetPhase {
        let assets = self.frames.assets();
        let resources = assets.resource_manager();
        let id = self.ready.id;
        let token = assets
            .current_project_generation_snapshot()
            .map(|snapshot| snapshot.into_parts().1);
        let key = resources.projection_snapshot().active().key(id);
        let typed_pair = key.as_ref().is_some_and(|key| {
            resources
                .snapshot_expected::<AnimationSequenceMarker, AnimationSequenceAsset>(
                    ResourceHandle::new(id),
                    key,
                )
                .is_ok_and(|paired| (paired.record(), &*paired) == (&self.record, &self.authored))
        });
        // Drop the registry guard before acquiring any other asset/resource lock.
        let same_record = resources.registry().get(id) == Some(&self.record);
        let same_id = assets.resolve_asset_id(&self.record.primary_locator) == Some(id);
        let phase = AssetPhase {
            same_generation: token.as_ref() == Some(&self.generation),
            same_record,
            same_id,
            state: resources.runtime_state(id),
            leases: resources.ref_count(id),
            key,
            has_payload: resources.get_untyped(id).is_some(),
            typed_pair,
        };
        println!("SEQUENCE_COVERAGE_V1 phase={stage} id={id} revision={} generation={token:?} asset={phase:?}", self.ready.revision);
        phase
    }
}

fn actor_added<const N: usize>(frames: &SequenceFrames, actors: [EntityId; N]) -> [ChangeTick; N] {
    frames.level.with_world(|world| {
        actors.map(|entity| {
            world
                .component_change_ticks::<LocalTransform>(entity)
                .unwrap()
                .added()
        })
    })
}

#[derive(Debug)]
struct FramePhase<const N: usize, const P: usize> {
    stage: &'static str,
    requests: u64,
    paths: [Option<EntityId>; P],
    values: [Option<Vec3>; N],
    added: [Option<ChangeTick>; N],
    stamp: PlayerStamp,
    continuous: bool,
    asset: AssetPhase,
}

impl<const N: usize, const P: usize> FramePhase<N, P> {
    fn assert_instance(
        &self,
        paused: &PlayerStamp,
        added: [ChangeTick; N],
        key: &ResourceActiveVersionKey,
    ) {
        assert_eq!(paused.0.time_seconds, PAUSED_TIME);
        assert!(!paused.0.playing);
        assert_eq!(
            &self.stamp, paused,
            "paused player and change tick: {}",
            self.stage
        );
        assert_eq!(
            self.added,
            added.map(Some),
            "same actor component lifetimes: {}",
            self.stage
        );
        assert!(
            !self.continuous,
            "paused must not self-schedule: {}",
            self.stage
        );
        let active = self.asset.installed(self.stage);
        println!(
            "SEQUENCE_COVERAGE_V1 phase={} same_active_key={}",
            self.stage,
            active == key
        );
        assert_eq!(active, key, "no re-residency: {}", self.stage);
    }

    fn assert_scene(
        &self,
        paths: [Option<EntityId>; P],
        values: [Vec3; N],
        requests: RangeInclusive<u64>,
    ) {
        assert_eq!(self.paths, paths, "actor paths: {}", self.stage);
        assert_eq!(
            self.values,
            values.map(Some),
            "track writes: {}",
            self.stage
        );
        assert!(
            requests.contains(&self.requests),
            "requests at {}: {} not in {requests:?}",
            self.stage,
            self.requests
        );
    }
}

fn observe<const N: usize, const P: usize>(
    disk: &DiskSequence<'_>,
    stage: &'static str,
    clock: EntityId,
    actors: [EntityId; N],
    paths: [&str; P],
) -> FramePhase<N, P> {
    let frames = disk.frames;
    let requests = frames.frame(stage, 1);
    let (values, added) = frames.level.with_world(|world| {
        (
            actors.map(|entity| {
                world
                    .get::<LocalTransform>(entity)
                    .map(|local| local.transform.translation)
            }),
            actors.map(|entity| {
                world
                    .component_change_ticks::<LocalTransform>(entity)
                    .map(|ticks| ticks.added())
            }),
        )
    });
    let actual = FramePhase {
        stage,
        requests,
        paths: paths.map(|path| frames.at_path(path)),
        values,
        added,
        stamp: frames.player_stamp(clock),
        continuous: frames.level.animation_requires_continuous_frame(),
        asset: disk.asset_phase(stage),
    };
    println!("SEQUENCE_COVERAGE_V1 phase={stage} world_epoch={} clock={clock} actors={actors:?} authored_paths={paths:?} observed={actual:?}", frames.epoch);
    actual
}

#[test]
#[ignore = "requires fresh owned Windows TEMP and ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT"]
fn sequence_tick_disk_cold_paused_installs_then_quiet() {
    // Same authored bytes and res:// URI as the prewarmed residency case, not its directory.
    let (a, paused, added, first, quiet) =
        run_disk("sequence-cold", "lease", authored_sequence(), |disk| {
            let frames = disk.frames;
            let a = frames.spawn(BOUND);
            let clock = frames.player("ColdClock", disk.ready, PAUSED_TIME, false);
            let paused = frames.player_stamp(clock);
            let added = actor_added(frames, [a]);
            assert_eq!(
                (
                    frames.at_path(BOUND),
                    frames.at_path(LATE),
                    frames.translation(a)
                ),
                (Some(a), None, Vec3::ZERO)
            );
            disk.asset_phase("cold_before").assert_cold("cold_before");

            let first = observe(disk, "cold_first", clock, [a], [BOUND, LATE]);
            edit_mixed_translation(frames, a, SENTINEL);
            let quiet = observe(disk, "cold_quiet", clock, [a], [BOUND, LATE]);
            (a, paused, added, first, quiet)
        });
    let key = first.asset.installed(first.stage);
    for frame in [&first, &quiet] {
        frame.assert_instance(&paused, added, key);
    }
    first.assert_scene([Some(a), None], [SAMPLED], 1..=1);
    quiet.assert_scene([Some(a), None], [SENTINEL], 0..=0);
}

fn path_only_sequence(a: &str, later: &[(&str, [Real; 3])]) -> AnimationSequenceAsset {
    let mut authored = sequence_asset_for_entity(a);
    authored.bindings[0].target_id = None;
    for &(path, end) in later {
        let mut binding = sequence_asset_for_entity(path).bindings.remove(0);
        binding.target_id = None;
        let channel = &mut binding.tracks[0].channel;
        channel.interpolation = AnimationInterpolationAsset::Linear;
        channel.keys[0].out_tangent = None;
        channel.keys[1].in_tangent = None;
        channel.keys[1].time_seconds = 1.0;
        channel.keys[1].value = AnimationChannelValueAsset::Vec3(end);
        authored.bindings.push(binding);
    }
    authored
}

const B_SAMPLE: Vec3 = Vec3::new(0.0, 2.0, 0.0);
const C_SAMPLE: Vec3 = Vec3::new(0.0, 0.0, 3.0);
const B_EDIT: Vec3 = Vec3::new(0.0, 29.0, 0.0);

#[test]
#[ignore = "requires fresh owned Windows TEMP and ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT"]
fn sequence_tick_paused_omits_previously_written_actor_after_rename() {
    const PATHS: [&str; 4] = ["HistoryA", "HistoryAway", "HistoryB", "HistoryC"];
    let [path_a, away, path_b, path_c] = PATHS;
    let authored = path_only_sequence(
        path_a,
        &[(path_b, [0.0, 4.0, 0.0]), (path_c, [0.0, 0.0, 6.0])],
    );
    let ([a, b, c], paused, added, [first, absent, admitted_b, returned, admitted_c, quiet]) =
        run_disk("sequence-history", "history", authored, |disk| {
            let frames = disk.frames;
            let a = frames.spawn(path_a);
            let b = frames.spawn("HistoryBWaiting");
            let c = frames.spawn("HistoryCWaiting");
            let actors = [a, b, c];
            assert!(a != b && a != c && b != c);
            assert_eq!(
                PATHS.map(|path| frames.at_path(path)),
                [Some(a), None, None, None]
            );
            assert_eq!(
                actors.map(|entity| frames.translation(entity)),
                [Vec3::ZERO; 3]
            );
            let added = actor_added(frames, actors);
            let clock = frames.player("HistoryClock", disk.ready, PAUSED_TIME, false);
            let paused = frames.player_stamp(clock);
            let step = |stage| observe(disk, stage, clock, actors, PATHS);
            disk.asset_phase("history_before")
                .assert_cold("history_before");

            // Capture every actual frame before evaluating the behavioral assertions.
            let first = step("history_first");
            edit_mixed_translation(frames, a, SENTINEL);
            frames.rename(a, away);
            let absent = step("history_a_away");
            frames.rename(b, path_b);
            let admitted_b = step("history_b_enters");
            edit_mixed_translation(frames, b, B_EDIT);
            frames.rename(a, path_a);
            let returned = step("history_a_returns");
            frames.rename(c, path_c);
            let admitted_c = step("history_c_enters");
            let quiet = step("history_quiet");
            (
                actors,
                paused,
                added,
                [first, absent, admitted_b, returned, admitted_c, quiet],
            )
        });

    let key = first.asset.installed(first.stage);
    for frame in [&first, &absent, &admitted_b, &returned, &admitted_c, &quiet] {
        frame.assert_instance(&paused, added, key);
    }
    first.assert_scene(
        [Some(a), None, None, None],
        [SAMPLED, Vec3::ZERO, Vec3::ZERO],
        1..=1,
    );
    absent.assert_scene(
        [None, Some(a), None, None],
        [SENTINEL, Vec3::ZERO, Vec3::ZERO],
        0..=1,
    );
    admitted_b.assert_scene(
        [None, Some(a), Some(b), None],
        [SENTINEL, B_SAMPLE, Vec3::ZERO],
        1..=1,
    );
    returned.assert_scene(
        [Some(a), None, Some(b), None],
        [SENTINEL, B_EDIT, Vec3::ZERO],
        0..=1,
    );
    let paths = [Some(a), None, Some(b), Some(c)];
    admitted_c.assert_scene(paths, [SENTINEL, B_EDIT, C_SAMPLE], 1..=1);
    quiet.assert_scene(paths, [SENTINEL, B_EDIT, C_SAMPLE], 0..=0);
}

const STALE_A: &str = "StaleA";
const STALE_B: &str = "StaleB";
const STALE_AWAY: &str = "StaleBAway";

#[derive(Debug, Default)]
struct RenameWitness {
    replaces: [u64; 2],
    at_a: Option<(CompiledScenePropertyAccessStats, [Vec3; 2])>,
    renamed: Option<SceneResult<bool>>,
}

impl Resource for RenameWitness {}

fn rename_on_a(ids: &[EntityId; 2], world: &mut World, event: &ComponentLifecycleEvent) {
    let Some(index) = ids.iter().position(|id| *id == event.entity()) else {
        return;
    };
    let witness = world.resource_mut::<RenameWitness>();
    witness.replaces[index] += 1;
    if index != 0 || witness.replaces[0] != 1 {
        return;
    }
    let values = ids.map(|id| {
        world
            .get::<LocalTransform>(id)
            .unwrap()
            .transform
            .translation
    });
    let stats = world.compiled_scene_property_access_stats();
    // Already in the Level's exclusive World loan. Never call back into Level or services.
    let renamed = world.rename_node(ids[1], STALE_AWAY);
    let witness = world.resource_mut::<RenameWitness>();
    witness.at_a = Some((stats, values));
    witness.renamed = Some(renamed);
}

#[test]
#[ignore = "requires fresh owned Windows TEMP and ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT"]
fn sequence_tick_ordinary_stale_writer_retries_that_track_only() {
    const PATHS: [&str; 3] = [STALE_A, STALE_B, STALE_AWAY];
    let authored = path_only_sequence(STALE_A, &[(STALE_B, [0.0, 4.0, 0.0])]);
    let ([a, b], paused, added, first, recovered, quiet) = run_disk(
        "sequence-stale",
        "stale",
        authored,
        |disk| {
            let frames = disk.frames;
            let a = frames.spawn(STALE_A);
            let b = frames.spawn(STALE_B);
            let actors = [a, b];
            assert_ne!(a, b);
            assert_eq!(
                PATHS.map(|path| frames.at_path(path)),
                [Some(a), Some(b), None]
            );
            assert_eq!(
                actors.map(|entity| frames.translation(entity)),
                [Vec3::ZERO; 2]
            );
            let added = actor_added(frames, actors);
            let clock = frames.player("StaleClock", disk.ready, PAUSED_TIME, false);
            let paused = frames.player_stamp(clock);
            let step = |stage| observe(disk, stage, clock, actors, PATHS);
            disk.asset_phase("stale_before").assert_cold("stale_before");

            let (observer, baseline, before_generations) = frames.level.with_world_mut(|world| {
                assert_eq!(actors.map(|actor| world.parent_of(actor)), [None, None]);
                world.insert_resource(RenameWitness::default());
                let observer = world.observe_component_lifecycle::<LocalTransform>(
                    LifecycleEventKind::Replace,
                    RetainedLifecycleObserverV1::new(
                        actors,
                        // A first Replace can retain at most the one fresh B Name string.
                        |_, _, meter| meter.array::<u8>(STALE_AWAY.len()),
                        rename_on_a,
                    ),
                );
                (
                    observer,
                    world.compiled_scene_property_access_stats(),
                    actors.map(|actor| world.scene_binding_generation(actor)),
                )
            });
            let first = step("stale_ordinary");
            let (after, after_generations, witness) = frames.level.with_world_mut(|world| {
                world.remove_observer(observer).unwrap();
                let witness = world.remove_resource::<RenameWitness>().unwrap();
                (
                    world
                        .compiled_scene_property_access_stats()
                        .saturating_delta_since(baseline),
                    actors.map(|actor| world.scene_binding_generation(actor)),
                    witness,
                )
            });
            println!("SEQUENCE_COVERAGE_V1 phase=stale_witness observer={witness:?} baseline={baseline:?} delta={after:?} roots_before={before_generations:?} roots_after={after_generations:?}");

            // Qualify the exact ordinary failure before restoring B. An unrelated
            // write error, conversion error or compile miss cannot produce this trace.
            let key = first.asset.installed(first.stage);
            first.assert_instance(&paused, added, key);
            first.assert_scene([Some(a), None, Some(b)], [SAMPLED, Vec3::ZERO], 1..=1);
            assert_eq!(
                witness.replaces,
                [1, 0],
                "qualification: real A Replace, no B Replace"
            );
            assert!(
                matches!(witness.renamed, Some(Ok(true))),
                "qualification: actual B rename"
            );
            let (at_a, values) = witness
                .at_a
                .expect("qualification: A write reached observer");
            let at_a = at_a.saturating_delta_since(baseline);
            assert_eq!(values, [SAMPLED, Vec3::ZERO]);
            assert_eq!(
                (
                    at_a.target_compilations,
                    at_a.field_dispatch_compilations,
                    at_a.compiled_writer_accesses,
                    at_a.compiled_reader_accesses,
                    at_a.stale_target_rejections
                ),
                (2, 2, 1, 0, 0)
            );
            assert_eq!(
                (
                    after.compiled_writer_accesses,
                    after.compiled_reader_accesses,
                    after.stale_target_rejections
                ),
                (1, 0, 1),
                "qualification: only B's second writer was stale"
            );
            assert_eq!(after_generations[0], before_generations[0]);
            assert!(after_generations[1] > before_generations[1]);

            edit_mixed_translation(frames, a, SENTINEL);
            frames.rename(b, STALE_B);
            let recovered = step("stale_same_b_returns");
            let quiet = step("stale_quiet");
            (actors, paused, added, first, recovered, quiet)
        },
    );
    let key = first.asset.installed(first.stage);
    for frame in [&recovered, &quiet] {
        frame.assert_instance(&paused, added, key);
    }
    let paths = [Some(a), Some(b), None];
    recovered.assert_scene(paths, [SENTINEL, B_SAMPLE], 1..=1);
    quiet.assert_scene(paths, [SENTINEL, B_SAMPLE], 0..=0);
}
