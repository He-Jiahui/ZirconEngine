use super::*;

#[test]
fn world_resolves_entity_paths_and_mutates_component_properties() {
    let mut world = World::new();
    let root = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    world.rename_node(root, "Root").unwrap();

    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .update_transform(hero, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    world
        .set_rigid_body(
            hero,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                mass: 2.5,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    world
        .set_animation_player(
            hero,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "res://animation/hero.clip.zranim",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.25,
                looping: true,
                playing: true,
            }),
        )
        .unwrap();

    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let translation_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let mass_path = ComponentPropertyPath::parse("RigidBody.mass").unwrap();
    let mass_mode_path = ComponentPropertyPath::parse("RigidBody.mass_properties.mode").unwrap();
    let mass_density_path =
        ComponentPropertyPath::parse("RigidBody.mass_properties.density").unwrap();
    let ccd_mode_path = ComponentPropertyPath::parse("RigidBody.ccd_mode").unwrap();
    let sleep_policy_path = ComponentPropertyPath::parse("RigidBody.sleep_policy").unwrap();
    let weight_path = ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap();
    let render_queue_path = ComponentPropertyPath::parse("MeshRenderer.render_queue").unwrap();
    let material_queue_path = ComponentPropertyPath::parse("MeshRenderer.material_queue").unwrap();
    let order_path = ComponentPropertyPath::parse("MeshRenderer.order_in_layer").unwrap();
    let depth_bias_path = ComponentPropertyPath::parse("MeshRenderer.depth_bias").unwrap();
    let morph_weight_path = ComponentPropertyPath::parse("MeshRenderer.morph_weights.1").unwrap();

    assert_eq!(world.entity_path(hero), Some(entity_path.clone()));
    assert_eq!(world.get_entity_by_path(&entity_path), Some(hero));
    assert_eq!(
        world.property(hero, &translation_path).unwrap(),
        ScenePropertyValue::Vec3([1.0, 2.0, 3.0])
    );
    assert_eq!(
        world.property(hero, &mass_path).unwrap(),
        ScenePropertyValue::Scalar(2.5)
    );
    assert_eq!(
        world.property(hero, &mass_mode_path).unwrap(),
        ScenePropertyValue::Enum("explicit".to_string())
    );
    assert_eq!(
        world.property(hero, &weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.25)
    );
    assert_eq!(
        world.property(hero, &render_queue_path).unwrap(),
        ScenePropertyValue::Integer(0)
    );
    assert_eq!(
        world.property(hero, &material_queue_path).unwrap(),
        ScenePropertyValue::Integer(0)
    );
    assert_eq!(
        world.property(hero, &order_path).unwrap(),
        ScenePropertyValue::Integer(0)
    );
    assert_eq!(
        world.property(hero, &depth_bias_path).unwrap(),
        ScenePropertyValue::Scalar(0.0)
    );

    assert!(world
        .set_property(
            hero,
            &translation_path,
            ScenePropertyValue::Vec3([4.0, 5.0, 6.0]),
        )
        .unwrap());
    assert!(world
        .set_property(hero, &mass_path, ScenePropertyValue::Scalar(5.5))
        .unwrap());
    assert!(world
        .set_property(hero, &mass_density_path, ScenePropertyValue::Scalar(3.25),)
        .unwrap());
    assert!(world
        .set_property(
            hero,
            &ccd_mode_path,
            ScenePropertyValue::Enum("linear_cast".to_string()),
        )
        .unwrap());
    assert!(world
        .set_property(
            hero,
            &sleep_policy_path,
            ScenePropertyValue::Enum("never".to_string()),
        )
        .unwrap());
    assert!(world
        .set_property(hero, &weight_path, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    assert!(!world
        .set_property(hero, &weight_path, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    assert!(world
        .set_property(hero, &render_queue_path, ScenePropertyValue::Integer(2_450))
        .unwrap());
    assert!(world
        .set_property(hero, &material_queue_path, ScenePropertyValue::Integer(-12))
        .unwrap());
    assert!(world
        .set_property(hero, &order_path, ScenePropertyValue::Integer(14))
        .unwrap());
    assert!(!world
        .set_property(hero, &order_path, ScenePropertyValue::Integer(14))
        .unwrap());
    assert!(world
        .set_property(hero, &depth_bias_path, ScenePropertyValue::Scalar(-0.5))
        .unwrap());
    assert!(!world
        .set_property(hero, &depth_bias_path, ScenePropertyValue::Scalar(-0.5))
        .unwrap());
    assert!(world
        .set_property(hero, &morph_weight_path, ScenePropertyValue::Scalar(0.6))
        .unwrap());

    let node = world.find_node(hero).unwrap();
    assert_eq!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(world.rigid_body(hero).unwrap().mass, 5.5);
    assert_eq!(
        world.rigid_body(hero).unwrap().mass_properties,
        crate::core::framework::scene::physics::PhysicsMassProperties::AutoFromShape {
            density: 3.25,
        }
    );
    assert_eq!(
        world.rigid_body(hero).unwrap().ccd_mode,
        crate::core::framework::scene::physics::PhysicsCcdMode::LinearCast
    );
    assert_eq!(
        world.rigid_body(hero).unwrap().sleep_policy,
        crate::core::framework::scene::physics::PhysicsSleepPolicy::Never
    );
    assert_eq!(world.animation_player(hero).unwrap().weight, 0.75);
    let mesh = world.get::<MeshRenderer>(hero).unwrap();
    assert_eq!(mesh.render_queue, 2_450);
    assert_eq!(mesh.material_queue, -12);
    assert_eq!(mesh.order_in_layer, 14);
    assert_eq!(mesh.depth_bias, -0.5);
    assert_eq!(mesh.morph_weights.as_slice(), &[0.0, 0.6]);
    assert_eq!(
        world.property(hero, &translation_path).unwrap(),
        ScenePropertyValue::Vec3([4.0, 5.0, 6.0])
    );
    assert_eq!(
        world.property(hero, &mass_path).unwrap(),
        ScenePropertyValue::Scalar(5.5)
    );
    assert_eq!(
        world.property(hero, &mass_mode_path).unwrap(),
        ScenePropertyValue::Enum("auto_from_shape".to_string())
    );
    assert_eq!(
        world.property(hero, &mass_density_path).unwrap(),
        ScenePropertyValue::Scalar(3.25)
    );
    assert_eq!(
        world.property(hero, &ccd_mode_path).unwrap(),
        ScenePropertyValue::Enum("linear_cast".to_string())
    );
    assert_eq!(
        world.property(hero, &sleep_policy_path).unwrap(),
        ScenePropertyValue::Enum("never".to_string())
    );
    assert_eq!(
        world.property(hero, &weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.75)
    );
    assert_eq!(
        world.property(hero, &render_queue_path).unwrap(),
        ScenePropertyValue::Integer(2_450)
    );
    assert_eq!(
        world.property(hero, &material_queue_path).unwrap(),
        ScenePropertyValue::Integer(-12)
    );
    assert_eq!(
        world.property(hero, &order_path).unwrap(),
        ScenePropertyValue::Integer(14)
    );
    assert_eq!(
        world.property(hero, &depth_bias_path).unwrap(),
        ScenePropertyValue::Scalar(-0.5)
    );
    assert_eq!(
        world.property(hero, &morph_weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.6)
    );
}

#[test]
fn world_property_writes_reject_unrepresentable_or_overflowing_values_atomically() {
    let mut world = World::new();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let original_render_mask = world
        .get::<crate::scene::components::RenderLayerMask>(entity)
        .expect("spawned mesh has its default render-layer component")
        .0;

    let morph_index_path =
        ComponentPropertyPath::parse(&format!("MeshRenderer.morph_weights.{}", usize::MAX))
            .unwrap();
    let error = world
        .set_property(entity, &morph_index_path, ScenePropertyValue::Scalar(0.5))
        .expect_err("usize::MAX cannot be converted to a vector length");
    assert!(matches!(
        error,
        crate::scene::SceneError::InvalidPropertyIndex { .. }
    ));
    assert!(world
        .get::<MeshRenderer>(entity)
        .unwrap()
        .morph_weights
        .is_empty());

    let render_layer_path = ComponentPropertyPath::parse("RenderLayer.mask").unwrap();
    let error = world
        .set_property(
            entity,
            &render_layer_path,
            ScenePropertyValue::Unsigned(u64::from(u32::MAX) + 1),
        )
        .expect_err("u64 values outside u32 must be rejected");
    assert!(matches!(
        error,
        crate::scene::SceneError::PropertyTypeMismatch { .. }
    ));
    assert_eq!(
        world
            .get::<crate::scene::components::RenderLayerMask>(entity)
            .expect("rejected property write must preserve the render-layer component")
            .0,
        original_render_mask
    );
}

#[test]
fn world_optional_physics_property_errors_do_not_materialize_defaults() {
    let mut world = World::new();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .set_collider(
            entity,
            Some(crate::scene::components::ColliderComponent::default()),
        )
        .unwrap();
    world
        .set_joint(
            entity,
            Some(crate::scene::components::JointComponent::default()),
        )
        .unwrap();

    let bad_override_path =
        ComponentPropertyPath::parse("Collider.material_override.unknown").unwrap();
    assert!(world
        .set_property(entity, &bad_override_path, ScenePropertyValue::Scalar(0.5),)
        .is_err());
    assert!(world
        .get::<crate::scene::components::ColliderComponent>(entity)
        .unwrap()
        .material_override
        .is_none());

    let invalid_override_path =
        ComponentPropertyPath::parse("Collider.material_override.static_friction").unwrap();
    assert!(world
        .set_property(
            entity,
            &invalid_override_path,
            ScenePropertyValue::String("invalid".to_string()),
        )
        .is_err());
    assert!(world
        .get::<crate::scene::components::ColliderComponent>(entity)
        .unwrap()
        .material_override
        .is_none());

    let unknown_limit_path = ComponentPropertyPath::parse("Joint.limits.unknown").unwrap();
    assert!(world
        .set_property(entity, &unknown_limit_path, ScenePropertyValue::Scalar(1.0),)
        .is_err());
    assert!(world
        .get::<crate::scene::components::JointComponent>(entity)
        .unwrap()
        .limits
        .is_none());

    let default_limit_path = ComponentPropertyPath::parse("Joint.limits.min").unwrap();
    assert!(!world
        .set_property(entity, &default_limit_path, ScenePropertyValue::Scalar(0.0),)
        .unwrap());
    assert!(world
        .get::<crate::scene::components::JointComponent>(entity)
        .unwrap()
        .limits
        .is_none());
}

#[test]
fn world_entity_paths_suffix_duplicate_sibling_names() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    world.rename_node(root, "Root").unwrap();
    let first = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.rename_node(first, "Hero").unwrap();
    world.set_parent_checked(first, Some(root)).unwrap();
    let second = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.rename_node(second, "Hero").unwrap();
    world.set_parent_checked(second, Some(root)).unwrap();

    let first_path = EntityPath::parse(&format!("Root/Hero#{first}")).unwrap();
    let second_path = EntityPath::parse(&format!("Root/Hero#{second}")).unwrap();

    assert_eq!(world.entity_path(first), Some(first_path.clone()));
    assert_eq!(world.entity_path(second), Some(second_path.clone()));
    assert_eq!(world.get_entity_by_path(&first_path), Some(first));
    assert_eq!(world.get_entity_by_path(&second_path), Some(second));
    assert_eq!(
        world.get_entity_by_path(&EntityPath::parse("Root/Hero").unwrap()),
        None
    );
}

#[test]
fn world_rejects_zero_length_transform_rotation_property_writes() {
    let mut world = World::new();
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let rotation_path = ComponentPropertyPath::parse("Transform.rotation").unwrap();
    let rotation_w_path = ComponentPropertyPath::parse("Transform.rotation.w").unwrap();

    let error = world
        .set_property(
            hero,
            &rotation_path,
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 0.0]),
        )
        .unwrap_err();
    assert!(
        matches!(error, crate::scene::SceneError::ZeroLengthQuaternion { .. }),
        "{error}"
    );
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );

    let error = world
        .set_property(hero, &rotation_w_path, ScenePropertyValue::Scalar(0.0))
        .unwrap_err();
    assert!(
        matches!(error, crate::scene::SceneError::ZeroLengthQuaternion { .. }),
        "{error}"
    );
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );
}

#[test]
fn world_rejects_non_finite_transform_property_writes() {
    let mut world = World::new();
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let translation_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let translation_x_path = ComponentPropertyPath::parse("Transform.translation.x").unwrap();
    let scale_path = ComponentPropertyPath::parse("Transform.scale").unwrap();

    let error = world
        .set_property(
            hero,
            &translation_path,
            ScenePropertyValue::Vec3([f32::NAN, 1.0, 2.0]),
        )
        .unwrap_err();
    assert!(
        matches!(
            error,
            crate::scene::SceneError::NonFinitePropertyValue { .. }
        ),
        "{error}"
    );
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );

    let error = world
        .set_property(
            hero,
            &translation_x_path,
            ScenePropertyValue::Scalar(f32::INFINITY),
        )
        .unwrap_err();
    assert!(
        matches!(
            error,
            crate::scene::SceneError::NonFinitePropertyValue { .. }
        ),
        "{error}"
    );
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );

    let error = world
        .set_property(
            hero,
            &scale_path,
            ScenePropertyValue::Vec3([1.0, f32::NEG_INFINITY, 1.0]),
        )
        .unwrap_err();
    assert!(
        matches!(
            error,
            crate::scene::SceneError::NonFinitePropertyValue { .. }
        ),
        "{error}"
    );
    assert_eq!(world.find_node(hero).unwrap().transform.scale, Vec3::ONE);
}
