use serde_json::json;
use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::core::framework::sound::{
    SoundOutputDeviceManager, SoundOutputDeviceState, SoundPlayRequests, SoundPlaybackManager,
    SoundSourceDescriptor, SoundSourceInput, SoundSourceManager, AUDIO_SOURCE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::{self, LevelSystem, NodeKind, World};
use zircon_runtime::script::{
    register_gameplay_host_module, with_script_runtime_test_context, CapabilitySet,
    HostExportRegistry, HostRegistry, ScriptRuntimeTestContext, VmError,
};

use super::scene_support::{add, Fixture};

const URI: &str = "res://sound/host.wav";

fn fixture() -> Fixture {
    let fixture = Fixture::client();
    fixture
        .manager
        .cache_clip_for_test(super::test_clip_with_rate(URI, 8, &[0.25; 8]));
    fixture
}

fn entity(level: &LevelSystem) -> u64 {
    level.with_world_mut(|world| world.spawn_node(NodeKind::Empty).unwrap())
}

fn call(
    fixture: &Fixture,
    level: &LevelSystem,
    entity: u64,
    uri: &str,
    allowed: bool,
) -> Result<ScriptHostValue, VmError> {
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = if allowed {
        CapabilitySet::default().with("gameplay.entity")
    } else {
        CapabilitySet::default()
    };
    with_script_runtime_test_context(
        ScriptRuntimeTestContext::new(fixture.runtime.weak(), level.clone(), entity, 0.1),
        || {
            exports.call_with_capabilities(
                "zr.zircon.gameplay",
                "play_sound",
                vec![
                    ScriptHostValue::Int(entity as i64),
                    ScriptHostValue::String(uri.to_owned()),
                ],
                &capabilities,
            )
        },
    )
}

#[test]
fn host_play_sound_creates_headless_transient_at_entity_world_position() {
    let fixture = fixture();
    let entity = entity(&fixture.level);
    fixture.level.with_world_mut(|world| {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        let mut transform = Transform::default();
        transform.translation = Vec3::new(10.0, 2.0, 0.0);
        world.update_transform(parent, transform).unwrap();
        world.set_parent_checked(entity, Some(parent)).unwrap();
        let mut transform = Transform::default();
        transform.translation = Vec3::new(1.0, 0.0, 3.0);
        world.update_transform(entity, transform).unwrap();
    });
    assert_eq!(
        call(&fixture, &fixture.level, entity, URI, true).unwrap(),
        ScriptHostValue::Bool(true)
    );
    assert!(
        fixture.sources().is_empty(),
        "host acceptance precedes PostUpdate sync"
    );
    fixture.tick();
    let sources = fixture.sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].position, [11.0, 2.0, 3.0]);
    assert_eq!(
        sources[0].input,
        SoundSourceInput::Clip(fixture.manager.load_clip(URI).unwrap())
    );
    assert!(
        fixture
            .manager
            .source_status(sources[0].id.unwrap())
            .unwrap()
            .playing
    );
    assert!(!sources[0].looped);
    assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 1);
    fixture.level.with_world(|world| {
        assert!(world
            .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
            .is_none());
        assert!(world
            .get_resource::<SoundPlayRequests>()
            .unwrap()
            .0
            .is_empty());
    });
    assert_eq!(
        fixture.manager.output_device_status().unwrap().state,
        SoundOutputDeviceState::Stopped
    );
    fixture.end();
    assert!(fixture.sources().is_empty());
    assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 0);
}

#[test]
fn host_play_sound_retriggers_existing_audio_source_and_preserves_authored_settings() {
    let fixture = fixture();
    let entity = add(
        &fixture.level,
        json!({
            "input": {"Clip": fixture.clip}, "gain": 0.4, "speed": 1.5, "looped": true
        }),
    );
    fixture.tick();
    let first = fixture.sources()[0].id.unwrap();
    for _ in 0..3 {
        call(&fixture, &fixture.level, entity, URI, true).unwrap();
    }
    fixture.tick();
    let source = fixture.sources().pop().unwrap();
    assert_eq!(
        fixture.sources().len(),
        1,
        "existing AudioSource has one voice"
    );
    assert_ne!(source.id, Some(first));
    assert!(fixture.manager.source_status(first).is_err());
    assert_eq!(source.gain, 0.4);
    assert_eq!(source.speed, 1.5);
    assert!(source.looped);
    assert_eq!(
        source.input,
        SoundSourceInput::Clip(fixture.manager.load_clip(URI).unwrap())
    );
    fixture.manager.complete_source_for_test(source.id.unwrap());
    fixture.tick();
    assert!(
        fixture.sources().is_empty(),
        "normal ticks must not replay completion"
    );
    call(&fixture, &fixture.level, entity, URI, true).unwrap();
    fixture.tick();
    assert_eq!(
        fixture.sources().len(),
        1,
        "same clip can be explicitly retriggered"
    );
    assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 1);
    fixture.end();
}

#[test]
fn host_play_sound_rearms_stopped_disabled_audio_source_without_adding_component() {
    let fixture = fixture();
    let entity = add(
        &fixture.level,
        json!({
            "enabled": false, "playing": false, "gain": 0.25, "custom": "keep"
        }),
    );
    call(&fixture, &fixture.level, entity, URI, true).unwrap();
    fixture.tick();
    assert_eq!(fixture.sources().len(), 1);
    fixture.level.with_world(|world| {
        let value = world
            .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
            .unwrap();
        assert_eq!(value["enabled"], true);
        assert_eq!(value["playing"], true);
        assert_eq!(value["custom"], "keep");
    });
    assert_eq!(fixture.sources()[0].gain, 0.25);
    fixture.end();
}

#[test]
fn host_play_sound_invalid_uri_entity_and_capability_fail_without_side_effects() {
    let fixture = fixture();
    let entity = add(&fixture.level, fixture.component());
    fixture.tick();
    let before = fixture.sources();
    let component = fixture.level.with_world(|world| {
        world
            .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
            .cloned()
    });
    for uri in [
        "",
        "not a URI",
        "res://sound/missing.wav",
        "res://sound/../escape.wav",
    ] {
        assert!(
            call(&fixture, &fixture.level, entity, uri, true).is_err(),
            "{uri}"
        );
    }
    assert!(call(&fixture, &fixture.level, 999_999, URI, true).is_err());
    assert!(call(&fixture, &fixture.level, entity, URI, false)
        .unwrap_err()
        .to_string()
        .contains("missing capability"));
    fixture.level.with_world(|world| {
        assert!(world.get_resource::<SoundPlayRequests>().is_none());
        assert_eq!(
            world
                .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
                .cloned(),
            component
        );
    });
    fixture.tick();
    assert_eq!(fixture.sources(), before);
    fixture.end();
}

#[test]
fn host_play_sound_transients_release_on_completion_stop_disable_and_entity_removal() {
    let fixture = fixture();
    for removal in 0..4 {
        let entity = entity(&fixture.level);
        call(&fixture, &fixture.level, entity, URI, true).unwrap();
        fixture.tick();
        let source = fixture.sources()[0].id.unwrap();
        match removal {
            0 => fixture.manager.complete_source_for_test(source),
            1 => fixture.manager.stop_source(source).unwrap(),
            2 => fixture.level.with_world_mut(|world| {
                world.set_active_self(entity, false).unwrap();
            }),
            _ => fixture
                .level
                .with_world_mut(|world| world.remove_entity(entity).unwrap()),
        }
        fixture.tick();
        assert!(fixture.sources().is_empty());
        assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 0);
    }
    assert_eq!(
        fixture.manager.drain_finished_sources().unwrap().len(),
        2,
        "scene cleanup does not consume finished events"
    );
    fixture.end();
}

#[test]
fn host_play_sound_world_and_session_cleanup_preserve_other_owners_and_discard_pending() {
    let fixture = fixture();
    let manual = fixture
        .manager
        .create_source(SoundSourceDescriptor::clip(fixture.clip))
        .unwrap();
    let first = entity(&fixture.level);
    call(&fixture, &fixture.level, first, URI, true).unwrap();
    fixture.tick();
    let second_level = scene::create_level(
        &fixture.runtime.handle(),
        World::empty(),
        Default::default(),
    )
    .unwrap();
    let second = entity(&second_level);
    call(&fixture, &second_level, second, URI, true).unwrap();
    let frame = fixture
        .runtime
        .advance_time_by(std::time::Duration::from_millis(100), 1);
    second_level.tick(&fixture.runtime.handle(), frame).unwrap();
    fixture
        .level
        .tick(&fixture.runtime.handle(), frame)
        .unwrap();
    assert_eq!(fixture.sources().len(), 3);
    call(&fixture, &fixture.level, first, URI, true).unwrap();
    fixture
        .level
        .replace_world_and_reset_runtime_state(World::empty());
    assert_eq!(fixture.sources().len(), 2);
    fixture
        .level
        .with_world(|world| assert!(world.get_resource::<SoundPlayRequests>().is_none()));
    assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 1);
    fixture.end();
    assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 0);
    assert_eq!(fixture.sources().len(), 1);
    assert_eq!(fixture.sources()[0].id, Some(manual));
    fixture.manager.remove_source(manual).unwrap();
}

#[test]
fn host_play_sound_removed_or_inactive_entity_cannot_start_queued_playback() {
    let fixture = fixture();
    for remove in [false, true] {
        let entity = entity(&fixture.level);
        call(&fixture, &fixture.level, entity, URI, true).unwrap();
        fixture.level.with_world_mut(|world| {
            if remove {
                world.remove_entity(entity).unwrap();
            } else {
                world.set_active_self(entity, false).unwrap();
            }
        });
        fixture.tick();
        assert!(fixture.sources().is_empty());
        assert_eq!(fixture.manager.owned_scene_source_count_for_test(), 0);
    }
    fixture.end();
}
