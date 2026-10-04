use super::*;
use crate::scene::components::{ActiveInHierarchy, WorldMatrix};
use crate::scene::ecs::{
    ChangeTickWindow, Component, ComponentStorageLocation, DeferredCommandOperation, Mut,
    QueryAccessError, QueryMutData, QueryState,
};
use crate::scene::SceneError;
use std::any::TypeId;
use zircon_runtime_interface::reflect::ReflectError;

#[derive(Debug, PartialEq, Eq)]
struct Unrelated(u32);

impl Component for Unrelated {}

fn clean_derived_world() -> (World, u64) {
    let mut world = World::new();
    world.flush_pending_scene_systems();
    let entity = world.active_camera();
    assert!(world.get::<WorldMatrix>(entity).is_some());
    assert!(world.get::<ActiveInHierarchy>(entity).is_some());
    (world, entity)
}

fn assert_derived_unchanged(
    world: &World,
    entity: u64,
    matrix: WorldMatrix,
    active: ActiveInHierarchy,
) {
    assert_eq!(world.get::<WorldMatrix>(entity), Some(&matrix));
    assert_eq!(world.get::<ActiveInHierarchy>(entity), Some(&active));
    assert_eq!(world.world_matrix(entity), Some(matrix.0));
    assert_eq!(world.active_in_hierarchy(entity), Some(active.0));
}

fn real_derived_location<T: Component>(world: &mut World, entity: u64) -> ComponentStorageLocation {
    let query = QueryState::<&T>::new(world);
    let component_id = world.registered_component_id::<T>().unwrap();
    for archetype in world.matching_query_archetypes(query.access()) {
        for row in 0..world.query_archetype_entity_count(archetype) {
            let stable = world.query_stable_location_at(archetype, row).unwrap();
            if stable.stable_id() == entity {
                let column_slot = world
                    .query_archetype_column_slot(archetype, component_id)
                    .unwrap();
                let location = ComponentStorageLocation::table(
                    component_id,
                    stable.internal,
                    archetype,
                    stable.location.table_row,
                    column_slot,
                )
                .with_rust_type_id(TypeId::of::<T>());
                assert!(world
                    .component_ref_with_ticks_at_location::<T>(location)
                    .is_some());
                return location;
            }
        }
    }
    panic!("derived entity must have a real query storage location")
}

#[test]
fn generic_world_writes_and_removals_reject_protected_derived_rows_without_side_effects() {
    let (mut world, entity) = clean_derived_world();
    let matrix = *world.get::<WorldMatrix>(entity).unwrap();
    let active = *world.get::<ActiveInHierarchy>(entity).unwrap();
    let generation = world.world_generation();

    assert_eq!(
        world.insert(entity, WorldMatrix::default()),
        Err(SceneError::ProtectedDerivedComponentMutation {
            component: "WorldMatrix",
            operation: "insert",
        })
    );
    assert_eq!(
        world.insert(entity, ActiveInHierarchy(!active.0)),
        Err(SceneError::ProtectedDerivedComponentMutation {
            component: "ActiveInHierarchy",
            operation: "insert",
        })
    );
    assert!(world.get_mut::<WorldMatrix>(entity).is_none());
    assert!(world.get_mut::<ActiveInHierarchy>(entity).is_none());
    assert_eq!(
        world.remove::<WorldMatrix>(entity),
        Err(SceneError::ProtectedDerivedComponentMutation {
            component: "WorldMatrix",
            operation: "remove",
        })
    );
    assert_eq!(
        world.remove::<ActiveInHierarchy>(entity),
        Err(SceneError::ProtectedDerivedComponentMutation {
            component: "ActiveInHierarchy",
            operation: "remove",
        })
    );
    assert_eq!(world.world_generation(), generation);
    assert_derived_unchanged(&world, entity, matrix, active);
}

#[test]
fn mutable_query_access_rejects_both_forms_but_read_queries_continue_to_work() {
    let (mut world, entity) = clean_derived_world();
    let generation = world.world_generation();
    assert!(matches!(
        QueryState::<&mut WorldMatrix>::try_new(&mut world),
        Err(QueryAccessError::ProtectedDerivedComponentWrite {
            component: "WorldMatrix"
        })
    ));
    assert!(matches!(
        QueryState::<Mut<'_, ActiveInHierarchy>>::try_new(&mut world),
        Err(QueryAccessError::ProtectedDerivedComponentWrite {
            component: "ActiveInHierarchy"
        })
    ));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = world.query::<&mut WorldMatrix>();
    }))
    .is_err());
    assert!(world
        .component_mut_with_ticks::<WorldMatrix>(entity)
        .is_none());
    assert_eq!(world.world_generation(), generation);

    let mut read = QueryState::<&WorldMatrix>::new(&mut world);
    assert_eq!(
        read.get(&world, entity).unwrap(),
        world.get::<WorldMatrix>(entity).unwrap()
    );
}

#[test]
// SAFETY: each call uses this exclusive World loan and its real current
// bound location, and returns None without publishing a protected item.
fn direct_mutable_query_fetches_fail_closed_on_real_bound_storage_locations() {
    let (mut world, entity) = clean_derived_world();
    let matrix_location = real_derived_location::<WorldMatrix>(&mut world, entity);
    let active_location = real_derived_location::<ActiveInHierarchy>(&mut world, entity);
    let generation = world.world_generation();
    let ticks = ChangeTickWindow::all(world.read_change_tick());

    assert!(unsafe {
        <&mut WorldMatrix as QueryMutData>::fetch_mut_with_component_locations(
            &mut world as *mut World,
            entity,
            &[matrix_location],
            ticks,
        )
    }
    .is_none());
    assert!(unsafe {
        <Mut<'_, ActiveInHierarchy> as QueryMutData>::fetch_mut_with_component_locations(
            &mut world as *mut World,
            entity,
            &[active_location],
            ticks,
        )
    }
    .is_none());
    assert!(unsafe {
        <Mut<'_, WorldMatrix> as QueryMutData>::fetch_mut_with_ticks(
            &mut world as *mut World,
            entity,
            ticks,
        )
    }
    .is_none());
    assert!(unsafe {
        World::query_component_mut_at_location::<WorldMatrix>(
            &mut world as *mut World,
            entity,
            matrix_location,
        )
    }
    .is_none());
    assert!(unsafe {
        World::query_component_mut_with_ticks_at_location::<ActiveInHierarchy>(
            &mut world as *mut World,
            entity,
            active_location,
        )
    }
    .is_none());
    assert_eq!(world.world_generation(), generation);
}

#[test]
fn direct_bundle_spawn_and_insert_reject_derived_components_atomically() {
    let (mut world, entity) = clean_derived_world();
    let generation = world.world_generation();
    let matrix = *world.get::<WorldMatrix>(entity).unwrap();
    let active = *world.get::<ActiveInHierarchy>(entity).unwrap();

    assert_eq!(
        world.spawn((Unrelated(1), WorldMatrix::default())),
        Err(SceneError::ProtectedDerivedComponentMutation {
            component: "WorldMatrix",
            operation: "insert",
        })
    );
    assert_eq!(
        world.insert_bundle(entity, (Unrelated(2), ActiveInHierarchy(false))),
        Err(SceneError::ProtectedDerivedComponentMutation {
            component: "ActiveInHierarchy",
            operation: "insert",
        })
    );
    assert!(world.get::<Unrelated>(entity).is_none());
    assert_eq!(world.world_generation(), generation);
    assert_derived_unchanged(&world, entity, matrix, active);
}

#[test]
fn deferred_insert_bundle_and_remove_reject_at_apply_without_publishing_partial_rows() {
    let (mut world, entity) = clean_derived_world();
    let matrix = *world.get::<WorldMatrix>(entity).unwrap();
    let active = *world.get::<ActiveInHierarchy>(entity).unwrap();
    {
        let mut commands = world.commands();
        commands.insert_bundle(entity, (Unrelated(3), WorldMatrix::default()));
        commands.insert(entity, Unrelated(4));
    }
    let report = world.apply_deferred();
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::Insert
    );
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::ProtectedDerivedComponentMutation {
            component: "WorldMatrix",
            operation: "insert",
        }
    );
    assert!(world.get::<Unrelated>(entity).is_none());
    assert_derived_unchanged(&world, entity, matrix, active);

    {
        let mut commands = world.commands();
        commands.remove::<ActiveInHierarchy>(entity);
        commands.insert(entity, Unrelated(5));
    }
    let report = world.apply_deferred();
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::Remove
    );
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::ProtectedDerivedComponentMutation {
            component: "ActiveInHierarchy",
            operation: "remove",
        }
    );
    assert!(world.get::<Unrelated>(entity).is_none());
    assert_derived_unchanged(&world, entity, matrix, active);
}

#[test]
fn deferred_spawn_with_derived_component_rejects_before_entity_publication() {
    let (mut world, _entity) = clean_derived_world();
    let deferred = world
        .commands()
        .spawn((Unrelated(6), ActiveInHierarchy(false)))
        .into_deferred_entity();
    let report = world.apply_deferred();
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::ProtectedDerivedComponentMutation {
            component: "ActiveInHierarchy",
            operation: "insert",
        }
    );
    assert!(report.resolve(&deferred).is_none());
}

#[test]
fn reflection_reports_derived_removal_as_read_only_and_internal_projection_still_rebuilds() {
    let (mut world, entity) = clean_derived_world();
    let adapter = world
        .type_registry()
        .runtime_registration("ActiveInHierarchy")
        .expect("built-in derived component registration")
        .component
        .clone()
        .expect("active state reflection adapter");
    assert_eq!(
        adapter.remove(&mut world, entity),
        Err(ReflectError::NonRemovableComponent {
            type_path: "zircon_runtime::scene::components::ActiveInHierarchy".to_string(),
        })
    );
    let generic_adapter = crate::scene::derived_component_registration::<ActiveInHierarchy>()
        .expect("generic adapter can be constructed for the same protected type")
        .component
        .expect("generic reflected component adapter");
    assert_eq!(
        generic_adapter.remove(&mut world, entity),
        Err(ReflectError::NonRemovableComponent {
            type_path: "zircon_runtime::scene::components::ActiveInHierarchy".to_string(),
        })
    );
    assert!(world.get::<ActiveInHierarchy>(entity).unwrap().0);

    world.set_active_self(entity, false).unwrap();
    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        )
        .unwrap();
    world.flush_pending_scene_systems();
    assert_eq!(
        world.get::<ActiveInHierarchy>(entity),
        Some(&ActiveInHierarchy(false))
    );
    assert_eq!(
        world.get::<WorldMatrix>(entity).map(|matrix| matrix.0),
        world.world_matrix(entity)
    );
    assert_eq!(world.world_transform(entity).unwrap().translation.x, 3.0);
}
