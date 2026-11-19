//! Real LevelSystem frames using the product portable animation registration.

#[allow(dead_code)] // Reuse the sequence payload from physics_tick, not its native SDK host.
#[path = "animation_assets.rs"]
mod animation_assets;

use std::path::{Path, PathBuf};
use std::time::Duration;

use animation_assets::sequence_asset_for_entity;
use zircon_plugin_animation_runtime::{self as animation, AnimationEvaluationPipeline};
use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPathIdentity};
use zircon_runtime::asset::{self, AssetUri, ProjectAssetManager};
use zircon_runtime::core::framework::{animation::AnimationSequenceAsset, scene::EntityPath};
use zircon_runtime::core::manager::{animation_manager_handle, resolve_manager_service};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::core::resource::{
    AnimationSequenceMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::core::runtime::modules::{FrameCountModule, TasksModule, TimeModule};
use zircon_runtime::core::{CoreRuntime, ServiceCallGuard};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::scene::components::{
    AnimationSequencePlayerComponent, LocalTransform, NodeKind,
};
use zircon_runtime::scene::ecs::{ChangeTick, Ref};
use zircon_runtime::scene::{self, EntityId, LevelSystem, World};

const FRAME: Duration = Duration::from_millis(125);
const PAUSED_TIME: Real = 0.5;
const SAMPLED: Vec3 = Vec3::new(2.0, 0.0, 0.0);
const SENTINEL: Vec3 = Vec3::new(17.0, 0.0, 0.0);

type PlayerStamp = (AnimationSequencePlayerComponent, ChangeTick);

#[derive(Clone, Copy)]
struct ReadySequence {
    id: ResourceId,
    revision: u64,
}

struct SequenceFrames {
    level: LevelSystem,
    core: CoreRuntime,
    epoch: u64,
}

fn checked_fixture_path(path: &Path, directory: bool) -> ResolvedProjectPathIdentity {
    assert!(path.is_absolute(), "fixture path is not absolute: {path:?}");
    let is_separator = |byte| byte == b'/' || (cfg!(windows) && byte == b'\\');
    let raw = path.as_os_str().as_encoded_bytes();
    // Path::components would discard interior `.` before the admission check.
    assert!(
        !raw.split(|&byte| is_separator(byte))
            .any(|part| part == b"." || part == b".."),
        "fixture path contains a dot component: {path:?}"
    );
    assert!(
        directory || !raw.last().is_some_and(|&byte| is_separator(byte)),
        "config must name a file, not a directory: {path:?}"
    );
    let spelling = if directory {
        directory_spelling(path)
    } else {
        path.to_path_buf()
    };
    assert!(
        spelling.is_absolute() && spelling.parent().is_some(),
        "fixture cannot be a filesystem root: {path:?}"
    );
    for ancestor in spelling.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor)
            .unwrap_or_else(|err| panic!("cannot inspect fixture component {ancestor:?}: {err}"));
        assert!(
            !metadata.file_type().is_symlink(),
            "fixture symlink: {ancestor:?}"
        );
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            assert_eq!(
                metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT,
                0,
                "fixture reparse point: {ancestor:?}"
            );
        }
    }
    let resolved = ProjectPaths::resolve_existing(path)
        .unwrap_or_else(|err| panic!("cannot resolve existing fixture path {path:?}: {err}"));
    let physical = resolved.operation_path();
    assert!(
        if directory {
            physical.is_dir()
        } else {
            physical.is_file()
        },
        "unexpected fixture path kind: {path:?}"
    );
    assert!(
        ProjectPaths::same_lexical_path(&spelling, resolved.display_path()).unwrap(),
        "fixture path alias: declared={path:?} physical={:?}",
        resolved.display_path()
    );
    resolved.into()
}

#[cfg(windows)]
fn directory_spelling(path: &Path) -> PathBuf {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    // GetTempPath supplies a final separator that canonicalization does not.
    // Remove only that directory syntax; no component or non-ASCII data changes.
    let mut wide: Vec<_> = path.as_os_str().encode_wide().collect();
    while wide
        .last()
        .is_some_and(|&unit| unit == u16::from(b'\\') || unit == u16::from(b'/'))
    {
        wide.pop();
    }
    PathBuf::from(std::ffi::OsString::from_wide(&wide))
}

#[cfg(not(windows))]
fn directory_spelling(path: &Path) -> PathBuf {
    // Non-Windows ProjectPaths already compares by components. Strip the final
    // directory syntax for symlink_metadata as well, after rejecting raw dots.
    path.components().collect()
}

impl SequenceFrames {
    fn open() -> Self {
        // Use the normal Foundation config selector, never a user's default profile.
        let declared = PathBuf::from(
            std::env::var_os("ZIRCON_CONFIG_PATH").expect("set an isolated ZIRCON_CONFIG_PATH"),
        );
        let declared_temp = PathBuf::from(std::env::var_os("TEMP").expect("set an isolated TEMP"));
        let scratch = std::env::temp_dir();
        let temp = checked_fixture_path(&declared_temp, true);
        let actual_temp = checked_fixture_path(&scratch, true);
        assert_eq!(
            actual_temp, temp,
            "system temp_dir must select the declared physical TEMP"
        );
        let config = checked_fixture_path(&declared, false);
        assert!(
            config != temp && config.is_within(&temp),
            "config must be under the isolated temp directory"
        );
        println!(
            "SEQUENCE_FIXTURE_V1 declared_temp={declared_temp:?} os_temp={scratch:?} physical_temp={:?}",
            temp.operation_path()
        );

        let core = CoreRuntime::try_new().expect("start real Core");
        // Like BrowserEngine, use the shipped portable Animation module/plan. The
        // native libtest selects the real native AssetModule for resident payloads.
        for descriptor in [
            TasksModule.descriptor(),
            TimeModule.descriptor(),
            FrameCountModule.descriptor(),
            zircon_runtime::foundation::module_descriptor(),
            asset::module_descriptor(),
            scene::module_descriptor(),
            animation::module_descriptor(),
        ] {
            core.register_module(descriptor)
                .expect("register real module");
        }
        core.activate_registered_modules()
            .expect("activate real module graph");
        let handle = core.handle();
        let settings = resolve_manager_service(&handle, animation_manager_handle(&handle).unwrap())
            .expect("production AnimationManager")
            .playback_settings();
        assert!(settings.enabled && settings.property_tracks);
        scene::install_world_runtime_extension_plan(
            &handle,
            animation::portable_runtime_extension_plan().expect("portable Animation plan"),
        )
        .expect("install product plan in WorldDriver");
        let level = scene::create_level(&handle, World::empty(), Default::default())
            .expect("create LevelSystem with installed plan");
        assert!(level.with_world(|world| world.contains_resource::<AnimationEvaluationPipeline>()));
        let epoch = level.capture_world_replacement_epoch();
        Self { level, core, epoch }
    }

    fn assets(&self) -> ServiceCallGuard<ProjectAssetManager> {
        let handle = self.core.handle();
        resolve_manager_service(
            &handle,
            asset::project_asset_manager_handle(&handle).unwrap(),
        )
        .expect("production ProjectAssetManager")
    }

    fn ready(&self, name: &str, asset: AnimationSequenceAsset) -> ReadySequence {
        let uri = AssetUri::parse(&format!("res://animation/{name}.sequence.zranim")).unwrap();
        let id = ResourceId::from_locator(&uri);
        let manager = self.assets();
        manager.resource_manager().register_ready(
            ResourceRecord::new(id, ResourceKind::AnimationSequence, uri),
            asset.clone(),
        );
        assert_eq!(
            manager
                .load_animation_sequence_asset(id)
                .expect("production Ready loader"),
            asset
        );
        ReadySequence {
            id,
            revision: self.asset_revision(id),
        }
    }

    fn asset_revision(&self, id: ResourceId) -> u64 {
        self.assets()
            .resource_manager()
            .snapshot::<AnimationSequenceMarker, AnimationSequenceAsset>(ResourceHandle::new(id))
            .expect("typed Ready revision")
            .revision()
    }

    fn spawn(&self, name: &str) -> EntityId {
        self.level.with_world_mut(|world| {
            let entity = world.spawn_node(NodeKind::Empty).unwrap();
            assert!(world.rename_node(entity, name).unwrap());
            entity
        })
    }

    fn rename(&self, entity: EntityId, name: &str) {
        self.level
            .with_world_mut(|world| assert!(world.rename_node(entity, name).unwrap()));
    }

    fn at_path(&self, name: &str) -> Option<EntityId> {
        let path = EntityPath::parse(name).unwrap();
        self.level
            .with_world(|world| world.get_entity_by_path(&path))
    }

    fn player(
        &self,
        name: &str,
        sequence: ReadySequence,
        time_seconds: Real,
        playing: bool,
    ) -> EntityId {
        let entity = self.spawn(name);
        self.level.with_world_mut(|world| {
            world
                .set_animation_sequence_player(
                    entity,
                    Some(AnimationSequencePlayerComponent {
                        sequence: ResourceHandle::new(sequence.id),
                        playback_speed: 1.0,
                        time_seconds,
                        looping: false,
                        playing,
                    }),
                )
                .unwrap();
        });
        entity
    }

    fn player_stamp(&self, entity: EntityId) -> PlayerStamp {
        self.level.with_world_mut(|world| {
            let query = world.query::<Ref<'static, AnimationSequencePlayerComponent>>();
            let player = query.get(world, entity).unwrap();
            ((*player).clone(), player.last_changed())
        })
    }

    #[track_caller]
    fn assert_paused(&self, entity: EntityId, sequence: ReadySequence, before: &PlayerStamp) {
        assert_eq!(
            &self.player_stamp(entity),
            before,
            "paused player value and change tick"
        );
        assert_eq!(before.0.time_seconds, PAUSED_TIME);
        assert!(!before.0.playing);
        assert_eq!(
            self.asset_revision(sequence.id),
            sequence.revision,
            "Ready revision must not change"
        );
    }

    fn translation(&self, entity: EntityId) -> Vec3 {
        self.level.with_world(|world| {
            world
                .get::<LocalTransform>(entity)
                .unwrap()
                .transform
                .translation
        })
    }

    fn frame(&self, stage: &str, players: u64) -> u64 {
        let stats = || {
            self.level.with_world(|world| {
                world
                    .resource::<AnimationEvaluationPipeline>()
                    .projection_stats()
            })
        };
        let before = stats();
        let time = self.core.advance_time_by(FRAME, 4);
        self.level
            .tick(&self.core.handle(), time)
            .expect("real LevelSystem frame");
        let after = stats();
        let scanned =
            after.sequence_player_candidate_count - before.sequence_player_candidate_count;
        let requested = after.sequence_request_count - before.sequence_request_count;
        println!(
            "SEQUENCE_TICK_V1 stage={stage} frame={} players={scanned} requests={requested}",
            time.outer_frame_index()
        );
        assert_eq!(self.level.capture_world_replacement_epoch(), self.epoch);
        assert_eq!(
            scanned, players,
            "PostUpdate must observe every player at {stage}"
        );
        requested
    }
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_samples_initial_paused_target() {
    let frames = SequenceFrames::open();
    let preferred = frames.spawn("StableIdTarget");
    let fallback = frames.spawn("StartupFallback");
    let mut asset = sequence_asset_for_entity("StartupFallback");
    asset.bindings[0].target_id = Some(preferred.to_string());
    let sequence = frames.ready("initial-paused", asset);
    let clock = frames.player("SequenceClock", sequence, PAUSED_TIME, false);
    let before = frames.player_stamp(clock);
    assert_eq!(frames.at_path("StartupFallback"), Some(fallback));
    assert_eq!(frames.translation(preferred), Vec3::ZERO);
    assert_eq!(frames.frame("initial_bound", 1), 1);
    assert_eq!(
        frames.translation(preferred),
        SAMPLED,
        "real stable ID wins"
    );
    assert_eq!(
        frames.translation(fallback),
        Vec3::ZERO,
        "valid fallback loses"
    );
    assert_eq!(frames.translation(clock), Vec3::ZERO);
    assert!(!frames.level.animation_requires_continuous_frame());
    frames.assert_paused(clock, sequence, &before);
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_paused_retries_spawned_target() {
    let frames = SequenceFrames::open();
    let sequence = frames.ready("paused-spawn", sequence_asset_for_entity("SpawnedTarget"));
    let clock = frames.player("SequenceClock", sequence, PAUSED_TIME, false);
    let before = frames.player_stamp(clock);
    assert_eq!(frames.at_path("SpawnedTarget"), None);
    assert_eq!(frames.frame("initial_missing", 1), 1);
    assert_eq!(frames.translation(clock), Vec3::ZERO);
    assert_eq!(frames.frame("unchanged_missing", 1), 0);
    frames.assert_paused(clock, sequence, &before);

    let target = frames.spawn("SpawnedTarget");
    assert_eq!(frames.at_path("SpawnedTarget"), Some(target));
    assert_eq!(frames.translation(target), Vec3::ZERO);
    let requested = frames.frame("after_spawn", 1);
    frames.assert_paused(clock, sequence, &before);
    assert_eq!(
        (frames.translation(target), requested),
        (SAMPLED, 1),
        "late spawn must sample at the unchanged paused time"
    );
    assert_eq!(frames.frame("after_spawn_settled", 1), 0);
    assert_eq!(frames.translation(target), SAMPLED);
    assert!(!frames.level.animation_requires_continuous_frame());
    frames.assert_paused(clock, sequence, &before);
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_paused_retries_renamed_target() {
    let frames = SequenceFrames::open();
    let preferred = frames.spawn("PreferredBeforeRename");
    let fallback = frames.spawn("FallbackBeforeRename");
    let mut asset = sequence_asset_for_entity("RenamedFallback");
    asset.bindings[0].target_id = Some("RenamedPreferred".into());
    let sequence = frames.ready("paused-rename", asset);
    let clock = frames.player("SequenceClock", sequence, PAUSED_TIME, false);
    let before = frames.player_stamp(clock);
    assert_eq!(frames.at_path("RenamedPreferred"), None);
    assert_eq!(frames.at_path("RenamedFallback"), None);
    assert_eq!(frames.frame("initial_missing", 1), 1);
    assert_eq!(frames.translation(preferred), Vec3::ZERO);
    assert_eq!(frames.translation(fallback), Vec3::ZERO);
    assert_eq!(frames.frame("unchanged_missing", 1), 0);

    frames.rename(fallback, "RenamedFallback");
    frames.rename(preferred, "RenamedPreferred");
    assert_eq!(frames.at_path("RenamedPreferred"), Some(preferred));
    assert_eq!(frames.at_path("RenamedFallback"), Some(fallback));
    assert_eq!(frames.translation(preferred), Vec3::ZERO);
    let requested = frames.frame("after_rename", 1);
    frames.assert_paused(clock, sequence, &before);
    assert_eq!(
        (
            frames.translation(preferred),
            frames.translation(fallback),
            requested
        ),
        (SAMPLED, Vec3::ZERO, 1),
        "retry must preserve preferred target-id selection before the valid fallback"
    );
    assert_eq!(frames.frame("after_rename_settled", 1), 0);
    assert!(!frames.level.animation_requires_continuous_frame());
    frames.assert_paused(clock, sequence, &before);
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_paused_keeps_bound_target_quiet() {
    let frames = SequenceFrames::open();
    let bound = frames.spawn("BoundTarget");
    let busy = frames.spawn("BusyTarget");
    let noise = frames.spawn("UnrelatedRootBeforeRename");
    let paused = frames.ready("already-bound", sequence_asset_for_entity("BoundTarget"));
    let playing = frames.ready("other-playing", sequence_asset_for_entity("BusyTarget"));
    let clock = frames.player("PausedClock", paused, PAUSED_TIME, false);
    let busy_clock = frames.player("PlayingClock", playing, 0.0, true);
    let before = frames.player_stamp(clock);
    assert_eq!(frames.frame("initial_bound_plus_playing", 2), 2);
    assert_eq!(frames.translation(bound), SAMPLED);
    let binding_generation = |id| {
        frames
            .level
            .with_world(|world| world.scene_binding_generation(id))
    };
    let bound_generation = binding_generation(bound);

    // A value different from the sampled key exposes otherwise invisible rewrites.
    frames.level.with_world_mut(|world| {
        assert!(world
            .update_transform(bound, Transform::from_translation(SENTINEL))
            .unwrap());
    });
    assert_eq!(frames.frame("playing_only", 2), 1);
    assert_eq!(frames.translation(bound), SENTINEL);
    for name in ["UnrelatedRootRenameA", "UnrelatedRootRenameB"] {
        let noise_before = binding_generation(noise);
        frames.rename(noise, name);
        assert_ne!(binding_generation(noise), noise_before);
        assert_eq!(binding_generation(bound), bound_generation);
        let requested = frames.frame(name, 2);
        assert_eq!(
            (frames.translation(bound), requested),
            (SENTINEL, 1),
            "only the unrelated playing sequence may submit; no blanket paused reapply"
        );
        frames.assert_paused(clock, paused, &before);
    }
    assert_eq!(frames.player_stamp(busy_clock).0.time_seconds, PAUSED_TIME);
    assert_eq!(frames.translation(busy), SAMPLED);
    assert_eq!(frames.asset_revision(playing.id), playing.revision);
    assert!(frames.level.animation_requires_continuous_frame());
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_playing_retries_spawned_target() {
    let frames = SequenceFrames::open();
    let sequence = frames.ready("playing-spawn", sequence_asset_for_entity("PlayingTarget"));
    let clock = frames.player("PlayingClock", sequence, 0.25, true);
    assert_eq!(frames.at_path("PlayingTarget"), None);
    assert_eq!(frames.frame("playing_missing", 1), 1);
    assert_eq!(frames.player_stamp(clock).0.time_seconds, 0.375);
    assert_eq!(frames.translation(clock), Vec3::ZERO);
    assert!(frames.level.animation_requires_continuous_frame());

    let target = frames.spawn("PlayingTarget");
    assert_eq!(frames.translation(target), Vec3::ZERO);
    assert_eq!(frames.frame("playing_after_spawn", 1), 1);
    assert_eq!(frames.translation(target), SAMPLED);
    assert_eq!(frames.player_stamp(clock).0.time_seconds, PAUSED_TIME);
    assert_eq!(frames.asset_revision(sequence.id), sequence.revision);
    assert!(frames.level.animation_requires_continuous_frame());
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_paused_retries_shared_asset_after_eviction() {
    let frames = SequenceFrames::open();
    let busy = frames.spawn("BusyTarget");
    let shared = frames.ready(
        "shared-paused",
        sequence_asset_for_entity("SharedLateTarget"),
    );
    let playing = frames.ready("other-playing", sequence_asset_for_entity("BusyTarget"));
    let a = frames.player("PausedClockA", shared, PAUSED_TIME, false);
    let b = frames.player("PausedClockB", shared, PAUSED_TIME, false);
    let playing_clock = frames.player("PlayingClock", playing, 0.0, true);
    let a_before = frames.player_stamp(a);
    let b_before = frames.player_stamp(b);
    assert_eq!(frames.at_path("SharedLateTarget"), None);
    assert_eq!(frames.frame("shared_missing_plus_playing", 3), 3);
    assert_eq!(frames.translation(a), Vec3::ZERO);
    assert_eq!(frames.translation(b), Vec3::ZERO);

    // Only the other asset loads this frame. The per-frame request-set retention
    // of the compiled sequence cache cannot preserve the shared missing asset.
    assert_eq!(frames.frame("playing_only_evicts_missing", 3), 1);
    assert_eq!(frames.player_stamp(playing_clock).0.time_seconds, 0.25);
    assert_eq!(frames.at_path("SharedLateTarget"), None);
    let target = frames.spawn("SharedLateTarget");
    assert_eq!(frames.translation(target), Vec3::ZERO);
    let requested = frames.frame("shared_retries_after_eviction", 3);
    frames.assert_paused(a, shared, &a_before);
    frames.assert_paused(b, shared, &b_before);
    assert_eq!(
        (frames.translation(target), requested),
        (SAMPLED, 3),
        "both paused players must be admitted with the other playing asset"
    );
    assert_eq!(frames.frame("settled_shared_plus_playing", 3), 1);
    assert_eq!(frames.translation(target), SAMPLED);
    assert_eq!(frames.translation(busy), SAMPLED);
    assert_eq!(
        frames.player_stamp(playing_clock).0.time_seconds,
        PAUSED_TIME
    );
    assert_eq!(frames.asset_revision(playing.id), playing.revision);
    frames.assert_paused(a, shared, &a_before);
    frames.assert_paused(b, shared, &b_before);
}

fn mixed_transform_sequence(first: &str, others: &[&str]) -> AnimationSequenceAsset {
    let mut asset = sequence_asset_for_entity(first);
    for target in others {
        asset
            .bindings
            .extend(sequence_asset_for_entity(target).bindings);
    }
    asset
}

fn edit_mixed_translation(frames: &SequenceFrames, target: EntityId, next: Vec3) {
    frames.level.with_world_mut(|world| {
        let mut transform = world.get::<LocalTransform>(target).unwrap().transform;
        transform.translation = next;
        assert!(world.update_transform(target, transform).unwrap());
    });
}

#[track_caller]
fn assert_mixed_frame<const N: usize>(
    frames: &SequenceFrames,
    stage: &str,
    targets: [EntityId; N],
    expected: [Vec3; N],
    requests: Option<u64>,
) {
    let observed_requests = frames.frame(stage, 1);
    let actual = targets.map(|entity| frames.translation(entity));
    println!(
        "SEQUENCE_MIXED_V1 stage={stage} requests={observed_requests} targets={targets:?} translations={actual:?}"
    );
    assert_eq!(actual, expected, "target translations at {stage}");
    if let Some(requests) = requests {
        assert_eq!(observed_requests, requests, "sequence requests at {stage}");
    } else {
        // While bindings are unresolved, an unrelated catalog edit may retry once.
        assert!(
            observed_requests <= 1,
            "duplicate sequence requests at {stage}"
        );
    }
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_mixed_late_bindings_preserve_independent_edits() {
    let frames = SequenceFrames::open();
    let bound = frames.spawn("MixedBound");
    let renamable = frames.spawn("MixedBeforeRename");
    let unrelated = frames.spawn("MixedUnrelated");
    let asset = mixed_transform_sequence("MixedBound", &["MixedSpawned", "MixedRenamed"]);
    let sequence = frames.ready("mixed-late-targets", asset);
    let clock = frames.player("MixedClock", sequence, PAUSED_TIME, false);
    let before = frames.player_stamp(clock);
    assert_eq!(frames.at_path("MixedSpawned"), None);
    assert_eq!(frames.at_path("MixedRenamed"), None);
    assert_eq!(frames.translation(bound), Vec3::ZERO);
    assert_mixed_frame(
        &frames,
        "mixed_initial_partial",
        [bound, renamable],
        [SAMPLED, Vec3::ZERO],
        Some(1),
    );

    edit_mixed_translation(&frames, bound, SENTINEL);
    assert_mixed_frame(
        &frames,
        "mixed_authored_edit_settles",
        [bound, renamable],
        [SENTINEL, Vec3::ZERO],
        Some(0),
    );
    frames.rename(unrelated, "MixedStillUnrelated");
    assert_mixed_frame(
        &frames,
        "mixed_unrelated_catalog_edit",
        [bound, renamable, unrelated],
        [SENTINEL, Vec3::ZERO, Vec3::ZERO],
        None,
    );
    frames.assert_paused(clock, sequence, &before);

    let spawned = frames.spawn("MixedSpawned");
    assert_eq!(frames.at_path("MixedSpawned"), Some(spawned));
    assert_eq!(frames.at_path("MixedRenamed"), None);
    assert_eq!(frames.translation(spawned), Vec3::ZERO);
    let targets = [bound, spawned, renamable];
    let after_spawn = [SENTINEL, SAMPLED, Vec3::ZERO];
    assert_mixed_frame(&frames, "mixed_after_spawn", targets, after_spawn, Some(1));
    assert_mixed_frame(
        &frames,
        "mixed_spawn_settled",
        targets,
        after_spawn,
        Some(0),
    );
    frames.assert_paused(clock, sequence, &before);

    let edited_spawn = Vec3::new(0.0, 29.0, 0.0);
    edit_mixed_translation(&frames, spawned, edited_spawn);
    frames.rename(renamable, "MixedRenamed");
    assert_eq!(frames.at_path("MixedRenamed"), Some(renamable));
    assert_eq!(frames.translation(renamable), Vec3::ZERO);
    let after_rename = [SENTINEL, edited_spawn, SAMPLED];
    assert_mixed_frame(
        &frames,
        "mixed_after_rename",
        targets,
        after_rename,
        Some(1),
    );
    assert_mixed_frame(
        &frames,
        "mixed_rename_settled",
        targets,
        after_rename,
        Some(0),
    );
    assert_eq!(frames.translation(clock), Vec3::ZERO);
    assert!(!frames.level.animation_requires_continuous_frame());
    frames.assert_paused(clock, sequence, &before);
}

fn publish_mixed_ready(
    frames: &SequenceFrames,
    uri: &AssetUri,
    asset: &AnimationSequenceAsset,
) -> ReadySequence {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    // Identify the actual encoded fixture content; ResourceManager owns the revision.
    let mut digest = DefaultHasher::new();
    digest.write(&asset.to_bytes().unwrap());
    let id = ResourceId::from_locator(uri);
    let manager = frames.assets();
    manager
        .resource_manager()
        .register_ready(
            ResourceRecord::new(id, ResourceKind::AnimationSequence, uri.clone())
                .with_source_hash(format!("{:016x}", digest.finish())),
            asset.clone(),
        )
        .expect("publish Ready sequence content");
    assert_eq!(&manager.load_animation_sequence_asset(id).unwrap(), asset);
    ReadySequence {
        id,
        revision: frames.asset_revision(id),
    }
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_mixed_explicit_seek_and_asset_revision_resample_all_tracks() {
    use zircon_runtime::core::framework::animation::AnimationChannelValueAsset;

    let frames = SequenceFrames::open();
    let first = frames.spawn("MixedExplicitFirst");
    let second = frames.spawn("MixedExplicitSecond");
    let targets = [first, second];
    let mut asset = mixed_transform_sequence(
        "MixedExplicitFirst",
        &["MixedExplicitSecond", "MixedExplicitMissing"],
    );
    let uri = AssetUri::parse("res://animation/mixed-explicit.sequence.zranim").unwrap();
    let sequence = publish_mixed_ready(&frames, &uri, &asset);
    let clock = frames.player("MixedExplicitClock", sequence, PAUSED_TIME, false);
    let before = frames.player_stamp(clock);
    assert_eq!(frames.at_path("MixedExplicitMissing"), None);
    assert_mixed_frame(
        &frames,
        "mixed_explicit_initial",
        targets,
        [SAMPLED; 2],
        Some(1),
    );
    frames.assert_paused(clock, sequence, &before);

    let edits = [SENTINEL, Vec3::new(0.0, 19.0, 0.0)];
    for (target, edit) in targets.into_iter().zip(edits) {
        edit_mixed_translation(&frames, target, edit);
    }
    assert_mixed_frame(
        &frames,
        "mixed_before_explicit_seek",
        targets,
        edits,
        Some(0),
    );
    let mut requested_seek = before.0.clone();
    requested_seek.time_seconds = 0.0;
    frames.level.with_world_mut(|world| {
        assert!(world
            .set_animation_sequence_player(clock, Some(requested_seek.clone()))
            .unwrap());
    });
    let seeked = frames.player_stamp(clock);
    assert_eq!(seeked.0, requested_seek);
    assert_ne!(seeked.1, before.1);
    assert_mixed_frame(
        &frames,
        "mixed_explicit_seek",
        targets,
        [Vec3::ZERO; 2],
        Some(1),
    );
    assert_mixed_frame(
        &frames,
        "mixed_seek_settled",
        targets,
        [Vec3::ZERO; 2],
        Some(0),
    );
    assert_eq!(frames.player_stamp(clock), seeked);
    assert_eq!(frames.asset_revision(sequence.id), sequence.revision);

    let revised_values = [Vec3::new(7.0, 0.0, 0.0), Vec3::new(0.0, 8.0, 0.0)];
    for (binding, sample) in asset.bindings.iter_mut().zip(revised_values) {
        binding.tracks[0].channel.keys[0].value =
            AnimationChannelValueAsset::Vec3(sample.to_array());
    }
    let revised = publish_mixed_ready(&frames, &uri, &asset);
    assert_eq!(revised.id, sequence.id);
    assert!(revised.revision > sequence.revision);
    assert_mixed_frame(
        &frames,
        "mixed_explicit_ready_revision",
        targets,
        revised_values,
        Some(1),
    );
    assert_mixed_frame(
        &frames,
        "mixed_revision_settled",
        targets,
        revised_values,
        Some(0),
    );
    assert_eq!(frames.player_stamp(clock), seeked);
    assert_eq!(frames.asset_revision(sequence.id), revised.revision);
    assert_eq!(frames.at_path("MixedExplicitMissing"), None);
    assert_eq!(frames.translation(clock), Vec3::ZERO);
    assert!(!frames.level.animation_requires_continuous_frame());
}
