//! Disk-backed sequence residency between real PostUpdate frames.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::{AssetManager, ProjectManifest, SceneAsset};
use zircon_runtime::core::framework::animation::{
    AnimationChannelValueAsset, AnimationInterpolationAsset,
};
use zircon_runtime::core::resource::{
    ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceScheme, ResourceState,
    RuntimeResourceState,
};

use super::{
    checked_fixture_path, edit_mixed_translation, sequence_asset_for_entity,
    AnimationSequenceAsset, AnimationSequenceMarker, AssetUri, ProjectPaths, ReadySequence,
    ResourceHandle, ResourceKind, SequenceFrames, Vec3, PAUSED_TIME, SAMPLED, SENTINEL,
};

const BOUND: &str = "LeaseBound";
const LATE: &str = "LeaseLate";
const LATE_SAMPLE: Vec3 = Vec3::new(0.0, 2.0, 0.0);

fn owned_project_paths() -> ProjectPaths {
    let opt_in = PathBuf::from(
        std::env::var_os("ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT")
            .expect("qualification: set ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT to the owned TEMP"),
    );
    let declared = PathBuf::from(std::env::var_os("TEMP").expect("qualification: isolated TEMP"));
    let actual = std::env::temp_dir();
    // The shared checks reject aliases and handle GetTempPath's final separator.
    let scratch = checked_fixture_path(&opt_in, true);
    assert_eq!(scratch, checked_fixture_path(&declared, true));
    assert_eq!(scratch, checked_fixture_path(&actual, true));

    let physical = scratch.operation_path();
    let config = PathBuf::from(
        std::env::var_os("ZIRCON_CONFIG_PATH").expect("qualification: isolated config path"),
    );
    let config_file = checked_fixture_path(&config, false);
    let expected_config = physical.join(r"home\Config\config.json");
    assert!(
        ProjectPaths::same_lexical_path(config_file.operation_path(), expected_config).unwrap()
    );
    assert!(config_file != scratch && config_file.is_within(&scratch));
    assert_fresh_project(&ProjectPaths::from_root(physical).unwrap());

    // A pre-existing directory (even an empty one) is not owned by this invocation.
    let project_dir = physical.join("sequence-residency");
    fs::create_dir(&project_dir)
        .expect("qualification: claim a new sequence-residency project directory");
    let project = checked_fixture_path(&project_dir, true);
    assert!(project != scratch && project.is_within(&scratch));
    ProjectPaths::from_root(project.operation_path()).expect("qualification: owned project paths")
}

fn assert_fresh_project(paths: &ProjectPaths) {
    for path in [paths.manifest_path(), paths.derived_root()] {
        assert!(
            matches!(fs::symlink_metadata(path), Err(e) if e.kind() == ErrorKind::NotFound),
            "qualification: expected a fresh project at {}",
            path.display()
        );
    }
}

fn write_project(paths: &ProjectPaths, uri: &AssetUri, sequence: &[u8]) -> PathBuf {
    assert_fresh_project(paths);
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let manifest = ProjectManifest::new("Sequence Residency", scene_uri.clone(), 1);
    let assets = manifest.primary_asset_root_path(paths).unwrap();
    fs::create_dir(&assets).expect("qualification: claim a fresh project asset directory");
    // Real default-scene source; the frame test keeps its original World.
    let scene_path = assets.join(scene_uri.path());
    let scene = SceneAsset {
        entities: Vec::new(),
    };
    let scene = scene
        .to_project_toml_string(|_| unreachable!("no asset references"))
        .unwrap();
    fs::create_dir(scene_path.parent().unwrap()).unwrap();
    fs::write(scene_path, scene).unwrap();
    let source = assets.join(uri.path());
    fs::create_dir(source.parent().unwrap()).unwrap();
    fs::write(&source, sequence).expect("qualification: write sequence source");
    manifest
        .save(paths.manifest_path())
        .expect("qualification: save project manifest");
    source
}

struct CloseOnUnwind<'a>(&'a SequenceFrames);

impl Drop for CloseOnUnwind<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Err(error) = AssetManager::close_project(&*self.0.assets()) {
                eprintln!("SEQUENCE_RESIDENCY_V1 phase=close_on_unwind error={error}");
            }
        }
    }
}

fn authored_sequence() -> AnimationSequenceAsset {
    let mut authored = sequence_asset_for_entity(BOUND);
    let mut late_binding = sequence_asset_for_entity(LATE).bindings.remove(0);
    // B's Y channel continues changing after t=0.5, so advancing time cannot pass.
    let channel = &mut late_binding.tracks[0].channel;
    channel.interpolation = AnimationInterpolationAsset::Linear;
    channel.keys[0].out_tangent = None;
    channel.keys[1].in_tangent = None;
    channel.keys[1].time_seconds = 1.0;
    channel.keys[1].value = AnimationChannelValueAsset::Vec3([0.0, 4.0, 0.0]);
    authored.bindings.push(late_binding);
    authored
}

#[test]
#[ignore = "set ZIRCON_SEQUENCE_RESIDENCY_PROJECT_ROOT to an isolated Windows TEMP with home/Config/config.json"]
fn sequence_tick_residency_rebinds_only_missing_target() {
    let paths = owned_project_paths();
    let uri = AssetUri::parse("res://animation/lease.sequence.zranim").unwrap();
    let authored = authored_sequence();
    let bytes = authored
        .to_bytes()
        .expect("qualification: encode normal .zranim");
    let source = write_project(&paths, &uri, &bytes);

    let frames = SequenceFrames::open();
    let project_guard = CloseOnUnwind(&frames);
    let assets = frames.assets();
    let project = AssetManager::open_project(&*assets, paths.root().to_str().unwrap())
        .expect("qualification: activate and import the owned project");
    assert_eq!(Path::new(&project.root_path), paths.root());
    let project_token = || {
        frames
            .assets()
            .current_project_generation_snapshot()
            .unwrap()
            .into_parts()
            .1
    };
    let generation = project_token();
    let indexed = AssetManager::current_project_source_path(&*assets, &uri)
        .expect("qualification: project source path")
        .expect("qualification: active project source");
    let indexed = checked_fixture_path(&indexed, false);
    assert_eq!(
        indexed.operation_path(),
        ProjectPaths::resolve_existing_path(&source).unwrap()
    );
    assert_eq!(fs::read(&source).unwrap(), bytes);
    let id = assets
        .resolve_asset_id(&uri)
        .expect("qualification: imported sequence ID");
    let resources = assets.resource_manager();
    let catalog = || resources.registry().get(id).cloned();
    let record = catalog().expect("qualification: imported record");
    assert_eq!(
        (record.kind, record.state),
        (ResourceKind::AnimationSequence, ResourceState::Ready)
    );
    assert_eq!(record.primary_locator, uri);
    assert!(!record.source_hash.is_empty());
    let artifact = record
        .artifact_locator()
        .expect("qualification: durable sequence artifact");
    assert_eq!(artifact.scheme(), ResourceScheme::Library);
    let artifact_path = paths.asset_artifact_root().join(artifact.path());
    let artifact_file = checked_fixture_path(&artifact_path, false);
    let artifact_path = artifact_file.operation_path();
    let artifact_bytes =
        fs::read(artifact_path).expect("qualification: committed artifact manifest");

    let key = || resources.projection_snapshot().active().key(id);
    let facts = || {
        (
            resources.runtime_state(id),
            resources.ref_count(id),
            key(),
            resources.get_untyped(id).is_some(),
        )
    };
    let expect_unloaded = |stage| {
        assert_eq!(
            catalog().as_ref(),
            Some(&record),
            "{stage}: same Ready record"
        );
        assert_eq!(
            facts(),
            (Some(RuntimeResourceState::Unloaded), Some(0), None, false),
            "{stage}"
        );
    };
    let snapshot = |key: &ResourceActiveVersionKey| {
        resources.snapshot_expected::<AnimationSequenceMarker, AnimationSequenceAsset>(
            ResourceHandle::new(id),
            key,
        )
    };
    let checked_key = || {
        let key = key().expect("qualification: installed active version");
        let paired =
            snapshot(&key).expect("qualification: pair actual typed payload to active key");
        assert_eq!((paired.record(), &*paired), (&record, &authored));
        key
    };
    expect_unloaded("qualification: lazy Ready asset after project open");
    let first = assets
        .load_animation_sequence_asset(id)
        .expect("qualification: first disk-backed typed load");
    assert_eq!(first, authored);
    drop(assets);
    let initial_key = checked_key();
    let sequence = ReadySequence {
        id,
        revision: record.revision,
    };
    let (revision, hash) = (record.revision, &record.source_hash);
    println!(
        "SEQUENCE_RESIDENCY_V1 phase=first_load id={id} revision={revision} source_hash={hash}"
    );

    let bound = frames.spawn(BOUND);
    let clock = frames.player("LeaseClock", sequence, PAUSED_TIME, false);
    let paused = frames.player_stamp(clock);
    assert_eq!(frames.at_path(LATE), None);
    assert_eq!(frames.translation(bound), Vec3::ZERO);
    let requests = frames.frame("lease_initial", 1);
    assert_eq!(
        (requests, frames.translation(bound)),
        (1, SAMPLED),
        "qualification: initial A track sampled"
    );
    frames.assert_paused(clock, sequence, &paused);
    edit_mixed_translation(&frames, bound, SENTINEL);

    // No frame may see the unloaded interval: the scan sees the same Ready revision twice.
    let assets = frames.assets();
    assert_eq!(resources.ref_count(id), Some(0));
    let lease = assets
        .acquire_animation_sequence_asset(id)
        .expect("qualification: unique typed lease");
    assert_eq!((lease.id(), &*lease), (id, &authored));
    assert_eq!(
        facts(),
        (
            Some(RuntimeResourceState::Loaded),
            Some(1),
            Some(initial_key.clone()),
            true
        )
    );
    drop(lease);
    expect_unloaded("qualification: after last lease drop");
    println!(
        "SEQUENCE_RESIDENCY_V1 phase=unloaded id={id} revision={revision} leases=0-1-0 active=false"
    );
    let reloaded = assets
        .load_animation_sequence_asset(id)
        .expect("qualification: reload published artifact");
    assert_eq!(reloaded, authored);
    let reloaded_key = checked_key();
    assert_ne!(
        reloaded_key, initial_key,
        "qualification: a new active residency"
    );
    assert!(matches!(
        snapshot(&initial_key),
        Err(ResourceActiveSnapshotError::Stale)
    ));
    assert_eq!(assets.resolve_asset_id(&uri), Some(id));
    assert_eq!(catalog().as_ref(), Some(&record));
    assert_eq!(
        facts(),
        (
            Some(RuntimeResourceState::Loaded),
            Some(0),
            Some(reloaded_key.clone()),
            true
        )
    );
    assert_eq!(project_token(), generation);
    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(fs::read(artifact_path).unwrap(), artifact_bytes);
    drop(assets);
    frames.assert_paused(clock, sequence, &paused);
    assert_eq!(frames.at_path(LATE), None);
    println!("SEQUENCE_RESIDENCY_V1 phase=reloaded id={id} revision={revision} source_hash={hash} same_bytes=true new_active_key=true");

    let late = frames.spawn(LATE);
    assert_eq!(frames.at_path(LATE), Some(late));
    assert_eq!(
        [frames.translation(bound), frames.translation(late)],
        [SENTINEL, Vec3::ZERO]
    );
    let observe = |stage| {
        let requests = frames.frame(stage, 1);
        let translations = [frames.translation(bound), frames.translation(late)];
        println!("SEQUENCE_RESIDENCY_V1 phase={stage} requests={requests} a={bound:?} b={late:?} translations={translations:?}");
        frames.assert_paused(clock, sequence, &paused);
        assert_eq!(key().as_ref(), Some(&reloaded_key));
        assert!(!frames.level.animation_requires_continuous_frame());
        (requests, translations)
    };
    let after_spawn = observe("lease_late");
    let next_frame = observe("lease_quiet");

    let assets = frames.assets();
    let closed = AssetManager::close_project(&*assets).expect("teardown: close the owned project");
    assert_eq!(closed.as_deref(), Some(paths.root()));
    assert!(assets.current_project_manager().is_none() && catalog().is_none());
    drop(assets);
    drop(project_guard);
    println!("SEQUENCE_RESIDENCY_V1 phase=closed");

    let translations = [SENTINEL, LATE_SAMPLE];
    assert_eq!(
        after_spawn,
        (1, translations),
        "behavior: sample only newly bound B after same-source re-residency"
    );
    assert_eq!(
        next_frame,
        (0, translations),
        "behavior: subsequent paused frame must be quiet"
    );
}

#[path = "coverage.rs"]
mod coverage;
