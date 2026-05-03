use super::*;

#[test]
fn integrate_builtin_physics_steps_ignores_non_finite_step_seconds() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::X,
                    angular_velocity: Vec3::Y,
                    gravity_scale: 1.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        integrate_builtin_physics_steps(
            world,
            PhysicsWorldStepPlan {
                steps: 1,
                step_seconds: f32::NAN,
                remaining_seconds: 0.0,
            },
        );
        body
    });

    let (transform, rigid_body) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform,
            world.rigid_body(body).unwrap().clone(),
        )
    });
    assert_eq!(transform.translation, Vec3::ZERO);
    assert_eq!(transform.rotation, Transform::identity().rotation);
    assert_eq!(rigid_body.linear_velocity, Vec3::X);
    assert_eq!(rigid_body.angular_velocity, Vec3::Y);
}

#[test]
fn integrate_builtin_physics_steps_ignores_non_finite_body_velocity() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::new(f32::NAN, 1.0, 0.0),
                    angular_velocity: Vec3::new(0.0, f32::INFINITY, 0.0),
                    gravity_scale: 0.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        integrate_builtin_physics_steps(
            world,
            PhysicsWorldStepPlan {
                steps: 1,
                step_seconds: 1.0 / 60.0,
                remaining_seconds: 0.0,
            },
        );
        body
    });

    let (transform, rigid_body) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform,
            world.rigid_body(body).unwrap().clone(),
        )
    });
    assert_eq!(transform.translation, Vec3::ZERO);
    assert_eq!(transform.rotation, Transform::identity().rotation);
    assert!(rigid_body.linear_velocity.x.is_nan());
    assert_eq!(rigid_body.linear_velocity.y, 1.0);
    assert_eq!(rigid_body.linear_velocity.z, 0.0);
    assert_eq!(rigid_body.angular_velocity.x, 0.0);
    assert_eq!(rigid_body.angular_velocity.y, f32::INFINITY);
    assert_eq!(rigid_body.angular_velocity.z, 0.0);
}
