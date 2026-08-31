use std::sync::Arc;

use serde_json::json;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::components::Name;
use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};
use zircon_runtime_interface::world_sync::{
    ComponentSelector, ComponentWorldQuery, QueryFilter, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration, WorldFact};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle,
    EditorRuntimeHighlightSet, GatewayError, InProcessGateway,
};

#[test]
fn in_process_gateway_reads_and_mutates_the_owned_level_without_serialization() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_default_level();
    let gateway = InProcessGateway::new(runtime.handle(), level.clone());

    let mut generation = None;
    gateway
        .with_world(&mut |world: &World| generation = Some(world.world_generation()))
        .unwrap();

    let mut spawned = None;
    gateway
        .with_world_mut(&mut |world: &mut World| {
            spawned = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            );
        })
        .unwrap();

    let spawned = spawned.expect("in-process mutation should return the spawned entity");
    assert!(level.with_world(|world| world.contains_entity(spawned)));
    assert!(level.with_world(World::world_generation) > generation.unwrap());
}

#[test]
fn in_process_submission_updates_only_its_viewport_latest_value() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = InProcessGateway::for_authoring_level(level.clone());

    gateway
        .submit_highlight_set(EditorRuntimeHighlightSet::new(
            ZrRuntimeViewportHandle::new(2),
            5,
            [8, 2, 8],
            true,
            [0.2, 0.5, 0.8, 1.0],
        ))
        .unwrap();
    gateway
        .submit_highlight_set(EditorRuntimeHighlightSet::new(
            ZrRuntimeViewportHandle::new(2),
            4,
            [99],
            true,
            [0.2, 0.5, 0.8, 1.0],
        ))
        .unwrap();

    let retained = level.viewport_highlight_set(2).unwrap();
    assert_eq!(retained.generation(), 5);
    assert_eq!(retained.set().entities(), &[2, 8]);
    assert!(level.viewport_highlight_set(3).is_none());
}

#[test]
fn in_process_gateway_production_owner_has_no_inline_concrete_level_fixture() {
    let source = include_str!("../../core/gateway/in_process.rs");

    assert!(
        !source.contains("#[cfg(test)]"),
        "gateway production owner must not retain inline test modules"
    );
    assert!(
        !source.contains("DefaultLevelManager"),
        "gateway production owner must not construct a concrete level manager"
    );
}

#[test]
fn in_process_gateway_routes_structural_facts_to_live_watch_tokens() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = InProcessGateway::new(runtime.handle(), level.clone());

    let mut root = None;
    gateway
        .with_world_mut(&mut |world| {
            root = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
        .unwrap();
    let root = root.expect("root spawn must produce an entity");
    assert_eq!(gateway.drain_world_invalidations().unwrap().len(), 1);

    let structure = gateway
        .watch_world(WatchRegistration::new(WatchKey::WorldStructure))
        .unwrap();
    let subtree = gateway
        .watch_world(WatchRegistration::new(WatchKey::Subtree { root }))
        .unwrap();

    let mut staging_snapshot = level.snapshot();
    staging_snapshot
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    assert!(gateway.drain_world_invalidations().unwrap().is_empty());

    let mut child = None;
    gateway
        .with_world_mut(&mut |world| {
            let entity = world
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed");
            world.set_parent_checked(entity, Some(root)).unwrap();
            child = Some(entity);
        })
        .unwrap();
    let child = child.expect("child spawn must produce an entity");

    let batches = gateway.drain_world_invalidations().unwrap();
    assert_eq!(batches.len(), 1);
    let batch = &batches[0];
    assert_eq!(batch.dirty, vec![structure, subtree]);
    assert!(matches!(
        batch.facts.as_slice(),
        [WorldFact::Reparented {
            entity,
            new_parent: Some(parent)
        }] if *entity == child && *parent == root
    ));

    assert!(gateway.unwatch_world(subtree).unwrap());
    assert!(!gateway.unwatch_world(subtree).unwrap());
    gateway
        .with_world_mut(&mut |world| {
            assert!(world.set_parent_checked(child, None).unwrap());
        })
        .unwrap();

    let batches = gateway.drain_world_invalidations().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].dirty, vec![structure]);
}

#[test]
fn in_process_gateway_queries_the_same_reflected_projection_as_its_level() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = InProcessGateway::new(runtime.handle(), level.clone());

    let mut entity = None;
    gateway
        .with_world_mut(&mut |world| {
            entity = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
        .unwrap();
    let entity = entity.expect("spawn must produce a queryable entity");
    let component_type = level.with_world(|world| {
        world
            .inspection_fields_artifact(entity)
            .expect("default nodes expose reflected editor fields")
            .fields()
            .iter()
            .next()
            .expect("default nodes expose reflected editor fields")
            .component_type_path
            .clone()
    });
    let query = WorldQuery::Components(ComponentWorldQuery {
        filter: QueryFilter {
            with: vec![component_type.clone()],
            without: Vec::new(),
        },
        select: vec![ComponentSelector::new(component_type)],
        generation_hint: None,
    });

    let result = gateway.query_world(query.clone()).unwrap();
    assert!(matches!(
        result,
        WorldQueryResult::ComponentRows { ref rows, .. } if rows.len() == 1
    ));
    assert_eq!(result, level.with_world(|world| world.query_world(&query)));

    let current_generation = level.world_generation();
    assert_eq!(
        gateway
            .query_world(query.with_generation_hint(Some(current_generation)))
            .unwrap(),
        WorldQueryResult::NotModified {
            generation: current_generation
        }
    );
}

#[test]
fn in_process_gateway_routes_component_type_mutations_by_reflection_type_path() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = InProcessGateway::new(runtime.handle(), level.clone());

    let mut entity = None;
    gateway
        .with_world_mut(&mut |world| {
            entity = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
        .unwrap();
    let entity = entity.expect("spawn must produce an entity");
    assert_eq!(gateway.drain_world_invalidations().unwrap().len(), 1);

    let component = gateway
        .watch_world(WatchRegistration::new(WatchKey::ComponentType {
            type_name: std::any::type_name::<Name>().to_string(),
        }))
        .unwrap();
    gateway
        .with_world_mut(&mut |world| {
            assert!(world.rename_node(entity, "Renamed").unwrap());
        })
        .unwrap();

    let batches = gateway.drain_world_invalidations().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].dirty, vec![component]);
    assert!(batches[0].facts.is_empty());
}

#[test]
fn in_process_gateway_routes_dynamic_component_mutations_by_canonical_type_id() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = InProcessGateway::new(runtime.handle(), level.clone());

    let mut entity = None;
    gateway
        .with_world_mut(&mut |world| {
            entity = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
        .unwrap();
    let entity = entity.expect("spawn must produce an entity");
    assert_eq!(gateway.drain_world_invalidations().unwrap().len(), 1);

    let component_id = "tests.Health";
    let token = gateway
        .watch_world(WatchRegistration::new(WatchKey::ComponentType {
            type_name: component_id.to_string(),
        }))
        .unwrap();
    gateway
        .with_world_mut(&mut |world| {
            assert!(world
                .set_dynamic_component(entity, component_id, json!({ "hp": 100 }))
                .unwrap());
            assert!(world
                .set_dynamic_component(entity, component_id, json!({ "hp": 80 }))
                .unwrap());
        })
        .unwrap();

    let batches = gateway.drain_world_invalidations().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].dirty, vec![token]);
    assert!(batches[0].facts.is_empty());
}

#[test]
fn gateway_handle_forwards_borrowed_world_access_to_the_current_transport() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_default_level();
    let handle = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::new(
        runtime.handle(),
        level.clone(),
    )));

    let mut spawned = None;
    handle
        .with_world_mut(&mut |world| {
            spawned = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
        .unwrap();

    assert!(level.with_world(|world| world.contains_entity(spawned.unwrap())));
}

#[test]
fn in_process_gateway_rejects_write_reentry_from_a_world_read_callback() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_default_level();
    let handle =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::new(runtime.handle(), level)));
    let nested_handle = handle.clone();
    let mut nested_results = Vec::new();

    handle
        .with_world(&mut |_world| {
            nested_results.push(nested_handle.with_world_mut(&mut |_world| {
                panic!("reentrant callback must not acquire the world lock")
            }));
            nested_results.push(nested_handle.with_world_mut(&mut |_world| {
                panic!("repeated reentry must not acquire the world lock")
            }));
        })
        .unwrap();

    assert_eq!(
        nested_results,
        vec![
            Err(GatewayError::ReentrantBorrowedWorldAccess),
            Err(GatewayError::ReentrantBorrowedWorldAccess),
        ]
    );
}

#[test]
fn in_process_gateway_rejects_read_reentry_from_a_world_write_callback() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_default_level();
    let handle =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::new(runtime.handle(), level)));
    let nested_handle = handle.clone();
    let mut nested_result = None;

    handle
        .with_world_mut(&mut |_world| {
            nested_result = Some(nested_handle.with_world(&mut |_world| {
                panic!("reentrant callback must not acquire the world lock")
            }));
        })
        .unwrap();

    assert_eq!(
        nested_result,
        Some(Err(GatewayError::ReentrantBorrowedWorldAccess))
    );
}

#[test]
fn in_process_gateway_clears_the_reentry_guard_after_a_callback_panics() {
    let runtime = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_default_level();
    let gateway = InProcessGateway::new(runtime.handle(), level);

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = gateway.with_world(&mut |_world| panic!("test callback panic"));
    }));

    assert!(panic.is_err());
    let mut generation = None;
    gateway
        .with_world(&mut |world| generation = Some(world.world_generation()))
        .unwrap();
    assert!(generation.is_some());
}

#[test]
fn in_process_gateway_borrow_guards_are_isolated_between_threads() {
    let runtime = CoreRuntime::new();
    let outer_level = DefaultLevelManager::default().create_default_level();
    let outer_gateway = InProcessGateway::new(runtime.handle(), outer_level);
    let thread_level = DefaultLevelManager::default().create_default_level();
    let thread_gateway = InProcessGateway::new(runtime.handle(), thread_level);
    let mut thread_result = None;

    outer_gateway
        .with_world(&mut |_world| {
            let thread_gateway = thread_gateway.clone();
            thread_result = Some(
                std::thread::spawn(move || thread_gateway.with_world(&mut |_world| {}))
                    .join()
                    .unwrap(),
            );
        })
        .unwrap();

    assert_eq!(thread_result, Some(Ok(())));
}

#[test]
fn detached_gateway_rejects_borrowed_world_access() {
    let gateway = DetachedEditorRuntimeGateway;
    let mut read = |_world: &World| {};
    let mut write = |_world: &mut World| {};

    assert_eq!(
        gateway.with_world(&mut read),
        Err(GatewayError::RequiresSerializedAccess)
    );
    assert_eq!(
        gateway.with_world_mut(&mut write),
        Err(GatewayError::RequiresSerializedAccess)
    );
}
