use serde_json::json;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::sound::{
    SoundPlaybackCompletionAction, SoundSourceDescriptor, SoundSourceInput, SoundSourceManager,
    AUDIO_SOURCE_COMPONENT_TYPE,
};
use zircon_runtime::scene::{self, NodeKind, World};

use crate::scene_system::{PLAYBACK_MODULE, SYNC_SYSTEM};

use super::scene_support::{add, extensions, set, Fixture};

#[test]
fn scene_audio_source_creates_playing_service_source_and_reuses_identity() {
    let fixture = Fixture::client();
    add(&fixture.level, fixture.component());
    assert!(fixture.sources().is_empty());
    fixture.tick();
    let sources = fixture.sources();
    assert_eq!(sources.len(), 1);
    let id = sources[0].id.unwrap();
    let status = fixture.manager.source_status(id).unwrap();
    assert_eq!(status.input, SoundSourceInput::Clip(fixture.clip));
    assert!(status.playing);
    assert!(!status.muted);
    assert!(!status.looped);
    assert_eq!(status.gain, 1.0);
    for _ in 0..3 {
        fixture.tick();
        assert_eq!(fixture.sources(), sources);
    }
    fixture.end();
    assert!(fixture.sources().is_empty());
}

#[test]
fn scene_audio_gain_mute_and_speed_updates_preserve_source_controls_then_loop_rebinds() {
    let fixture = Fixture::client();
    let mut value = fixture.component();
    let entity = add(&fixture.level, value.clone());
    fixture.tick();
    let id = fixture.sources()[0].id.unwrap();
    // A full update_source would restart this paused voice. Live scalar edits
    // must use the existing targeted controls and preserve its playback state.
    fixture.manager.pause_source(id).unwrap();
    for muted in [true, false] {
        value["gain"] = json!(0.25);
        value["speed"] = json!(1.5);
        value["muted"] = json!(muted);
        set(&fixture.level, entity, value.clone());
        fixture.tick();
        let status = fixture.manager.source_status(id).unwrap();
        assert!(!status.playing);
        assert_eq!(status.muted, muted);
        assert_eq!(status.gain, 0.25);
        assert_eq!(status.speed, 1.5);
        assert_eq!(fixture.sources().len(), 1);
    }
    value["looped"] = json!(true);
    value["start_seconds"] = json!(0.25);
    value["duration_seconds"] = json!(0.5);
    value["completion_action"] = json!("DespawnEntity");
    set(&fixture.level, entity, value);
    fixture.tick();
    let status = fixture.manager.source_status(id).unwrap();
    assert!(status.looped);
    assert!(status.playing);
    assert_eq!(status.range_start_frame, 2);
    assert_eq!(status.range_end_frame, Some(6));
    assert_eq!(
        status.completion_action,
        SoundPlaybackCompletionAction::DespawnEntity
    );
    assert_eq!(fixture.sources().len(), 1);
    fixture.end();
}

#[test]
fn scene_audio_stop_disable_component_and_entity_removal_release_sources() {
    let fixture = Fixture::client();
    for removal in 0..4 {
        let mut value = fixture.component();
        value["playing"] = json!(false);
        let entity = add(&fixture.level, value.clone());
        fixture.tick();
        assert!(fixture.sources().is_empty());
        value["playing"] = json!(true);
        set(&fixture.level, entity, value.clone());
        fixture.tick();
        let id = fixture.sources()[0].id.unwrap();
        match removal {
            0 => {
                value["playing"] = json!(false);
                set(&fixture.level, entity, value);
            }
            1 => {
                value["enabled"] = json!(false);
                set(&fixture.level, entity, value);
            }
            2 => fixture.level.with_world_mut(|world| {
                world
                    .remove_dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
                    .unwrap();
            }),
            _ => fixture
                .level
                .with_world_mut(|world| world.remove_entity(entity).unwrap()),
        }
        fixture.tick();
        assert!(fixture.sources().is_empty());
        assert!(fixture.manager.source_status(id).is_err());
    }
    fixture.end();
}

#[test]
fn scene_audio_inactive_hierarchy_releases_and_reactivation_creates_fresh_source() {
    let fixture = Fixture::client();
    let child = add(&fixture.level, fixture.component());
    let parent = fixture.level.with_world_mut(|world| {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world.set_parent_checked(child, Some(parent)).unwrap();
        parent
    });
    fixture.tick();
    let first = fixture.sources()[0].id.unwrap();
    fixture
        .level
        .with_world_mut(|world| world.set_active_self(parent, false).unwrap());
    fixture.tick();
    assert!(fixture.sources().is_empty());
    fixture
        .level
        .with_world_mut(|world| world.set_active_self(parent, true).unwrap());
    fixture.tick();
    assert_ne!(fixture.sources()[0].id.unwrap(), first);
    fixture.end();
}

#[test]
fn scene_audio_world_replacement_and_session_cleanup_preserve_other_owners() {
    let fixture = Fixture::client();
    let manual = fixture
        .manager
        .create_source(SoundSourceDescriptor::clip(fixture.clip))
        .unwrap();
    let mut value = fixture.component();
    value["id"] = json!(manual);
    add(&fixture.level, value);
    fixture.tick();
    let first = fixture
        .sources()
        .into_iter()
        .find_map(|source| source.id.filter(|id| *id != manual))
        .unwrap();
    let second_level = scene::create_level(
        &fixture.runtime.handle(),
        World::empty(),
        Default::default(),
    )
    .unwrap();
    add(&second_level, fixture.component());
    let frame = fixture
        .runtime
        .advance_time_by(std::time::Duration::from_millis(100), 1);
    second_level.tick(&fixture.runtime.handle(), frame).unwrap();
    fixture
        .level
        .tick(&fixture.runtime.handle(), frame)
        .unwrap();
    assert_eq!(fixture.sources().len(), 3);
    // Low-level replacement accepts an already prepared runtime world, as the
    // level factory does; a raw empty world contains no plugin systems.
    let mut replacement = World::empty();
    fixture
        .runtime
        .handle()
        .resolve_driver::<scene::WorldDriver>(scene::WORLD_DRIVER_NAME)
        .unwrap()
        .apply_world_runtime_extensions(&mut replacement)
        .unwrap();
    fixture
        .level
        .replace_world_and_reset_runtime_state(replacement);
    assert!(fixture.manager.source_status(first).is_err());
    assert_eq!(fixture.sources().len(), 2);
    second_level.replace_world_and_reset_runtime_state(World::empty());
    assert_eq!(fixture.sources().len(), 1);
    add(&fixture.level, fixture.component());
    fixture.tick();
    assert_eq!(fixture.sources().len(), 2);
    fixture.end();
    assert_eq!(
        fixture.sources().len(),
        1,
        "retained Level must release only its scene sources"
    );
    assert_eq!(fixture.sources()[0].id, Some(manual));
    fixture.manager.remove_source(manual).unwrap();
}

#[test]
fn scene_audio_sessions_are_isolated_and_repeated_sessions_do_not_leak() {
    let first = Fixture::client();
    let second = Fixture::client();
    add(&first.level, first.component());
    add(&second.level, second.component());
    first.tick();
    second.tick();
    first.end();
    assert!(first.sources().is_empty());
    assert_eq!(second.sources().len(), 1);
    second.tick();
    assert_eq!(second.sources().len(), 1);
    second.end();
    for _ in 0..3 {
        let session = Fixture::client();
        add(&session.level, session.component());
        session.tick();
        assert_eq!(session.sources().len(), 1);
        session.end();
        assert!(session.sources().is_empty());
    }
}

#[test]
fn scene_audio_target_filter_runs_client_playback_and_keeps_editor_idle() {
    for (target, expected) in [
        (RuntimeTargetMode::ClientRuntime, true),
        (RuntimeTargetMode::EditorHost, false),
    ] {
        let registry = extensions(target);
        assert_eq!(
            registry.plugin_runtime_systems().any(|(owner, system)| {
                system.id == SYNC_SYSTEM
                    && registry.plugin_module_name(owner) == Some(PLAYBACK_MODULE)
            }),
            expected
        );
        assert!(registry
            .components()
            .iter()
            .any(|row| row.type_id == AUDIO_SOURCE_COMPONENT_TYPE));
        let fixture = Fixture::new(target);
        add(&fixture.level, fixture.component());
        fixture.tick();
        assert_eq!(!fixture.sources().is_empty(), expected);
        fixture.end();
    }
}

#[test]
fn scene_audio_changed_clip_replaces_source_and_invalid_input_releases_old_voice() {
    let fixture = Fixture::client();
    let entity = add(&fixture.level, fixture.component());
    fixture.tick();
    let first = fixture.sources()[0].id.unwrap();
    let second_clip = fixture
        .manager
        .insert_clip_for_test(super::test_clip_with_rate(
            "res://sound/other.wav",
            8,
            &[0.5; 8],
        ));
    set(
        &fixture.level,
        entity,
        json!({"input": {"Clip": second_clip}}),
    );
    fixture.tick();
    assert_eq!(fixture.sources().len(), 1);
    assert_ne!(fixture.sources()[0].id, Some(first));
    assert!(fixture.manager.source_status(first).is_err());
    for invalid in [
        json!({}),
        json!({"input": {"Clip": 999999}}),
        json!({"input": {"Clip": fixture.clip}, "speed": 0}),
        json!({"input": {"Clip": fixture.clip}, "gain": "loud"}),
    ] {
        set(&fixture.level, entity, invalid);
        assert!(fixture.try_tick().is_err());
        assert!(fixture.sources().is_empty());
    }
    set(&fixture.level, entity, fixture.component());
    fixture.tick();
    assert_eq!(fixture.sources().len(), 1);
    fixture.end();
}

#[test]
fn scene_audio_service_stop_does_not_replay_until_component_rearmed() {
    let fixture = Fixture::client();
    let entity = add(&fixture.level, fixture.component());
    fixture.tick();
    let id = fixture.sources()[0].id.unwrap();
    fixture.manager.stop_source(id).unwrap();
    for _ in 0..3 {
        fixture.tick();
        assert!(fixture.sources().is_empty());
    }
    assert_eq!(fixture.manager.drain_finished_sources().unwrap().len(), 1);
    let mut value = fixture.component();
    value["playing"] = json!(false);
    set(&fixture.level, entity, value);
    fixture.tick();
    set(&fixture.level, entity, fixture.component());
    fixture.tick();
    assert_eq!(fixture.sources().len(), 1);
    assert_ne!(fixture.sources()[0].id, Some(id));
    fixture.end();
}
