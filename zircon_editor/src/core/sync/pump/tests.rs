use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration, WatchToken, WorldFact};
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1,
};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, EditorViewInvalidationMask, SharedEditorMessageBus,
};
use crate::core::gateway::{
    EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError, InProcessGateway,
    RuntimeCapabilities,
};

use super::{WorldSyncPump, WorldSyncPumpError, TOPIC_WORLD_FACT};

struct BlockingWatchGateway {
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
    token: WatchToken,
}

impl EditorRuntimeGateway for BlockingWatchGateway {
    fn session_handle(&self) -> zircon_runtime_interface::ZrRuntimeSessionHandle {
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(1)
    }

    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        Arc::new(RuntimeCapabilities::editor_default())
    }

    fn watch_world(&self, _registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        self.entered.wait();
        self.release.wait();
        Ok(self.token)
    }

    fn unwatch_world(&self, _token: WatchToken) -> Result<bool, GatewayError> {
        Ok(true)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

struct TrackingWatchGateway {
    token: WatchToken,
    watch_calls: Arc<AtomicUsize>,
    unwatch_calls: Arc<AtomicUsize>,
}

impl TrackingWatchGateway {
    fn new(token: WatchToken) -> Self {
        Self {
            token,
            watch_calls: Arc::new(AtomicUsize::new(0)),
            unwatch_calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl EditorRuntimeGateway for TrackingWatchGateway {
    fn session_handle(&self) -> zircon_runtime_interface::ZrRuntimeSessionHandle {
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(2)
    }

    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        Arc::new(RuntimeCapabilities::editor_default())
    }

    fn watch_world(&self, _registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        self.watch_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.token)
    }

    fn unwatch_world(&self, _token: WatchToken) -> Result<bool, GatewayError> {
        self.unwatch_calls.fetch_add(1, Ordering::SeqCst);
        Ok(true)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

fn hierarchy_watch_registration() -> WatchRegistration {
    WatchRegistration::new(WatchKey::WorldStructure)
}

fn hierarchy_watch_view() -> ViewInstanceId {
    ViewInstanceId::new("hierarchy")
}

#[test]
fn pump_forwards_facts_and_projects_dirty_tokens_once_per_runtime_drain() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        level.clone(),
    )));
    let bus = SharedEditorMessageBus::default();
    let subscriber = bus
        .register_subscriber([EditorTopic::parse(TOPIC_WORLD_FACT).unwrap()])
        .unwrap();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let mut pump = WorldSyncPump::default();

    let token = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();

    let mut spawned = None;
    gateway
        .with_world_mut(&mut |world| spawned = Some(world.spawn_node(NodeKind::Empty)))
        .unwrap();
    let spawned = spawned.expect("world mutation should return the new entity");

    let report = pump.pump(&gateway, &bus).unwrap();
    assert!(report.transport_available());
    assert_eq!(report.batches(), 1);
    assert_eq!(report.published_facts(), 1);
    assert_eq!(report.matched_tokens(), 1);
    assert_eq!(report.dirty_views(), 1);
    assert_eq!(report.last_generation(), Some(level.world_generation()));
    assert_eq!(
        bus.drain_dirty().mask_for(&hierarchy),
        Some(EditorViewInvalidationMask::TREE_STRUCTURE)
    );

    let deliveries = bus.drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].topic().as_str(), TOPIC_WORLD_FACT);
    match deliveries[0].message().payload() {
        EditorMessagePayload::Custom { schema_id, payload } => {
            assert_eq!(schema_id, "zircon.editor.world_fact.v1");
            assert_eq!(
                serde_json::from_value::<WorldFact>(payload.clone()).unwrap(),
                WorldFact::Spawned(spawned)
            );
        }
        unexpected => panic!("world fact must use the custom fact envelope, got {unexpected:?}"),
    }

    let next = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(next.batches(), 0);
    assert_eq!(next.published_facts(), 0);
    assert_eq!(next.matched_tokens(), 0);
    assert_eq!(next.dirty_views(), 0);
    assert!(bus.drain_dirty().is_empty());
    assert!(bus.drain_deliveries(subscriber).is_empty());

    assert!(pump.unwatch_view(&gateway, token).unwrap());
}

#[test]
fn repeated_view_watch_registration_reuses_the_existing_runtime_token() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(level)));
    let bus = SharedEditorMessageBus::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let mut pump = WorldSyncPump::default();

    let first = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    let repeated = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();

    assert_eq!(repeated, first);
    assert_eq!(pump.watches().len(), 1);

    gateway
        .with_world_mut(&mut |world| {
            world.spawn_node(NodeKind::Empty);
        })
        .unwrap();
    let report = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(report.matched_tokens(), 1);
    assert_eq!(report.dirty_views(), 1);
    assert_eq!(
        bus.drain_dirty().mask_for(&hierarchy),
        Some(EditorViewInvalidationMask::TREE_STRUCTURE)
    );
}

#[test]
fn watch_registration_receipt_keeps_the_token_with_its_issuing_gateway_generation() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(level)));
    let mut pump = WorldSyncPump::default();

    let (token, generation) = pump
        .watch_view_with_gateway_generation(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("watch registration should report its issuing session");

    assert_eq!(generation, gateway.generation());
    gateway
        .replace(Arc::new(TrackingWatchGateway::new(token)))
        .expect("gateway replacement should succeed");
    assert_ne!(generation, gateway.generation());
}

#[test]
fn live_token_collision_preserves_the_existing_editor_binding() {
    let mut pump = WorldSyncPump::default();
    let token = WatchToken::new(7);
    let hierarchy = ViewInstanceId::new("hierarchy");
    pump.watches
        .bind(
            token,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("fixture token should bind");

    let error = pump
        .reject_live_watch_token(token)
        .expect_err("a reused runtime token must be rejected");

    assert!(
        matches!(error, WorldSyncPumpError::TokenCollision { token: actual } if actual == token)
    );
    assert_eq!(pump.watches().binding(token).unwrap().view(), &hierarchy);
}

#[test]
fn unwatch_removes_the_editor_projection_without_suppressing_runtime_facts() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(level)));
    let bus = SharedEditorMessageBus::default();
    let subscriber = bus
        .register_subscriber([EditorTopic::parse(TOPIC_WORLD_FACT).unwrap()])
        .unwrap();
    let mut pump = WorldSyncPump::default();
    let token = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            ViewInstanceId::new("hierarchy"),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    assert!(pump.unwatch_view(&gateway, token).unwrap());

    gateway
        .with_world_mut(&mut |world| {
            world.spawn_node(NodeKind::Empty);
        })
        .unwrap();

    let report = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(report.published_facts(), 1);
    assert_eq!(report.matched_tokens(), 0);
    assert_eq!(report.dirty_views(), 0);
    assert!(bus.drain_dirty().is_empty());
    assert_eq!(bus.drain_deliveries(subscriber).len(), 1);
}

#[test]
fn gateway_replacement_discards_retired_session_tokens_before_the_next_drain() {
    let first_level =
        DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let second_level =
        DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        first_level,
    )));
    let bus = SharedEditorMessageBus::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        WatchRegistration::new(WatchKey::WorldStructure),
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    assert_eq!(pump.watches().len(), 1);

    gateway
        .replace(Arc::new(InProcessGateway::for_authoring_level(
            second_level,
        )))
        .unwrap();
    gateway
        .with_world_mut(&mut |world| {
            world.spawn_node(NodeKind::Empty);
        })
        .unwrap();

    let report = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(report.published_facts(), 1);
    assert_eq!(report.matched_tokens(), 0);
    assert!(bus.drain_dirty().is_empty());
    assert!(pump.watches().is_empty());
}

#[test]
fn watch_registration_cannot_bind_a_token_from_a_gateway_replaced_mid_registration() {
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let token = WatchToken::new(71);
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(BlockingWatchGateway {
        entered: entered.clone(),
        release: release.clone(),
        token,
    }));
    let replacement_gateway = Arc::new(TrackingWatchGateway::new(token));
    let replacement_watch_calls = replacement_gateway.watch_calls.clone();
    let worker_gateway = gateway.clone();
    let worker = std::thread::spawn(move || {
        let mut pump = WorldSyncPump::default();
        let token = pump
            .watch_view(
                &worker_gateway,
                hierarchy_watch_registration(),
                hierarchy_watch_view(),
                EditorViewInvalidationMask::TREE_STRUCTURE,
            )
            .expect("first session watch should register");
        (pump, token)
    });

    entered.wait();
    let (replacement_started_sender, replacement_started) = mpsc::channel();
    let replacement_complete = Arc::new(AtomicBool::new(false));
    let replacement_complete_for_thread = replacement_complete.clone();
    let replacement_handle = gateway.clone();
    let replacement = std::thread::spawn(move || {
        replacement_started_sender
            .send(())
            .expect("replacement start receiver should remain alive");
        replacement_handle
            .replace(replacement_gateway)
            .expect("gateway replacement should succeed");
        replacement_complete_for_thread.store(true, Ordering::SeqCst);
    });
    replacement_started
        .recv()
        .expect("replacement thread should start");
    let deadline = Instant::now() + Duration::from_millis(100);
    while !replacement_complete.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        !replacement_complete.load(Ordering::SeqCst),
        "gateway replacement must wait for the in-flight watch registration"
    );

    release.wait();
    let (mut pump, first_token) = worker.join().expect("watch worker should complete");
    replacement
        .join()
        .expect("replacement worker should complete");

    let current_token = pump
        .watch_view(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("current generation should receive its own watch");
    assert_eq!(first_token, token);
    assert_eq!(current_token, token);
    assert_eq!(replacement_watch_calls.load(Ordering::SeqCst), 1);
    assert_eq!(pump.watches().len(), 1);
}

#[test]
fn stale_unwatch_after_gateway_replacement_cannot_revoke_a_current_session_token() {
    let token = WatchToken::new(73);
    let first_gateway = Arc::new(TrackingWatchGateway::new(token));
    let gateway = EditorRuntimeGatewayHandle::new(first_gateway);
    let mut pump = WorldSyncPump::default();
    let stale_token = pump
        .watch_view(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("first generation should register a watch");

    let current_gateway = Arc::new(TrackingWatchGateway::new(token));
    let current_unwatch_calls = current_gateway.unwatch_calls.clone();
    gateway
        .replace(current_gateway)
        .expect("gateway replacement should succeed");

    assert!(!pump.unwatch_view(&gateway, stale_token).unwrap());
    assert_eq!(current_unwatch_calls.load(Ordering::SeqCst), 0);
    assert!(pump.watches().is_empty());
}

#[test]
fn shared_bus_exposes_a_borrowed_single_view_dirty_path() {
    let shared_bus_source = include_str!("../../editor_message/shared.rs");
    let bus_source = include_str!("../../editor_message/bus.rs");

    assert!(shared_bus_source.contains("pub fn mark_view_dirty_ref("));
    assert!(shared_bus_source.contains("self.lock().mark_view_dirty_ref(view, mask)"));
    assert!(bus_source.contains("pub fn mark_view_dirty_ref("));
    assert!(bus_source.contains("self.dirty.mark_ref(view, mask);"));
}

#[test]
fn pump_submits_each_projected_dirty_set_through_one_bus_lock_boundary() {
    let pump_source = include_str!("../pump.rs");
    let shared_bus_source = include_str!("../../editor_message/shared.rs");
    let bus_source = include_str!("../../editor_message/bus.rs");

    assert!(pump_source.contains("bus.mark_view_dirty_set(projection.dirty());"));
    assert!(shared_bus_source.contains("pub fn mark_view_dirty_set("));
    assert!(shared_bus_source.contains("self.lock().mark_view_dirty_set(dirty);"));
    assert!(bus_source.contains("pub fn mark_view_dirty_set("));
    assert!(bus_source.contains("for (view, mask) in dirty.iter()"));
}
