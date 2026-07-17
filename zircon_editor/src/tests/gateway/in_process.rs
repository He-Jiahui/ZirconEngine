use std::sync::Arc;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
    InProcessGateway,
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
            spawned = Some(world.spawn_node(NodeKind::Empty));
        })
        .unwrap();

    let spawned = spawned.expect("in-process mutation should return the spawned entity");
    assert!(level.with_world(|world| world.contains_entity(spawned)));
    assert!(level.with_world(World::world_generation) > generation.unwrap());
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
        .with_world_mut(&mut |world| spawned = Some(world.spawn_node(NodeKind::Empty)))
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
