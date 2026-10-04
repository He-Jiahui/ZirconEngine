use super::*;
use crate::scene::components::{Hierarchy, LocalTransform};
use crate::scene::ecs::{
    ChangeTickWindow, Component, ComponentStorageLocation, Mut, QueryAccessError, QueryMutData,
    QueryState,
};
use crate::scene::SceneError;
use std::any::TypeId;
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

fn real_authored_location<T: Component>(
    world: &mut World,
    entity: u64,
) -> ComponentStorageLocation {
    let query = QueryState::<&T>::new(world);
    let component_id = world.registered_component_id::<T>().unwrap();
    for archetype in world.matching_query_archetypes(query.access()) {
        for row in 0..world.query_archetype_entity_count(archetype) {
            let stable = world.query_stable_location_at(archetype, row).unwrap();
            if stable.stable_id() == entity {
                let slot = world
                    .query_archetype_column_slot(archetype, component_id)
                    .unwrap();
                return ComponentStorageLocation::table(
                    component_id,
                    stable.internal,
                    archetype,
                    stable.location.table_row,
                    slot,
                )
                .with_rust_type_id(TypeId::of::<T>());
            }
        }
    }
    panic!("authored component must have a real query location")
}

#[test]
fn generic_authored_writes_reject_without_changing_scene() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    let generation = world.world_generation();
    let parent_before = world.parent_of(child);
    let transform_before = world.get::<LocalTransform>(child).copied();
    let mobility_before = world.mobility(child);
    assert_eq!(
        world.insert(
            child,
            Hierarchy {
                parent: Some(parent)
            }
        ),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "Hierarchy",
            operation: "insert",
        })
    );
    assert_eq!(
        world.insert(child, LocalTransform::default()),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "LocalTransform",
            operation: "insert",
        })
    );
    assert_eq!(
        world.insert(child, Mobility::Static),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "Mobility",
            operation: "insert",
        })
    );
    assert!(world.get_mut::<Hierarchy>(child).is_none());
    assert!(world.get_mut::<LocalTransform>(child).is_none());
    assert!(world.get_mut::<Mobility>(child).is_none());
    assert_eq!(
        world.remove::<Hierarchy>(child),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "Hierarchy",
            operation: "remove",
        })
    );
    assert_eq!(
        world.remove::<LocalTransform>(child),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "LocalTransform",
            operation: "remove",
        })
    );
    assert_eq!(
        world.remove::<Mobility>(child),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "Mobility",
            operation: "remove",
        })
    );
    assert_eq!(world.world_generation(), generation);
    assert_eq!(world.parent_of(child), parent_before);
    assert_eq!(
        world.get::<LocalTransform>(child).copied(),
        transform_before
    );
    assert_eq!(world.mobility(child), mobility_before);
}

#[test]
fn authored_mutable_queries_and_direct_fetches_fail_closed() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let hierarchy_location = real_authored_location::<Hierarchy>(&mut world, entity);
    let transform_location = real_authored_location::<LocalTransform>(&mut world, entity);
    let generation = world.world_generation();
    assert!(matches!(
        QueryState::<&mut Hierarchy>::try_new(&mut world),
        Err(QueryAccessError::ProtectedAuthoredComponentWrite {
            component: "Hierarchy"
        })
    ));
    assert!(matches!(
        QueryState::<Mut<'_, LocalTransform>>::try_new(&mut world),
        Err(QueryAccessError::ProtectedAuthoredComponentWrite {
            component: "LocalTransform"
        })
    ));
    assert!(matches!(
        QueryState::<Mut<'_, Mobility>>::try_new(&mut world),
        Err(QueryAccessError::ProtectedAuthoredComponentWrite {
            component: "Mobility"
        })
    ));
    // SAFETY: each call uses this exclusive World loan and its real current
    // bound location, and returns None without publishing a protected item.
    let ticks = ChangeTickWindow::all(world.read_change_tick());
    assert!(unsafe {
        <&mut Hierarchy as QueryMutData>::fetch_mut(&mut world as *mut World, entity)
    }
    .is_none());
    assert!(unsafe {
        <Mut<'_, LocalTransform> as QueryMutData>::fetch_mut_with_ticks(
            &mut world as *mut World,
            entity,
            ticks,
        )
    }
    .is_none());
    assert!(world.component_mut_with_ticks::<Mobility>(entity).is_none());
    assert!(unsafe {
        <&mut Hierarchy as QueryMutData>::fetch_mut_with_component_locations(
            &mut world as *mut World,
            entity,
            &[hierarchy_location],
            ticks,
        )
    }
    .is_none());
    assert!(unsafe {
        <Mut<'_, LocalTransform> as QueryMutData>::fetch_mut_with_component_locations(
            &mut world as *mut World,
            entity,
            &[transform_location],
            ticks,
        )
    }
    .is_none());
    assert!(unsafe {
        World::query_component_mut_at_location::<Hierarchy>(
            &mut world as *mut World,
            entity,
            hierarchy_location,
        )
    }
    .is_none());
    assert!(unsafe {
        World::query_component_mut_with_ticks_at_location::<LocalTransform>(
            &mut world as *mut World,
            entity,
            transform_location,
        )
    }
    .is_none());
    assert_eq!(world.world_generation(), generation);
    assert_eq!(world.parent_of(entity), None);
}

#[test]
fn scene_commands_retain_checked_authored_mutation() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    assert!(world.set_parent_checked(child, Some(parent)).unwrap());
    assert_eq!(world.parent_of(child), Some(parent));
    assert!(!world.set_parent_checked(child, Some(parent)).unwrap());
    assert!(world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap());
    assert_eq!(
        world
            .get::<LocalTransform>(child)
            .unwrap()
            .transform
            .translation,
        Vec3::new(2.0, 0.0, 0.0)
    );
    assert!(world.set_mobility(parent, Mobility::Static).unwrap());
    assert!(world.set_mobility(child, Mobility::Static).unwrap());
    assert_eq!(
        world.update_transform(child, Transform::default()),
        Err(SceneError::StaticTransformMutation { entity: child })
    );
}

#[test]
fn authored_bundle_and_deferred_writes_reject_atomically() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let generation = world.world_generation();
    assert_eq!(
        world.spawn((Hierarchy { parent: None },)),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "Hierarchy",
            operation: "insert",
        })
    );
    assert_eq!(
        world.insert_bundle(entity, (LocalTransform::default(),)),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "LocalTransform",
            operation: "insert",
        })
    );
    assert_eq!(world.world_generation(), generation);
    world.commands().insert(entity, Mobility::Static);
    let report = world.apply_deferred();
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::ProtectedAuthoredComponentMutation {
            component: "Mobility",
            operation: "insert",
        }
    );
    world.commands().remove::<Hierarchy>(entity);
    let report = world.apply_deferred();
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::ProtectedAuthoredComponentMutation {
            component: "Hierarchy",
            operation: "remove",
        }
    );
    assert_eq!(world.parent_of(entity), None);
}

#[test]
fn reflected_authored_writes_use_scene_checks_and_clone_into_preflight_world() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    let local = world
        .type_registry()
        .runtime_registration("LocalTransform")
        .unwrap()
        .component
        .clone()
        .unwrap();
    assert!(local
        .write_field(
            &mut world,
            child,
            "translation",
            ReflectedValue::Vec3([3.0, 0.0, 0.0])
        )
        .unwrap());
    assert!(!local
        .write_field_by_slot(&mut world, child, 0, ReflectedValue::Vec3([3.0, 0.0, 0.0]))
        .unwrap());
    assert!(local
        .write_fields_by_slot(
            &mut world,
            child,
            vec![(0, ReflectedValue::Vec3([4.0, 0.0, 0.0]))],
        )
        .unwrap());
    let mut preflight = World::empty();
    assert!(preflight.spawn_empty_at(child).unwrap());
    assert!(local.stage_clone(&world, child, &mut preflight).unwrap());
    assert_eq!(
        preflight.get::<LocalTransform>(child),
        world.get::<LocalTransform>(child)
    );

    let mobility = world
        .type_registry()
        .runtime_registration("Mobility")
        .unwrap()
        .component
        .clone()
        .unwrap();
    assert!(world.set_mobility(parent, Mobility::Static).unwrap());
    assert!(mobility
        .write_field_by_slot(
            &mut world,
            child,
            0,
            ReflectedValue::Enum("static".to_string()),
        )
        .unwrap());
    assert!(matches!(
        mobility.write_field(
            &mut world,
            parent,
            "kind",
            ReflectedValue::Enum("dynamic".to_string()),
        ),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert_eq!(world.mobility(parent), Some(Mobility::Static));
    assert_eq!(world.mobility(child), Some(Mobility::Static));
    assert!(matches!(
        local.write_field(
            &mut world,
            child,
            "translation",
            ReflectedValue::Vec3([9.0, 0.0, 0.0]),
        ),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    let hierarchy = world
        .type_registry()
        .runtime_registration("Hierarchy")
        .unwrap()
        .component
        .clone()
        .unwrap();
    assert!(matches!(
        hierarchy.remove(&mut world, child),
        Err(ReflectError::NonRemovableComponent { .. })
    ));
    assert_eq!(
        hierarchy.remove(&mut world, u64::MAX),
        Err(ReflectError::MissingEntity { entity: u64::MAX })
    );
}

#[test]
fn generic_hierarchy_reflection_cannot_bypass_checked_scene_authority() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    let adapter = crate::scene::derived_component_registration::<Hierarchy>()
        .unwrap()
        .component
        .unwrap();
    let generation = world.world_generation();
    assert!(matches!(
        adapter.write_field(
            &mut world,
            child,
            "parent",
            ReflectedValue::Entity(Some(parent)),
        ),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert_eq!(
        adapter.remove(&mut world, child),
        Err(ReflectError::NonRemovableComponent {
            type_path: "zircon_runtime::scene::components::Hierarchy".to_string(),
        })
    );
    assert_eq!(world.parent_of(child), None);
    assert_eq!(world.world_generation(), generation);
    assert!(world.set_parent_checked(child, Some(parent)).unwrap());
    assert_eq!(world.parent_of(child), Some(parent));
}
