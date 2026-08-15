use std::sync::atomic::Ordering;

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchKey, WatchRegistration, WatchToken, WorldFact, WorldQuery,
    WorldQueryResult,
};

use crate::core::gateway::{EditorRuntimeGateway, GatewayError};

use super::fixture::{
    api_table, fake_watch_world_invalid_token, gateway, FREED_OUTPUTS, OUTPUT_TEST_LOCK,
    WORLD_QUERY_REQUESTS, WORLD_WATCH_REQUESTS,
};

#[test]
fn session_gateway_transports_world_sync_over_owned_abi_buffers() {
    let _guard = OUTPUT_TEST_LOCK.lock().unwrap();
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    WORLD_QUERY_REQUESTS.lock().unwrap().clear();
    WORLD_WATCH_REQUESTS.lock().unwrap().clear();
    let gateway = gateway(api_table());
    let query = WorldQuery {
        generation_hint: Some(72),
        ..WorldQuery::default()
    };
    let registration = WatchRegistration::new(WatchKey::WorldStructure);

    assert_eq!(
        gateway.query_world(query.clone()),
        Ok(WorldQueryResult::NotModified { generation: 73 })
    );
    let token = gateway
        .watch_world(registration.clone())
        .expect("runtime issues a watch token");
    assert_eq!(token, WatchToken::new(41));
    assert!(gateway
        .unwatch_world(token)
        .expect("runtime revokes the watch"));
    assert_eq!(
        gateway
            .drain_world_invalidations()
            .expect("runtime drains invalidations"),
        vec![InvalidationBatch {
            generation: 73,
            dirty: vec![WatchToken::new(41)],
            facts: vec![WorldFact::Spawned(7)],
        }]
    );
    assert_eq!(*WORLD_QUERY_REQUESTS.lock().unwrap(), vec![query]);
    assert_eq!(*WORLD_WATCH_REQUESTS.lock().unwrap(), vec![registration]);
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 2);
}

#[test]
fn session_gateway_rejects_invalid_runtime_world_watch_token() {
    let mut api = api_table();
    api.watch_world = Some(fake_watch_world_invalid_token);
    let error = gateway(api)
        .watch_world(WatchRegistration::new(WatchKey::WorldStructure))
        .expect_err("zero is never a valid runtime-issued watch token");

    assert!(matches!(error, GatewayError::Protocol { .. }));
}
