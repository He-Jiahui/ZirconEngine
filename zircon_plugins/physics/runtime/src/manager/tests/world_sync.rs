use super::*;

#[test]
fn owned_projection_preserves_nested_payloads() {
    let shape = ColliderShape::Compound {
        children: vec![(
            Transform::default(),
            Box::new(ColliderShape::ConvexHull {
                points: vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)],
            }),
        )],
    };

    let PhysicsColliderShape::Compound { children } = collider_shape_into_physics(shape) else {
        panic!("owned compound projection must preserve its variant");
    };
    let PhysicsColliderShape::ConvexHull { points } = children[0].1.as_ref() else {
        panic!("owned compound projection must preserve its child variant");
    };
    assert_eq!(points, &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
}

#[test]
fn typed_world_sync_ignores_ten_thousand_inert_entities() {
    use zircon_runtime::scene::{components::RigidBodyComponent, NodeKind};
    let mut world = World::empty();
    for _ in 0..10_000 {
        world.spawn_node(NodeKind::Empty).unwrap();
    }
    let body = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_rigid_body(body, Some(RigidBodyComponent::default()))
        .unwrap();
    let sync = build_world_sync_state(WorldHandle::new(71), &world);
    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, body);
    assert!(sync.colliders.is_empty());
    assert!(sync.joints.is_empty());
    assert!(sync.materials.is_empty());
}
