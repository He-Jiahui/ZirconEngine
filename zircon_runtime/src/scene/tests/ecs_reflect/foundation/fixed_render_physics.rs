//! 渲染层与刚体只向编辑器开放适配器允许的字段；不可编辑的物理状态须被反射入口拒绝。

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

#[test]
fn registered_reflection_remove_preserves_lifecycle_cancellation_kind_and_target() {
    use crate::scene::ecs::LifecycleEventKind;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };
    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    for schedule_only in [false, true] {
        for retain_schedule in [false, true] {
            let mut world = World::empty();
            let entity = world.spawn_node(NodeKind::Mesh).unwrap();
            world
                .insert(
                    entity,
                    RigidBodyComponent {
                        mass: 3.0,
                        ..Default::default()
                    },
                )
                .unwrap();
            let adapter = world
                .type_registry()
                .runtime_registration("RigidBodyComponent")
                .unwrap()
                .component
                .clone()
                .unwrap();
            let mut replacement = World::empty();
            assert_eq!(replacement.spawn_node(NodeKind::Mesh).unwrap(), entity);
            replacement
                .insert(
                    entity,
                    RigidBodyComponent {
                        mass: 29.0,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(
                world.registered_component_id::<RigidBodyComponent>(),
                replacement.registered_component_id::<RigidBodyComponent>()
            );
            let generation = replacement.world_generation();
            let original = world.schedule().stage_plan();
            let replacement = Arc::new(Mutex::new(Some(replacement)));
            let fresh_calls = Arc::new(AtomicUsize::new(0));
            let observed = Arc::clone(&fresh_calls);
            world.observe_component_lifecycle::<RigidBodyComponent>(
                LifecycleEventKind::Remove,
                // SAFETY: After World or schedule retirement, only fixed resources and counter-
                // only commands change; no replacement history is adopted.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                        if schedule_only {
                            *world.schedule_mut() = Default::default();
                        } else {
                            let mut fresh = replacement.lock().unwrap().take().unwrap();
                            if retain_schedule {
                                *fresh.schedule_mut() = std::mem::take(world.schedule_mut());
                                assert!(Arc::ptr_eq(&original, &fresh.schedule().stage_plan()));
                            }
                            *world = fresh;
                        }
                        let observed = Arc::clone(&observed);
                        world.commands().queue_fn(move |_| {
                            observed.fetch_add(1, Ordering::SeqCst);
                        });
                    })
                },
            );
            assert_eq!(
                adapter.remove(&mut world, entity),
                Err(ReflectError::LifecycleTargetReplaced {
                    operation: "remove component".into(),
                    target: if schedule_only {
                        LifecycleTargetChange::Schedule
                    } else {
                        LifecycleTargetChange::World
                    },
                })
            );
            assert_eq!(
                world.get::<RigidBodyComponent>(entity).unwrap().mass,
                if schedule_only { 3.0 } else { 29.0 }
            );
            if !schedule_only {
                assert_eq!(world.world_generation(), generation);
            }
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
            assert!(world.apply_deferred().is_success());
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        }
    }
}

#[test]
fn registered_reflection_writes_forward_typed_lifecycle_cancellation_and_stop_current_work() {
    use crate::scene::components::{Hierarchy, Name};
    use crate::scene::ecs::LifecycleEventKind;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };
    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    for hierarchy_write in [false, true] {
        for schedule_only in [false, true] {
            for retain_schedule in [false, true] {
                let mut world = World::empty();
                let entity = world.spawn_node(NodeKind::Empty).unwrap();
                let parent = world.spawn_node(NodeKind::Empty).unwrap();
                let mut replacement = World::empty();
                assert_eq!(replacement.spawn_node(NodeKind::Empty).unwrap(), entity);
                assert_eq!(replacement.spawn_node(NodeKind::Empty).unwrap(), parent);
                replacement
                    .insert(entity, Name("replacement row".into()))
                    .unwrap();
                let generation = replacement.world_generation();
                let original = world.schedule().stage_plan();
                let replacement = Arc::new(Mutex::new(Some(replacement)));
                let fresh_calls = Arc::new(AtomicUsize::new(0));
                let observed = Arc::clone(&fresh_calls);
                let callback =
                    move |world: &mut World, _: &crate::scene::ecs::ComponentLifecycleEvent| {
                        if schedule_only {
                            *world.schedule_mut() = Default::default();
                        } else {
                            let mut fresh = replacement.lock().unwrap().take().unwrap();
                            if retain_schedule {
                                *fresh.schedule_mut() = std::mem::take(world.schedule_mut());
                                assert!(Arc::ptr_eq(&original, &fresh.schedule().stage_plan()));
                            }
                            *world = fresh;
                        }
                        let observed = Arc::clone(&observed);
                        world.commands().queue_fn(move |_| {
                            observed.fetch_add(1, Ordering::SeqCst);
                        });
                    };
                let (component, type_path, field, value) = if hierarchy_write {
                    world.observe_component_lifecycle::<Hierarchy>(
                        LifecycleEventKind::Replace,
                        // SAFETY: After World or schedule retirement, only fixed resources and
                        // counter-only commands change; no replacement history is adopted.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                callback,
                            )
                        },
                    );
                    (
                        "Hierarchy",
                        "zircon_runtime::scene::components::Hierarchy",
                        "parent",
                        ReflectedValue::Entity(Some(parent)),
                    )
                } else {
                    world
                        .observe_component_lifecycle::<Name>(
                            LifecycleEventKind::Replace,
                            // SAFETY: After World or schedule retirement, only fixed resources
                            // and counter-only commands change; no replacement history is
                            // adopted.
                            unsafe {
                                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                    callback,
                                )
                            },
                        );
                    (
                        "Name",
                        "zircon_runtime::scene::components::Name",
                        "value",
                        ReflectedValue::String("edited row".into()),
                    )
                };
                let error = world
                    .reflect_write(ReflectWriteRequest::new(
                        fixed_component_address(entity, component),
                        reflected_field_id(type_path, field),
                        value,
                    ))
                    .expect_err("the real component callback retired its target");
                let expected = if schedule_only {
                    LifecycleTargetChange::Schedule
                } else {
                    LifecycleTargetChange::World
                };
                assert_eq!(error.lifecycle_target_change(), Some(expected));
                assert!(
                    matches!(error, ReflectError::LifecycleTargetReplaced { target, .. } if target == expected)
                );
                if !schedule_only {
                    assert_eq!(world.world_generation(), generation);
                    assert_eq!(world.get::<Name>(entity).unwrap().0, "replacement row");
                    assert_eq!(world.parent_of(entity), None);
                }
                assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
                assert!(world.apply_deferred().is_success());
                assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
            }
        }
    }
}
