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
