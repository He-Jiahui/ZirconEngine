use super::*;

#[test]
fn render_layer_mask_reflection_roundtrips_unsigned_mask() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let address = fixed_component_address(entity, "RenderLayerMask");

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id("zircon_runtime::scene::components::RenderLayerMask", "mask"),
            ReflectedValue::Unsigned(0x0000_00ff),
        ))
        .expect("render layer mask should be writable");

    assert!(response.changed);
    assert_eq!(
        world.get::<RenderLayerMask>(entity),
        Some(&RenderLayerMask(0xff))
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                address.clone(),
                reflected_field_id("zircon_runtime::scene::components::RenderLayerMask", "mask",),
            ))
            .expect("mask should read back")
            .field,
        ReflectFieldValue::new(
            reflected_field_id("zircon_runtime::scene::components::RenderLayerMask", "mask"),
            "mask",
            ReflectedValue::Unsigned(0xff),
        )
    );
    assert!(matches!(
        world.reflect_write(ReflectWriteRequest::new(
            address,
            reflected_field_id("zircon_runtime::scene::components::RenderLayerMask", "mask",),
            ReflectedValue::Unsigned(u32::MAX as u64 + 1),
        )),
        Err(ReflectError::TypeMismatch { .. })
    ));
}

#[test]
fn rigid_body_reflection_exposes_selected_safe_fields() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .insert(
            entity,
            RigidBodyComponent {
                body_type: RigidBodyType::Kinematic,
                linear_velocity: Vec3::new(1.0, 2.0, 3.0),
                lock_rotation: [true, false, true],
                ..RigidBodyComponent::default()
            },
        )
        .unwrap();
    let address = fixed_component_address(entity, "RigidBodyComponent");

    let fields = world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(address.clone()),
        )
        .expect("rigid body fields should be enumerable")
        .fields;

    assert!(fields.contains(&ReflectFieldValue::new(
        reflected_field_id(
            "zircon_runtime::scene::components::RigidBodyComponent",
            "body_type",
        ),
        "body_type",
        ReflectedValue::Enum("Kinematic".to_string())
    )));
    assert!(fields.contains(&ReflectFieldValue::new(
        reflected_field_id(
            "zircon_runtime::scene::components::RigidBodyComponent",
            "linear_velocity",
        ),
        "linear_velocity",
        ReflectedValue::Vec3([1.0, 2.0, 3.0])
    )));
    assert!(fields.contains(&ReflectFieldValue::new(
        reflected_field_id(
            "zircon_runtime::scene::components::RigidBodyComponent",
            "lock_rotation",
        ),
        "lock_rotation",
        ReflectedValue::List(vec![
            ReflectedValue::Bool(true),
            ReflectedValue::Bool(false),
            ReflectedValue::Bool(true),
        ])
    )));
    let rigid_schema = world.reflect_schema("RigidBodyComponent").unwrap();
    assert!(
        !rigid_schema
            .type_info
            .fields
            .iter()
            .find(|field| field.name == "body_type")
            .unwrap()
            .editable
    );
    let lock_rotation_schema = rigid_schema
        .type_info
        .fields
        .iter()
        .find(|field| field.name == "lock_rotation")
        .unwrap();
    assert!(!lock_rotation_schema.editable);
    assert_eq!(lock_rotation_schema.value_type_path, "List<Bool>");
    assert!(matches!(
        lock_rotation_schema.editor_hint,
        ReflectEditorHint::None
    ));
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "mass",
            ),
            ReflectedValue::Scalar(9.5),
        ))
        .expect("mass should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "linear_damping",
            ),
            ReflectedValue::Scalar(0.25),
        ))
        .expect("linear damping should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "angular_damping",
            ),
            ReflectedValue::Scalar(0.5),
        ))
        .expect("angular damping should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "gravity_scale",
            ),
            ReflectedValue::Scalar(0.75),
        ))
        .expect("gravity scale should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "sleep_policy",
            ),
            ReflectedValue::Enum("Never".to_string()),
        ))
        .expect("sleep_policy should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "mass_density",
            ),
            ReflectedValue::Scalar(4.0),
        ))
        .expect("mass density should select auto-from-shape mode");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "ccd_mode",
            ),
            ReflectedValue::Enum("LinearCast".to_string()),
        ))
        .expect("ccd_mode should be writable");

    let rigid_body = world.get::<RigidBodyComponent>(entity).unwrap();
    assert_eq!(rigid_body.mass, 9.5);
    assert_eq!(rigid_body.linear_damping, 0.25);
    assert_eq!(rigid_body.angular_damping, 0.5);
    assert_eq!(rigid_body.gravity_scale, 0.75);
    assert_eq!(
        rigid_body.sleep_policy,
        crate::core::framework::scene::physics::PhysicsSleepPolicy::Never
    );
    assert_eq!(
        rigid_body.mass_properties,
        crate::core::framework::scene::physics::PhysicsMassProperties::AutoFromShape {
            density: 4.0,
        }
    );
    assert_eq!(
        rigid_body.ccd_mode,
        crate::core::framework::scene::physics::PhysicsCcdMode::LinearCast
    );
    let unchanged_mass = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::RigidBodyComponent",
                "mass",
            ),
            ReflectedValue::Scalar(9.5),
        ))
        .expect("same rigid body mass should be accepted as unchanged");
    assert!(!unchanged_mass.changed);
    assert!(matches!(
        world.reflect_write(ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id("zircon_runtime::scene::components::RigidBodyComponent", "mass"),
            ReflectedValue::Scalar(f32::INFINITY),
        )),
        Err(ReflectError::TypeMismatch { expected, .. }) if expected == "finite Scalar"
    ));
    assert_eq!(
        world
            .reflect_write(ReflectWriteRequest::new(
                address,
                reflected_field_id(
                    "zircon_runtime::scene::components::RigidBodyComponent",
                    "linear_velocity",
                ),
                ReflectedValue::Vec3([0.0, 0.0, 0.0]),
            ))
            .expect_err("linear velocity is read-only"),
        ReflectError::NonEditableField {
            type_path: "zircon_runtime::scene::components::RigidBodyComponent".to_string(),
            field_name: "linear_velocity".to_string(),
        }
    );
}

#[test]
fn unknown_fixed_field_returns_structured_error() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");

    let missing_field_id = reflected_field_id("zircon_runtime::scene::components::Name", "missing");
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                fixed_component_address(entity, "Name"),
                missing_field_id,
            ))
            .expect_err("unknown fields should be structured"),
        ReflectError::UnknownField {
            type_path: "zircon_runtime::scene::components::Name".to_string(),
            field_name: missing_field_id.to_string(),
        }
    );
}

#[test]
fn missing_fixed_component_returns_structured_error() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.remove::<RigidBodyComponent>(entity).unwrap();

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                fixed_component_address(entity, "RigidBodyComponent"),
                reflected_field_id(
                    "zircon_runtime::scene::components::RigidBodyComponent",
                    "mass",
                ),
            ))
            .expect_err("missing fixed components should be structured"),
        ReflectError::MissingComponent {
            entity,
            type_path: "zircon_runtime::scene::components::RigidBodyComponent".to_string(),
        }
    );
    let rigid_body_adapter = world
        .type_registry()
        .runtime_registration("RigidBodyComponent")
        .expect("rigid body registration should resolve")
        .component
        .clone()
        .expect("rigid body registration should have component adapter");
    assert_eq!(
        rigid_body_adapter
            .remove(&mut world, entity)
            .expect_err("missing fixed component removals should be structured"),
        ReflectError::MissingComponent {
            entity,
            type_path: "zircon_runtime::scene::components::RigidBodyComponent".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                fixed_component_address(999_999, "Name"),
                reflected_field_id("zircon_runtime::scene::components::Name", "value"),
            ))
            .expect_err("missing entities should be structured"),
        ReflectError::MissingEntity { entity: 999_999 }
    );
}
