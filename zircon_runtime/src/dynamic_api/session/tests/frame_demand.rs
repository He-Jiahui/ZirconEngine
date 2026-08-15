use zircon_runtime_interface::{ZrRuntimeFrameDemandV1, ZrStatus, ZrStatusCode};

use crate::core::CoreError;
use crate::core::resource::{AnimationSequenceMarker, ResourceHandle, ResourceId};
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::state::RuntimeDynamicSession;
use crate::scene::components::{AnimationSequencePlayerComponent, NodeKind};
use crate::scene::{DefaultLevelManager, EntityId, SystemStage, World};

use super::super::ffi;
use super::super::registry::{
    RuntimeFrameDemand, destroy_session_slot, insert_session, with_session, with_session_activity,
};
use super::super::state::animation_frame_demand;

#[test]
fn animation_runtime_state_maps_to_idle_or_immediate_frame_demand() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());

    assert_eq!(animation_frame_demand(&level), RuntimeFrameDemand::Idle);

    level.record_animation_requires_continuous_frame(true);
    assert_eq!(
        animation_frame_demand(&level),
        RuntimeFrameDemand::Immediate
    );

    level.record_animation_requires_continuous_frame(false);
    assert_eq!(animation_frame_demand(&level), RuntimeFrameDemand::Idle);
}

#[test]
fn active_animation_tick_emits_immediate_then_paused_tick_resets_to_idle() {
    let (session, entity) = session_with_active_sequence();
    let handle = insert_session(session);

    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let status = unsafe { ffi::tick_frame(handle, &mut demand) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert_eq!(demand, ZrRuntimeFrameDemandV1::immediate());

    let pause_status = with_session(handle, |session| {
        session.level.with_world_mut(|world| {
            let mut player = world.animation_sequence_player(entity).unwrap().clone();
            player.playing = false;
            world
                .set_animation_sequence_player(entity, Some(player))
                .unwrap();
        });
        ZrStatus::ok()
    });
    assert_eq!(pause_status.status_code(), ZrStatusCode::Ok);

    let status = unsafe { ffi::tick_frame(handle, &mut demand) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert_eq!(demand, ZrRuntimeFrameDemandV1::idle());
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn failed_tick_does_not_publish_or_retain_animation_frame_demand() {
    let (session, _) = session_with_active_sequence();
    let mut registry = crate::plugin::RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("test.runtime").unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "test.frame-demand-failure",
            SystemStage::PostUpdate,
            || |_| Err(CoreError::RuntimeUnavailable),
        )
        .with_order(1)
        .register()
        .unwrap();
    let plan = registry.world_runtime_extension_plan().unwrap();
    session
        .level
        .with_world_mut(|world| plan.apply_to_world(world))
        .unwrap();
    let handle = insert_session(session);

    let expected_output = ZrRuntimeFrameDemandV1::after(123);
    let mut demand = expected_output;
    let status = unsafe { ffi::tick_frame(handle, &mut demand) };
    assert_ne!(status.status_code(), ZrStatusCode::Ok);
    assert_eq!(demand, expected_output);

    let activity_status = with_session_activity(handle, |session, activity| {
        assert_eq!(session.frame_demand(), RuntimeFrameDemand::Idle);
        assert_eq!(activity.consume_frame_demand(), RuntimeFrameDemand::Idle);
        ZrStatus::ok()
    });
    assert_eq!(activity_status.status_code(), ZrStatusCode::Ok);
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

fn session_with_active_sequence() -> (RuntimeDynamicSession, EntityId) {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let entity = session.level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_animation_sequence_player(
                entity,
                Some(AnimationSequencePlayerComponent {
                    sequence: ResourceHandle::<AnimationSequenceMarker>::new(
                        ResourceId::from_stable_label("active-frame.sequence"),
                    ),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });
    (session, entity)
}
