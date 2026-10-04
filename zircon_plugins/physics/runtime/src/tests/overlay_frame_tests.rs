use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::World;

use crate::runtime_system::physics_overlay_frame_if_enabled;
use crate::{
    plugin_registration, PhysicsDebugOverlayCapture, PhysicsOverlayFrame,
    PHYSICS_OVERLAY_FRAME_EVENT_ID, PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};

#[test]
fn physics_overlay_frame_uses_the_canonical_sync_snapshot_only_when_enabled() {
    let sync = PhysicsWorldSyncState {
        colliders: vec![collider(41, false)],
        ..PhysicsWorldSyncState::default()
    };
    let mut world = World::empty();
    world.insert_resource(PhysicsDebugOverlayCapture::default());

    assert!(physics_overlay_frame_if_enabled(&world, &sync).is_none());

    world.resource_mut::<PhysicsDebugOverlayCapture>().enabled = true;
    let frame = physics_overlay_frame_if_enabled(&world, &sync)
        .expect("an active overlay reader receives the current sync snapshot");
    assert_eq!(frame.owner_generation, world.world_generation());
    assert_eq!(frame.colliders, sync.colliders);
}

#[test]
fn physics_overlay_mirror_controls_capture_and_delivers_frames() {
    let mut report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let mut world = World::empty();
    report.extensions.apply_to_world(&mut world).unwrap();
    assert!(!world.resource::<PhysicsDebugOverlayCapture>().enabled);

    let mut subscription = world
        .subscribe_runtime_event_mirror(
            PHYSICS_OVERLAY_FRAME_EVENT_ID,
            PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
        )
        .unwrap();
    assert!(world.resource::<PhysicsDebugOverlayCapture>().enabled);

    world.send_event(PhysicsOverlayFrame {
        owner_generation: 7,
        colliders: vec![collider(41, true)],
    });
    world.update_events::<PhysicsOverlayFrame>();
    let payloads = world.drain_runtime_event_mirror(&mut subscription).unwrap();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["owner_generation"], 7);
    assert_eq!(payloads[0]["colliders"][0]["entity"], 41);

    assert!(world
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
    assert!(!world.resource::<PhysicsDebugOverlayCapture>().enabled);
}

fn collider(entity: u64, sensor: bool) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        entity,
        shape: PhysicsColliderShape::Sphere { radius: 0.5 },
        sensor,
        layer: 0,
        collision_group: u32::MAX,
        collision_mask: u32::MAX,
        material: None,
        material_override: None,
        transform: Transform::default(),
    }
}
